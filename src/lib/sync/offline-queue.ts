/**
 * Offline event queue — persists events locally when offline,
 * drains to desktop sync endpoint when connectivity returns.
 */
import type { SyncEvent } from './types.js';
import { storage } from './encrypted-storage.js';

const QUEUE_KEY = 'offline_queue';
const MAX_RETRIES = 5;
const BACKOFF_BASE_MS = 1000;

interface QueuedEvent {
  event: SyncEvent;
  attempts: number;
  nextRetryAt: string;
}

export class OfflineQueue {
  private queue: QueuedEvent[] = [];
  private draining = false;
  private desktopEndpoint: string | null = null;

  async load(): Promise<void> {
    const stored = await storage.get<QueuedEvent[]>(QUEUE_KEY);
    this.queue = stored ?? [];
  }

  async save(): Promise<void> {
    await storage.set(QUEUE_KEY, this.queue);
  }

  setEndpoint(endpoint: string): void {
    this.desktopEndpoint = endpoint;
  }

  async push(event: SyncEvent): Promise<void> {
    this.queue.push({
      event,
      attempts: 0,
      nextRetryAt: new Date().toISOString(),
    });
    await this.save();
    this.drain();
  }

  size(): number {
    return this.queue.length;
  }

  async drain(): Promise<void> {
    if (this.draining || !this.desktopEndpoint || this.queue.length === 0) return;
    this.draining = true;

    const now = new Date();
    const ready = this.queue.filter((q) => new Date(q.nextRetryAt) <= now);

    for (const item of ready) {
      try {
        const res = await fetch(`${this.desktopEndpoint}/api/sync/events`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(item.event),
          signal: AbortSignal.timeout(8000),
        });

        if (res.ok) {
          this.queue = this.queue.filter((q) => q.event.id !== item.event.id);
        } else {
          item.attempts++;
          item.event.syncedAt = null;
          item.nextRetryAt = new Date(
            Date.now() + BACKOFF_BASE_MS * 2 ** item.attempts
          ).toISOString();
          if (item.attempts >= MAX_RETRIES) {
            // Dead-letter: keep but mark failed
            console.error('[OfflineQueue] Max retries exceeded for event', item.event.id);
          }
        }
      } catch {
        item.attempts++;
        item.nextRetryAt = new Date(
          Date.now() + BACKOFF_BASE_MS * 2 ** item.attempts
        ).toISOString();
      }
    }

    await this.save();
    this.draining = false;
  }

  pending(): QueuedEvent[] {
    return [...this.queue];
  }
}

export const offlineQueue = new OfflineQueue();
