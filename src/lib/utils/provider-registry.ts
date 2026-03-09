import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';
import { writable, derived, get } from 'svelte/store';

export type AuthType = 'bearer' | 'x-api-key' | 'api-key-param' | 'api-key-header' | 'key-secret' | 'xi-api-key' | 'authorization' | 'token' | 'basic' | 'apikey' | 'api-key-body' | 'x-bb-api-key' | 'e2b-api-key' | 'none';

export type ProviderCategory =
  | 'llm'
  | 'image'
  | 'video'
  | 'audio'
  | 'search'
  | 'automation'
  | 'voice'
  | 'mcp'
  | 'database'
  | 'custom';

export interface Provider {
  id: string;
  name: string;
  category: ProviderCategory;
  baseUrl: string;
  authType: AuthType;
  apiKeyEnv: string;
  models: string[];
  enabled: boolean;
  isLocal: boolean;
  description: string;
  docsUrl: string;
  logoEmoji: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateProviderRequest {
  name: string;
  category: ProviderCategory;
  baseUrl: string;
  authType: AuthType;
  apiKeyEnv: string;
  models: string[];
  isLocal: boolean;
  description: string;
  docsUrl: string;
  logoEmoji: string;
}

export interface UpdateProviderRequest {
  id: string;
  name?: string;
  baseUrl?: string;
  apiKeyEnv?: string;
  models?: string[];
  enabled?: boolean;
  description?: string;
}

export interface ApiUsageLog {
  id: string;
  providerId: string;
  providerName: string;
  model: string;
  tokensIn: number;
  tokensOut: number;
  costUsd: number;
  outputSummary: string;
  toolId: string;
  workflowId: string;
  createdAt: string;
}

export interface LogApiUsageRequest {
  providerId: string;
  providerName: string;
  model: string;
  tokensIn: number;
  tokensOut: number;
  costUsd: number;
  outputSummary: string;
  toolId: string;
  workflowId: string;
}

export interface KaizenTask {
  id: string;
  title: string;
  description: string;
  status: 'todo' | 'in_progress' | 'done' | 'blocked';
  priority: 'urgent' | 'high' | 'normal' | 'low';
  source: string;
  providerId: string;
  usageLogId: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateKaizenTaskRequest {
  title: string;
  description: string;
  priority: string;
  source: string;
  providerId: string;
  usageLogId: string;
}

export interface MemorySpineStats {
  totalEntries: number;
  totalTokens: number;
  totalCostUsd: number;
  providersUsed: string[];
  recentLogs: ApiUsageLog[];
  activeTasks: KaizenTask[];
}

export interface MemorySpineEntry {
  id: string;
  entryType: string;
  content: string;
  tags: string[];
  providerId: string;
  model: string;
  createdAt: string;
}

// ─── Stores ────────────────────────────────────────────────────────────────────

export const providersStore = writable<Provider[]>([]);
export const providerLoadingStore = writable<boolean>(false);
export const activeProviderIdStore = writable<string>('');
export const activeModelStore = writable<string>('');
export const memoryStatsStore = writable<MemorySpineStats | null>(null);
export const kaizenTasksStore = writable<KaizenTask[]>([]);

export const enabledProvidersStore = derived(providersStore, ($providers) =>
  $providers.filter((p) => p.enabled)
);

export const llmProvidersStore = derived(providersStore, ($providers) =>
  $providers.filter((p) => p.category === 'llm')
);

export const enabledLLMsStore = derived(providersStore, ($providers) =>
  $providers.filter((p) => p.category === 'llm' && p.enabled)
);

export const providersByCategoryStore = derived(providersStore, ($providers) => {
  const map: Record<string, Provider[]> = {};
  for (const p of $providers) {
    if (!map[p.category]) map[p.category] = [];
    map[p.category].push(p);
  }
  return map;
});

// ─── All Models Flat List ────────────────────────────────────────────────────

export const allModelsStore = derived(providersStore, ($providers) => {
  const models: Array<{ providerId: string; providerName: string; model: string; isLocal: boolean; emoji: string }> = [];
  for (const p of $providers.filter((x) => x.enabled || x.isLocal)) {
    for (const m of p.models) {
      models.push({
        providerId: p.id,
        providerName: p.name,
        model: m,
        isLocal: p.isLocal,
        emoji: p.logoEmoji,
      });
    }
  }
  return models;
});

// ─── API Functions ────────────────────────────────────────────────────────────

export async function loadProviders(category?: string): Promise<Provider[]> {
  if (!isDesktopRuntime()) {
    return getMockProviders();
  }
  providerLoadingStore.set(true);
  try {
    const providers = await invokeTauri<Provider[]>('list_providers_cmd', { category });
    providersStore.set(providers);
    return providers;
  } finally {
    providerLoadingStore.set(false);
  }
}

export async function createProvider(req: CreateProviderRequest): Promise<Provider> {
  const provider = await invokeTauri<Provider>('create_provider_cmd', { req });
  providersStore.update((ps) => [...ps, provider]);
  return provider;
}

export async function updateProvider(req: UpdateProviderRequest): Promise<Provider> {
  const updated = await invokeTauri<Provider>('update_provider_cmd', { req });
  providersStore.update((ps) => ps.map((p) => (p.id === updated.id ? updated : p)));
  return updated;
}

export async function deleteProvider(id: string): Promise<void> {
  await invokeTauri('delete_provider_cmd', { id });
  providersStore.update((ps) => ps.filter((p) => p.id !== id));
}

export async function searchProviders(query: string): Promise<Provider[]> {
  if (!isDesktopRuntime()) return getMockProviders().filter((p) => p.name.toLowerCase().includes(query.toLowerCase()));
  return await invokeTauri<Provider[]>('search_providers_cmd', { query });
}

export async function toggleProvider(id: string, enabled: boolean): Promise<Provider> {
  return updateProvider({ id, enabled });
}

// ─── Usage / Memory Spine ─────────────────────────────────────────────────────

export async function logApiUsage(req: LogApiUsageRequest): Promise<ApiUsageLog> {
  if (!isDesktopRuntime()) return mockLog(req);
  const log = await invokeTauri<ApiUsageLog>('log_api_usage_cmd', { req });
  // Auto-create Kaizen task if summary is substantial
  if (req.outputSummary && req.outputSummary.length > 20) {
    await createKaizenTask({
      title: `[Auto] ${req.providerName}/${req.model} output review`,
      description: req.outputSummary.slice(0, 300),
      priority: 'low',
      source: req.toolId || 'api_call',
      providerId: req.providerId,
      usageLogId: log.id,
    });
  }
  // Refresh stats
  refreshMemoryStats();
  return log;
}

export async function getMemoryStats(): Promise<MemorySpineStats> {
  if (!isDesktopRuntime()) return mockMemoryStats();
  const stats = await invokeTauri<MemorySpineStats>('get_memory_spine_stats_cmd');
  memoryStatsStore.set(stats);
  return stats;
}

export async function refreshMemoryStats(): Promise<void> {
  try {
    await getMemoryStats();
  } catch {
    // Non-critical
  }
}

// ─── Kaizen Tasks ──────────────────────────────────────────────────────────────

export async function createKaizenTask(req: CreateKaizenTaskRequest): Promise<KaizenTask> {
  if (!isDesktopRuntime()) return mockKaizenTask(req);
  const task = await invokeTauri<KaizenTask>('create_kaizen_task_cmd', { req });
  kaizenTasksStore.update((ts) => [task, ...ts]);
  return task;
}

export async function loadKaizenTasks(status?: string): Promise<KaizenTask[]> {
  if (!isDesktopRuntime()) return [];
  const tasks = await invokeTauri<KaizenTask[]>('list_kaizen_tasks_cmd', { status });
  kaizenTasksStore.set(tasks);
  return tasks;
}

export async function updateTaskStatus(id: string, status: string): Promise<KaizenTask> {
  const task = await invokeTauri<KaizenTask>('update_kaizen_task_status_cmd', { id, status });
  kaizenTasksStore.update((ts) => ts.map((t) => (t.id === id ? task : t)));
  return task;
}

// ─── Auto-Injection for Tools ─────────────────────────────────────────────────

export function buildEnvInjection(providers: Provider[], keyValues: Record<string, string>): Record<string, string> {
  const env: Record<string, string> = {};
  for (const p of providers) {
    if (p.enabled && p.apiKeyEnv && keyValues[p.apiKeyEnv]) {
      env[p.apiKeyEnv] = keyValues[p.apiKeyEnv];
    }
  }
  // Always inject Ollama host
  const ollamaProvider = providers.find((p) => p.id === 'builtin-ollama-(local)' || p.name === 'Ollama (Local)');
  if (ollamaProvider) {
    env['OLLAMA_HOST'] = 'http://localhost:11434';
  }
  return env;
}

export function getProviderForTool(providers: Provider[], toolRequiredKeys: string[]): Provider | null {
  for (const key of toolRequiredKeys) {
    const match = providers.find((p) => p.enabled && p.apiKeyEnv === key);
    if (match) return match;
  }
  // Fallback: local Ollama
  return providers.find((p) => p.isLocal && p.enabled) ?? null;
}

export function getActiveModel(providers: Provider[]): { provider: Provider | null; model: string } {
  const activeId = get(activeProviderIdStore);
  const activeModel = get(activeModelStore);
  const provider = providers.find((p) => p.id === activeId) ?? null;
  const model = activeModel || (provider?.models[0] ?? '');
  return { provider, model };
}

// ─── Category Labels ──────────────────────────────────────────────────────────

export const CATEGORY_LABELS: Record<string, string> = {
  llm: '🤖 LLMs',
  image: '🎨 Image Generation',
  video: '🎬 Video Generation',
  audio: '🔊 Audio',
  search: '🔍 Search & Research',
  automation: '🤖 Automation & Agents',
  voice: '📞 Voice & Phone',
  mcp: '🧩 MCP Tools',
  database: '🗄️ Database & Storage',
  custom: '⚙️ Custom',
};

export const CATEGORY_ORDER = ['llm', 'image', 'video', 'audio', 'search', 'automation', 'voice', 'mcp', 'database', 'custom'];

// ─── Mock data for non-desktop mode ──────────────────────────────────────────

function getMockProviders(): Provider[] {
  return [
    {
      id: 'mock-openai',
      name: 'OpenAI',
      category: 'llm',
      baseUrl: 'https://api.openai.com/v1',
      authType: 'bearer',
      apiKeyEnv: 'OPENAI_API_KEY',
      models: ['gpt-4o', 'gpt-4o-mini'],
      enabled: false,
      isLocal: false,
      description: 'OpenAI GPT-4o models',
      docsUrl: 'https://platform.openai.com/docs',
      logoEmoji: '🟢',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    },
    {
      id: 'mock-ollama',
      name: 'Ollama (Local)',
      category: 'llm',
      baseUrl: 'http://localhost:11434/api',
      authType: 'none',
      apiKeyEnv: 'OLLAMA_HOST',
      models: ['llama3.3', 'mistral', 'qwen2.5'],
      enabled: true,
      isLocal: true,
      description: 'Local Ollama LLMs',
      docsUrl: 'https://ollama.ai',
      logoEmoji: '🦙',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    },
  ];
}

function mockLog(req: LogApiUsageRequest): ApiUsageLog {
  return {
    id: crypto.randomUUID(),
    providerId: req.providerId,
    providerName: req.providerName,
    model: req.model,
    tokensIn: req.tokensIn,
    tokensOut: req.tokensOut,
    costUsd: req.costUsd,
    outputSummary: req.outputSummary,
    toolId: req.toolId,
    workflowId: req.workflowId,
    createdAt: new Date().toISOString(),
  };
}

function mockMemoryStats(): MemorySpineStats {
  return {
    totalEntries: 0,
    totalTokens: 0,
    totalCostUsd: 0,
    providersUsed: [],
    recentLogs: [],
    activeTasks: [],
  };
}

function mockKaizenTask(req: CreateKaizenTaskRequest): KaizenTask {
  return {
    id: crypto.randomUUID(),
    title: req.title,
    description: req.description,
    status: 'todo',
    priority: req.priority as 'normal',
    source: req.source,
    providerId: req.providerId,
    usageLogId: req.usageLogId,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}
