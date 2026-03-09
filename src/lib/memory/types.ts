/** Memory Spine canonical types — shared between desktop and mobile */

export type MemoryTier = 'raw' | 'working' | 'long_term';

export interface RawEvent {
  id: string;
  source: 'mobile' | 'desktop' | 'agent' | 'browser_agent' | 'api';
  eventType: string;
  payload: unknown;
  deviceId: string;
  timestamp: string;
  sessionId: string | null;
}

export interface WorkingMemoryEntry {
  id: string;
  topic: string;
  content: string;
  confidence: number;   // 0-1
  rawEventIds: string[];
  createdAt: string;
  expiresAt: string;   // working memory has TTL (default 7d)
  accessed: number;    // LRU counter
}

export interface LongTermMemoryEntry {
  id: string;
  category: 'task' | 'habit' | 'agent_outcome' | 'user_preference' | 'kaizen';
  summary: string;
  detail: string;
  sourceEventIds: string[];
  notionPageId: string | null;
  kaizenTaskId: string | null;
  createdAt: string;
  updatedAt: string;
  vector: number[] | null;  // embedding for future semantic search
}

export interface KaizenTask {
  id: string;
  title: string;
  description: string;
  sourceType: 'agent_error' | 'user_capture' | 'approval_reject' | 'memory_gap' | 'habit_miss';
  sourceEventId: string;
  priority: 'urgent' | 'high' | 'normal' | 'low';
  status: 'backlog' | 'in_review' | 'implementing' | 'done';
  notionTaskId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface MemoryWriteRequest {
  tier: MemoryTier;
  eventType: string;
  payload: unknown;
  source: RawEvent['source'];
  deviceId?: string;
  sessionId?: string;
  kaizenHint?: string;  // if set, auto-creates a Kaizen task
}

export interface MemoryReadResponse {
  rawEvents: RawEvent[];
  workingMemory: WorkingMemoryEntry[];
  longTermMemory: LongTermMemoryEntry[];
  kaizenTasks: KaizenTask[];
  totalCount: number;
}
