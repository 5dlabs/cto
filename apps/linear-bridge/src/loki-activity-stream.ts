/**
 * LokiActivityStream — Linear Bridge
 *
 * Polls Loki for ACP session logs, parses activity type from each log line,
 * and calls linearClient.createAgentActivity() to surface agent dialog in Linear.
 *
 * Correlation: agent_id label from Loki → matched against run-registry agentPod
 * to find the linearSessionId for the active Linear agent session.
 */

import type { LinearClient, ActivityContent } from "./linear-client.js";
import type { RunRegistry } from "./run-registry.js";

interface Logger {
  info: (...args: unknown[]) => void;
  warn: (...args: unknown[]) => void;
  error: (...args: unknown[]) => void;
}

export interface LokiActivityStreamConfig {
  lokiUrl: string;
  lokiOrgId: string;
  pollIntervalMs: number;
}

// Loki query_range API response shape
interface LokiStream {
  stream: Record<string, string>;
  values: [string, string][]; // [nanosecondTimestamp, logLine]
}

interface LokiQueryRangeResponse {
  status: string;
  data: {
    resultType: string;
    result: LokiStream[];
  };
}

// Cursor per stream key to avoid re-posting the same log lines
type StreamCursors = Map<string, bigint>; // key → last seen nanosecond timestamp

/**
 * Parse a raw log line into an ActivityContent.
 * Returns null if the log line should be skipped (e.g. elicitation type).
 */
export function parseLogLine(raw: string): ActivityContent | null {
  let parsed: Record<string, unknown>;

  try {
    parsed = JSON.parse(raw) as Record<string, unknown>;
  } catch {
    // Plain text — treat as thought
    const body = raw.trim();
    if (!body) return null;
    return { type: "thought", body };
  }

  const type = typeof parsed.type === "string" ? parsed.type : undefined;

  if (type === "elicitation") {
    // Handled by elicitation-handler.ts — skip
    return null;
  }

  if (type === "thought") {
    return { type: "thought", body: String(parsed.body ?? raw) };
  }

  if (type === "action") {
    return {
      type: "action",
      action: String(parsed.action ?? ""),
      parameter: String(parsed.parameter ?? ""),
      ...(parsed.result !== undefined ? { result: String(parsed.result) } : {}),
    };
  }

  if (type === "response") {
    return { type: "response", body: String(parsed.body ?? raw) };
  }

  if (type === "error") {
    return { type: "error", body: String(parsed.body ?? parsed.message ?? raw) };
  }

  // Unknown JSON structure — wrap as thought if there's something useful
  const body = String(parsed.message ?? parsed.msg ?? parsed.body ?? raw).trim();
  if (!body) return null;
  return { type: "thought", body };
}

/**
 * Find the linearSessionId for a given agentId by scanning active runs.
 */
function resolveLinearSessionId(
  agentId: string,
  runRegistry: RunRegistry,
): string | undefined {
  const runs = runRegistry.getActiveRuns();
  // agentId from Loki matches agentPod in run registry
  const match = runs.find((r) => r.agentPod === agentId && r.linearSessionId);
  return match?.linearSessionId;
}

export class LokiActivityStream {
  private readonly cursors: StreamCursors = new Map();
  private timer: ReturnType<typeof setInterval> | undefined;
  private readonly LOKI_QUERY = `{agent_id=~".+", source=~"acp-.*"} | json`;

  constructor(
    private readonly config: LokiActivityStreamConfig,
    private readonly linearClient: LinearClient,
    private readonly runRegistry: RunRegistry,
    private readonly logger: Logger,
  ) {}

  start(): void {
    if (this.timer) return;
    this.logger.info(
      `[loki-activity] Starting poll loop (interval=${this.config.pollIntervalMs}ms, url=${this.config.lokiUrl})`,
    );
    this.timer = setInterval(() => {
      this.poll().catch((err) => {
        this.logger.warn("[loki-activity] Poll error:", err);
      });
    }, this.config.pollIntervalMs);
    // Run first poll immediately
    this.poll().catch((err) => {
      this.logger.warn("[loki-activity] Initial poll error:", err);
    });
  }

  stop(): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = undefined;
      this.logger.info("[loki-activity] Stopped poll loop");
    }
  }

  /** Exposed for testing */
  async poll(): Promise<void> {
    const now = BigInt(Date.now()) * 1_000_000n; // ns
    // Default start: 5 seconds ago (covers first poll with no cursor)
    const defaultStart = now - 5_000_000_000n;

    // Determine the earliest cursor we need to query from
    let queryStart = defaultStart;
    if (this.cursors.size > 0) {
      // Use the smallest cursor so we catch all streams
      const minCursor = [...this.cursors.values()].reduce(
        (min, v) => (v < min ? v : min),
        now,
      );
      // Add 1ns to exclude the last seen entry
      queryStart = minCursor + 1n;
    }

    const url = new URL(`${this.config.lokiUrl}/loki/api/v1/query_range`);
    url.searchParams.set("query", this.LOKI_QUERY);
    url.searchParams.set("start", queryStart.toString());
    url.searchParams.set("end", now.toString());
    url.searchParams.set("limit", "100");
    url.searchParams.set("direction", "forward");

    let response: Response;
    try {
      response = await fetch(url.toString(), {
        headers: {
          "X-Scope-OrgID": this.config.lokiOrgId,
          Accept: "application/json",
        },
      });
    } catch (err) {
      this.logger.warn("[loki-activity] Fetch failed:", err);
      return;
    }

    if (!response.ok) {
      this.logger.warn(`[loki-activity] Loki returned ${response.status}`);
      return;
    }

    const body = (await response.json()) as LokiQueryRangeResponse;
    if (body.status !== "success" || !body.data?.result?.length) return;

    for (const stream of body.data.result) {
      const agentId = stream.stream["agent_id"];
      const source = stream.stream["source"] ?? "unknown";

      if (!agentId) continue;

      const streamKey = `${agentId}:${source}`;
      const cursor = this.cursors.get(streamKey) ?? 0n;

      const linearSessionId = resolveLinearSessionId(agentId, this.runRegistry);
      if (!linearSessionId) {
        // No active Linear session for this agent — advance cursor to avoid backlog
        for (const [tsStr] of stream.values) {
          const ts = BigInt(tsStr);
          if (ts > cursor) this.cursors.set(streamKey, ts);
        }
        continue;
      }

      for (const [tsStr, rawLine] of stream.values) {
        const ts = BigInt(tsStr);
        if (ts <= cursor) continue; // already processed

        const content = parseLogLine(rawLine);
        if (content) {
          try {
            await this.linearClient.createAgentActivity({
              agentSessionId: linearSessionId,
              content,
            });
          } catch (err) {
            this.logger.error(
              `[loki-activity] Failed to post activity for session ${linearSessionId}:`,
              err,
            );
          }
        }

        // Always advance cursor even if we skip the line
        this.cursors.set(streamKey, ts);
      }
    }
  }

  /** Exposed for testing — reset cursor state */
  resetCursors(): void {
    this.cursors.clear();
  }

  /** Exposed for testing — inspect cursors */
  getCursor(agentId: string, source: string): bigint | undefined {
    return this.cursors.get(`${agentId}:${source}`);
  }
}
