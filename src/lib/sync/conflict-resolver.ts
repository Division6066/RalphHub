/**
 * CRDT-inspired conflict resolution.
 * Uses vector clocks to detect concurrent edits; falls back to
 * last-write-wins (LWW) by createdAt timestamp for convergence.
 */
import type { SyncEvent, ConflictResolution } from './types.js';

type VectorClock = Record<string, number>;

function compareClocks(a: VectorClock, b: VectorClock): 'a>b' | 'b>a' | 'concurrent' {
  const allKeys = new Set([...Object.keys(a), ...Object.keys(b)]);
  let aGtB = false;
  let bGtA = false;

  for (const k of allKeys) {
    const av = a[k] ?? 0;
    const bv = b[k] ?? 0;
    if (av > bv) aGtB = true;
    if (bv > av) bGtA = true;
  }

  if (aGtB && !bGtA) return 'a>b';
  if (bGtA && !aGtB) return 'b>a';
  return 'concurrent';
}

export function resolve(local: SyncEvent, remote: SyncEvent): ConflictResolution {
  const order = compareClocks(local.vectorClock, remote.vectorClock);

  if (order === 'a>b') {
    return {
      winnerId: local.id,
      loserId: remote.id,
      strategy: 'last-write-wins',
      resolvedAt: new Date().toISOString(),
    };
  }

  if (order === 'b>a') {
    return {
      winnerId: remote.id,
      loserId: local.id,
      strategy: 'last-write-wins',
      resolvedAt: new Date().toISOString(),
    };
  }

  // Concurrent — fall back to wall-clock LWW (server/desktop wins for safety)
  const localTs = new Date(local.createdAt).getTime();
  const remoteTs = new Date(remote.createdAt).getTime();
  const winner = remoteTs >= localTs ? remote : local;
  const loser = winner === remote ? local : remote;

  return {
    winnerId: winner.id,
    loserId: loser.id,
    strategy: 'server-wins',
    resolvedAt: new Date().toISOString(),
  };
}

export function mergeVectorClocks(a: VectorClock, b: VectorClock): VectorClock {
  const merged: VectorClock = { ...a };
  for (const [k, v] of Object.entries(b)) {
    merged[k] = Math.max(merged[k] ?? 0, v);
  }
  return merged;
}

export function incrementClock(clock: VectorClock, deviceId: string): VectorClock {
  return { ...clock, [deviceId]: (clock[deviceId] ?? 0) + 1 };
}
