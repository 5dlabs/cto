/**
 * Run Registry — Linear Bridge
 *
 * In-memory map tracking active pipeline runs.
 * Maps runId → {agentPod, sessionKey, issueId, linearSessionId, resumeToken}.
 * Used to correlate Linear webhook callbacks back to the correct Lobster workflow.
 */

export interface RunEntry {
  agentPod: string;
  sessionKey: string;
  issueId: string;
  linearSessionId?: string;
  resumeToken?: string;
  registeredAt: number;
  lastAccessedAt: number;
}

export interface RunRegistry {
  register(runId: string, data: Omit<RunEntry, 'registeredAt' | 'lastAccessedAt'>): void;
  deregister(runId: string): void;
  lookup(runId: string): RunEntry | undefined;
  /** Update a run entry's fields (merge) */
  update(runId: string, data: Partial<Pick<RunEntry, 'linearSessionId' | 'resumeToken'>>): void;
  /** Remove runs older than maxAgeMs */
  gc(maxAgeMs: number): number;
  size(): number;
}

export function createRunRegistry(): RunRegistry {
  const runs = new Map<string, RunEntry>();

  return {
    register(runId, data) {
      runs.set(runId, {
        ...data,
        registeredAt: Date.now(),
        lastAccessedAt: Date.now(),
      });
    },

    deregister(runId) {
      runs.delete(runId);
    },

    lookup(runId) {
      const entry = runs.get(runId);
      if (entry) entry.lastAccessedAt = Date.now();
      return entry;
    },

    update(runId, data) {
      const entry = runs.get(runId);
      if (!entry) return;
      if (data.linearSessionId !== undefined) entry.linearSessionId = data.linearSessionId;
      if (data.resumeToken !== undefined) entry.resumeToken = data.resumeToken;
      entry.lastAccessedAt = Date.now();
    },

    gc(maxAgeMs) {
      const cutoff = Date.now() - maxAgeMs;
      let removed = 0;
      for (const [key, entry] of runs) {
        if (entry.lastAccessedAt < cutoff) {
          runs.delete(key);
          removed++;
        }
      }
      return removed;
    },

    size() {
      return runs.size;
    },
  };
}
