/** Canonical sync event envelope — every write from mobile or desktop */
export interface SyncEvent {
  id: string;              // uuid-v4
  type: SyncEventType;
  payload: unknown;
  deviceId: string;
  userId: string;
  createdAt: string;       // ISO-8601
  syncedAt: string | null; // null = still in offline queue
  vectorClock: Record<string, number>;
  checksum: string;        // sha256 of payload
}

export type SyncEventType =
  | 'task.create'
  | 'task.update'
  | 'task.complete'
  | 'task.delete'
  | 'habit.check'
  | 'habit.create'
  | 'capture.create'
  | 'approval.submit'
  | 'approval.resolve'
  | 'agent.start'
  | 'agent.complete'
  | 'voice.capture'
  | 'memory.write'
  | 'memory.summarize'
  | 'kaizen.create';

export interface Task {
  id: string;
  title: string;
  priority: 'urgent' | 'high' | 'normal' | 'low';
  status: 'todo' | 'in_progress' | 'done' | 'cancelled';
  dueDate: string | null;
  tags: string[];
  source: 'mobile' | 'desktop' | 'agent';
  createdAt: string;
  updatedAt: string;
}

export interface Habit {
  id: string;
  name: string;
  frequency: 'daily' | 'weekly';
  streak: number;
  completedDates: string[];
  color: string;
}

export interface CaptureItem {
  id: string;
  content: string;
  type: 'text' | 'voice' | 'image';
  transcription?: string;
  processed: boolean;
  createdAt: string;
}

export interface ApprovalItem {
  id: string;
  agentId: string;
  agentName: string;
  action: string;
  context: string;
  status: 'pending' | 'approved' | 'rejected';
  priority: 'urgent' | 'normal';
  createdAt: string;
  resolvedAt: string | null;
  resolvedBy: 'mobile' | 'desktop' | null;
}

export interface AgentRun {
  id: string;
  name: string;
  status: 'queued' | 'running' | 'success' | 'failed' | 'waiting_approval';
  startedAt: string | null;
  completedAt: string | null;
  memoryRef: string | null;
  notionTaskId: string | null;
}

export interface DigestEntry {
  date: string;
  tasksCompleted: number;
  habitsCompleted: number;
  agentsRun: number;
  capturesProcessed: number;
  highlights: string[];
  generatedAt: string;
}

export interface ConflictResolution {
  winnerId: string;
  loserId: string;
  strategy: 'last-write-wins' | 'server-wins' | 'client-wins' | 'merge';
  resolvedAt: string;
}
