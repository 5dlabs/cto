/**
 * Agent Session Manager
 *
 * In-memory map tracking active Linear agent sessions.
 * Maps deliberation session IDs to Linear agent session IDs,
 * enabling correlation of webhook events back to pending elicitations.
 */

export interface SessionEntry {
  /** Linear agent session ID */
  linearSessionId: string;
  /** Deliberation session ID */
  deliberationSessionId: string;
  /** Linear issue ID this session is attached to */
  issueId: string;
  /** When this entry was created */
  createdAt: number;
  /** When this entry was last accessed */
  lastAccessedAt: number;
}

export interface AgentSessionManager {
  /** Register a mapping between a deliberation session and a Linear agent session */
  register(deliberationSessionId: string, linearSessionId: string, issueId: string): void;
  /** Find session entry by deliberation session ID */
  findByDeliberation(deliberationSessionId: string): SessionEntry | undefined;
  /** Find session entry by Linear agent session ID (for webhook correlation) */
  findByLinearSession(linearSessionId: string): SessionEntry | undefined;
  /** Remove a session entry */
  remove(deliberationSessionId: string): void;
  /** Garbage-collect sessions older than maxAgeMs */
  gc(maxAgeMs: number): number;
}

export function createAgentSessionManager(): AgentSessionManager {
  const byDeliberation = new Map<string, SessionEntry>();
  const byLinearSession = new Map<string, SessionEntry>();

  return {
    register(deliberationSessionId, linearSessionId, issueId) {
      // Remove old entry if exists
      const existing = byDeliberation.get(deliberationSessionId);
      if (existing) {
        byLinearSession.delete(existing.linearSessionId);
      }

      const entry: SessionEntry = {
        linearSessionId,
        deliberationSessionId,
        issueId,
        createdAt: Date.now(),
        lastAccessedAt: Date.now(),
      };

      byDeliberation.set(deliberationSessionId, entry);
      byLinearSession.set(linearSessionId, entry);
    },

    findByDeliberation(deliberationSessionId) {
      const entry = byDeliberation.get(deliberationSessionId);
      if (entry) entry.lastAccessedAt = Date.now();
      return entry;
    },

    findByLinearSession(linearSessionId) {
      const entry = byLinearSession.get(linearSessionId);
      if (entry) entry.lastAccessedAt = Date.now();
      return entry;
    },

    remove(deliberationSessionId) {
      const entry = byDeliberation.get(deliberationSessionId);
      if (entry) {
        byDeliberation.delete(deliberationSessionId);
        byLinearSession.delete(entry.linearSessionId);
      }
    },

    gc(maxAgeMs) {
      const cutoff = Date.now() - maxAgeMs;
      let removed = 0;

      for (const [key, entry] of byDeliberation) {
        if (entry.lastAccessedAt < cutoff) {
          byDeliberation.delete(key);
          byLinearSession.delete(entry.linearSessionId);
          removed++;
        }
      }

      return removed;
    },
  };
}
