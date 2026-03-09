import { invoke } from '@tauri-apps/api/core';

export interface MobileServerInfo {
  running: boolean;
  port: number | null;
  localIp: string | null;
  qrUrl: string | null;
}

export async function getMobileServerInfo(): Promise<MobileServerInfo> {
  try {
    return await invoke<MobileServerInfo>('get_mobile_server_info');
  } catch {
    // Running in browser/dev mode — return mock info
    return {
      running: false,
      port: 7842,
      localIp: null,
      qrUrl: null,
    };
  }
}

export async function writeMobileMemoryEvent(
  source: string,
  eventType: string,
  payload: unknown,
  kaizenHint?: string
): Promise<string | null> {
  try {
    return await invoke<string>('write_to_memory', {
      source,
      eventType,
      payload,
      kaizenHint,
    });
  } catch (e) {
    console.error('[writeMobileMemoryEvent]', e);
    return null;
  }
}

export async function readMemoryEvents(opts: {
  since?: string;
  source?: string;
  limit?: number;
} = {}): Promise<unknown[]> {
  try {
    return await invoke<unknown[]>('read_memory', opts);
  } catch {
    return [];
  }
}

export async function listKaizenTasks(status?: string): Promise<unknown[]> {
  try {
    return await invoke<unknown[]>('list_kaizen_tasks', { status });
  } catch {
    return [];
  }
}
