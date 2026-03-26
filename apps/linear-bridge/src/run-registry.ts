import type { BridgeStateDb } from "./state/bridge-state-db.js";

/**
 * Run Registry — Linear Bridge
 *
 * SQLite-backed run map tracking active pipeline runs.
 * Maps runId → {agentPod, sessionKey, issueId, linearSessionId, resumeToken}.
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

export interface ActiveRunView {
  runId: string;
  agentPod: string;
  sessionKey: string;
  issueId: string;
  linearSessionId?: string;
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
  /** Return a snapshot of all currently tracked runs */
  getActiveRuns(): ActiveRunView[];
}

export function createRunRegistry(stateDb: BridgeStateDb): RunRegistry {
  return {
    register(runId, data) {
      stateDb.upsertRun({
        runId,
        agentPod: data.agentPod,
        sessionKey: data.sessionKey,
        issueId: data.issueId,
        linearSessionId: data.linearSessionId,
        resumeToken: data.resumeToken,
      });
    },

    deregister(runId) {
      stateDb.deleteRun(runId);
    },

    lookup(runId) {
      const row = stateDb.getRun(runId);
      if (!row) return undefined;
      stateDb.upsertRun({
        runId: row.runId,
        agentPod: row.agentPod,
        sessionKey: row.sessionKey,
        issueId: row.issueId,
        linearSessionId: row.linearSessionId,
        resumeToken: row.resumeToken,
      });
      return {
        agentPod: row.agentPod,
        sessionKey: row.sessionKey,
        issueId: row.issueId,
        linearSessionId: row.linearSessionId,
        resumeToken: row.resumeToken,
        registeredAt: row.registeredAt,
        lastAccessedAt: row.lastAccessedAt,
      };
    },

    update(runId, data) {
      stateDb.updateRun(runId, {
        linearSessionId: data.linearSessionId,
        resumeToken: data.resumeToken,
      });
    },

    gc(maxAgeMs) {
      return stateDb.gcRuns(maxAgeMs);
    },

    size() {
      return stateDb.countRuns();
    },

    getActiveRuns() {
      return stateDb.listRuns();
    },
  };
}
