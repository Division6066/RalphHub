/**
 * Core sync engine — orchestrates encrypted storage, offline queue,
 * conflict resolution, and real-time connection to desktop Ralph.
 */
import type { SyncEvent, SyncEventType, Task, Habit, CaptureItem, ApprovalItem, AgentRun } from './types.js';
import { storage } from './encrypted-storage.js';
import { offlineQueue } from './offline-queue.js';
import { resolve, incrementClock } from './conflict-resolver.js';

const DEVICE_ID_KEY = 'device_id';
const VECTOR_CLOCK_KEY = 'vector_clock';
const ENDPOINT_KEY = 'desktop_endpoint';

function uuid(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    return (c === 'x' ? r : (r & 0x3) | 0x8).toString(16);
  });
}

async function sha256(data: unknown): Promise<string> {
  const bytes = new TextEncoder().encode(JSON.stringify(data));
  const hash = await crypto.subtle.digest('SHA-256', bytes);
  return Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

export class SyncEngine {
  private deviceId: string = '';
  private vectorClock: Record<string, number> = {};
  private endpoint: string | null = null;
  private pollingInterval: ReturnType<typeof setInterval> | null = null;

  async init(passphrase: string): Promise<void> {
    await storage.init(passphrase);
    await offlineQueue.load();

    // Stable device ID
    let did = await storage.get<string>(DEVICE_ID_KEY);
    if (!did) {
      did = `mobile_${uuid()}`;
      await storage.set(DEVICE_ID_KEY, did);
    }
    this.deviceId = did;

    // Vector clock
    const clock = await storage.get<Record<string, number>>(VECTOR_CLOCK_KEY);
    this.vectorClock = clock ?? {};

    // Desktop endpoint
    const ep = await storage.get<string>(ENDPOINT_KEY);
    if (ep) {
      this.endpoint = ep;
      offlineQueue.setEndpoint(ep);
    }

    this.startPolling();
  }

  setDesktopEndpoint(endpoint: string): void {
    this.endpoint = endpoint;
    offlineQueue.setEndpoint(endpoint);
    storage.set(ENDPOINT_KEY, endpoint);
  }

  getDesktopEndpoint(): string | null {
    return this.endpoint;
  }

  getDeviceId(): string {
    return this.deviceId;
  }

  private startPolling(): void {
    if (this.pollingInterval) clearInterval(this.pollingInterval);
    this.pollingInterval = setInterval(() => {
      offlineQueue.drain();
      if (this.endpoint) this.pullFromDesktop();
    }, 15_000);
  }

  async emit(type: SyncEventType, payload: unknown): Promise<SyncEvent> {
    this.vectorClock = incrementClock(this.vectorClock, this.deviceId);
    await storage.set(VECTOR_CLOCK_KEY, this.vectorClock);

    const event: SyncEvent = {
      id: uuid(),
      type,
      payload,
      deviceId: this.deviceId,
      userId: 'ralph',
      createdAt: new Date().toISOString(),
      syncedAt: null,
      vectorClock: { ...this.vectorClock },
      checksum: await sha256(payload),
    };

    await offlineQueue.push(event);
    return event;
  }

  async pullFromDesktop(): Promise<SyncEvent[]> {
    if (!this.endpoint) return [];
    try {
      const since = await storage.get<string>('last_pull_at') ?? '1970-01-01T00:00:00Z';
      const res = await fetch(
        `${this.endpoint}/api/sync/events?since=${encodeURIComponent(since)}&deviceId=${this.deviceId}`,
        { signal: AbortSignal.timeout(10_000) }
      );
      if (!res.ok) return [];

      const events: SyncEvent[] = await res.json();
      await storage.set('last_pull_at', new Date().toISOString());
      return events;
    } catch {
      return [];
    }
  }

  // High-level helpers
  async captureTask(title: string, priority: Task['priority'] = 'normal'): Promise<SyncEvent> {
    const task: Task = {
      id: uuid(),
      title,
      priority,
      status: 'todo',
      dueDate: null,
      tags: [],
      source: 'mobile',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    return this.emit('task.create', task);
  }

  async checkHabit(habitId: string): Promise<SyncEvent> {
    return this.emit('habit.check', { habitId, date: new Date().toISOString().slice(0, 10) });
  }

  async captureText(content: string): Promise<SyncEvent> {
    const item: CaptureItem = {
      id: uuid(),
      content,
      type: 'text',
      processed: false,
      createdAt: new Date().toISOString(),
    };
    return this.emit('capture.create', item);
  }

  async captureVoice(transcription: string, audioBlob?: Blob): Promise<SyncEvent> {
    const item: CaptureItem = {
      id: uuid(),
      content: transcription,
      type: 'voice',
      transcription,
      processed: false,
      createdAt: new Date().toISOString(),
    };
    return this.emit('voice.capture', item);
  }

  async resolveApproval(approvalId: string, decision: 'approved' | 'rejected'): Promise<SyncEvent> {
    return this.emit('approval.submit', {
      approvalId,
      decision,
      resolvedBy: 'mobile',
      resolvedAt: new Date().toISOString(),
    });
  }
}

export const syncEngine = new SyncEngine();
