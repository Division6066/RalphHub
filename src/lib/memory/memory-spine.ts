/**
 * Memory Spine — single source of truth for all mobile + desktop state.
 *
 * Architecture:
 *   Tier 0: Raw events     — immutable append-only log of everything
 *   Tier 1: Working memory — short-lived context (7d TTL, LRU eviction)
 *   Tier 2: Long-term      — summarized, categorized, Notion-synced
 *   Tier 3: Kaizen tasks   — auto-generated improvement tasks from failures
 *
 * Every agent action, Browser Agent run, API call, and capture writes here.
 * Desktop syncs to Notion. Mobile reads / writes via REST on local LAN.
 */
import type {
  RawEvent,
  WorkingMemoryEntry,
  LongTermMemoryEntry,
  KaizenTask,
  MemoryWriteRequest,
  MemoryReadResponse,
} from './types.js';
import { storage } from '../sync/encrypted-storage.js';

const RAW_KEY = 'memory_raw';
const WORKING_KEY = 'memory_working';
const LONG_TERM_KEY = 'memory_longterm';
const KAIZEN_KEY = 'memory_kaizen';
const WORKING_TTL_MS = 7 * 24 * 60 * 60 * 1000;
const MAX_WORKING = 500;
const MAX_RAW = 2000;

function uuid(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    return (c === 'x' ? r : (r & 0x3) | 0x8).toString(16);
  });
}

export class MemorySpine {
  private raw: RawEvent[] = [];
  private working: WorkingMemoryEntry[] = [];
  private longTerm: LongTermMemoryEntry[] = [];
  private kaizen: KaizenTask[] = [];
  private desktopEndpoint: string | null = null;

  async load(): Promise<void> {
    this.raw = (await storage.get<RawEvent[]>(RAW_KEY)) ?? [];
    this.working = (await storage.get<WorkingMemoryEntry[]>(WORKING_KEY)) ?? [];
    this.longTerm = (await storage.get<LongTermMemoryEntry[]>(LONG_TERM_KEY)) ?? [];
    this.kaizen = (await storage.get<KaizenTask[]>(KAIZEN_KEY)) ?? [];
    this.evictExpiredWorking();
  }

  setDesktopEndpoint(endpoint: string): void {
    this.desktopEndpoint = endpoint;
  }

  /**
   * Write to memory — canonical entry point for ALL events.
   * Returns the written event ID for reference chains.
   */
  async write(req: MemoryWriteRequest): Promise<string> {
    const raw: RawEvent = {
      id: uuid(),
      source: req.source,
      eventType: req.eventType,
      payload: req.payload,
      deviceId: req.deviceId ?? 'mobile',
      timestamp: new Date().toISOString(),
      sessionId: req.sessionId ?? null,
    };

    this.raw.unshift(raw);
    if (this.raw.length > MAX_RAW) this.raw = this.raw.slice(0, MAX_RAW);
    await storage.set(RAW_KEY, this.raw);

    // Elevate to working memory
    const working: WorkingMemoryEntry = {
      id: uuid(),
      topic: req.eventType,
      content: JSON.stringify(req.payload),
      confidence: 0.8,
      rawEventIds: [raw.id],
      createdAt: new Date().toISOString(),
      expiresAt: new Date(Date.now() + WORKING_TTL_MS).toISOString(),
      accessed: 0,
    };
    this.working.unshift(working);
    this.evictExpiredWorking();
    await storage.set(WORKING_KEY, this.working);

    // Auto-create Kaizen if hinted
    if (req.kaizenHint) {
      await this.createKaizen({
        title: req.kaizenHint,
        description: `Auto-created from ${req.eventType}: ${JSON.stringify(req.payload).slice(0, 200)}`,
        sourceType: 'memory_gap',
        sourceEventId: raw.id,
        priority: 'normal',
      });
    }

    // Forward to desktop if endpoint available
    if (this.desktopEndpoint) {
      this.forwardToDesktop(raw).catch(console.error);
    }

    return raw.id;
  }

  async createKaizen(opts: Omit<KaizenTask, 'id' | 'status' | 'notionTaskId' | 'createdAt' | 'updatedAt'>): Promise<KaizenTask> {
    const task: KaizenTask = {
      id: uuid(),
      status: 'backlog',
      notionTaskId: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      ...opts,
    };
    this.kaizen.unshift(task);
    await storage.set(KAIZEN_KEY, this.kaizen);
    return task;
  }

  async promoteToLongTerm(
    workingIds: string[],
    category: LongTermMemoryEntry['category'],
    summary: string,
    detail: string
  ): Promise<LongTermMemoryEntry> {
    const rawIds = this.working
      .filter((w) => workingIds.includes(w.id))
      .flatMap((w) => w.rawEventIds);

    const entry: LongTermMemoryEntry = {
      id: uuid(),
      category,
      summary,
      detail,
      sourceEventIds: rawIds,
      notionPageId: null,
      kaizenTaskId: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      vector: null,
    };
    this.longTerm.unshift(entry);
    await storage.set(LONG_TERM_KEY, this.longTerm);
    return entry;
  }

  async read(opts: { limit?: number; source?: string; since?: string } = {}): Promise<MemoryReadResponse> {
    const { limit = 50, source, since } = opts;
    const sinceDate = since ? new Date(since) : null;

    const filterRaw = this.raw
      .filter((e) => (!source || e.source === source) && (!sinceDate || new Date(e.timestamp) >= sinceDate))
      .slice(0, limit);

    return {
      rawEvents: filterRaw,
      workingMemory: this.working.slice(0, limit),
      longTermMemory: this.longTerm.slice(0, limit),
      kaizenTasks: this.kaizen.slice(0, limit),
      totalCount: this.raw.length,
    };
  }

  getRawEvents(): RawEvent[] { return this.raw; }
  getWorkingMemory(): WorkingMemoryEntry[] { return this.working; }
  getLongTermMemory(): LongTermMemoryEntry[] { return this.longTerm; }
  getKaizenTasks(): KaizenTask[] { return this.kaizen; }

  private evictExpiredWorking(): void {
    const now = Date.now();
    this.working = this.working
      .filter((w) => new Date(w.expiresAt).getTime() > now)
      .sort((a, b) => b.accessed - a.accessed)
      .slice(0, MAX_WORKING);
  }

  private async forwardToDesktop(event: RawEvent): Promise<void> {
    if (!this.desktopEndpoint) return;
    await fetch(`${this.desktopEndpoint}/api/memory/write`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(event),
      signal: AbortSignal.timeout(5000),
    });
  }
}

export const memorySpine = new MemorySpine();
