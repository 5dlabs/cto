/**
 * Avatar protocol metrics recorder.
 *
 * Implements the six key metrics from
 * `docs/specs/avatar-session-protocol.md` §"Metrics and Failure Handling":
 *
 *   1. connection_latency_ms  — START_SESSION → SESSION_READY        (<2000ms)
 *   2. audio_latency_ms       — SESSION_READY → audio track playing  (<1500ms)
 *   3. viseme_sync_ms         — |audio_cursor − viseme_cursor|       (<50ms)
 *   4. frame_drop_rate        — dropped / target frames (rolling)    (<0.01)
 *   5. error_recovery_ms      — ERROR frame → recovered state        (<5000ms)
 *   6. memory_usage_mb        — Chromium performance.memory sample   (<500MB)
 *
 * Bridge-observable metrics (1, 2, 5) are also emitted by the voice-bridge
 * as structured JSON log events; see
 * `infra/images/voice-bridge/app/protocol_metrics.py`.
 */

export type AvatarMetricName =
  | "connection_latency_ms"
  | "audio_latency_ms"
  | "viseme_sync_ms"
  | "frame_drop_rate"
  | "error_recovery_ms"
  | "memory_usage_mb";

export type AvatarMetrics = Partial<Record<AvatarMetricName, number>>;

export const AVATAR_METRIC_TARGETS: Record<AvatarMetricName, number> = {
  connection_latency_ms: 2000,
  audio_latency_ms: 1500,
  viseme_sync_ms: 50,
  frame_drop_rate: 0.01,
  error_recovery_ms: 5000,
  memory_usage_mb: 500,
};

/** Monotonic clock — `performance.now()` if available, else `Date.now()`. */
export function nowMs(): number {
  if (typeof performance !== "undefined" && typeof performance.now === "function") {
    return performance.now();
  }
  return Date.now();
}

/** Non-negative latency in ms between two timestamps; `null` if either is missing. */
export function latencyMs(
  startMs: number | null | undefined,
  endMs: number | null | undefined,
): number | null {
  if (typeof startMs !== "number" || typeof endMs !== "number") {
    return null;
  }
  if (!Number.isFinite(startMs) || !Number.isFinite(endMs)) {
    return null;
  }
  return Math.max(0, endMs - startMs);
}

/** In-memory recorder; most recent value per metric wins. */
export class MetricsRecorder {
  private readonly values: Map<AvatarMetricName, number> = new Map();

  record(name: AvatarMetricName, value: number | null | undefined): void {
    if (typeof value !== "number" || !Number.isFinite(value)) {
      return;
    }
    this.values.set(name, value);
  }

  get(name: AvatarMetricName): number | undefined {
    return this.values.get(name);
  }

  /** Immutable snapshot suitable for merging into `AvatarStatePayload.metrics`. */
  snapshot(): AvatarMetrics {
    const out: AvatarMetrics = {};
    for (const [k, v] of this.values.entries()) {
      out[k] = v;
    }
    return out;
  }

  reset(): void {
    this.values.clear();
  }
}

export function createMetricsRecorder(): MetricsRecorder {
  return new MetricsRecorder();
}

// --- Frame drop tracking ----------------------------------------------------

export interface FrameDropTracker {
  /** Current dropped/target ratio (0..1) over the rolling window. */
  readonly rate: number;
  stop(): void;
}

export interface FrameDropTrackerOptions {
  targetFps?: number;
  windowMs?: number;
  /**
   * Called on every measurement update. Wired to a `MetricsRecorder` by
   * consumers that want the value pushed into snapshots automatically.
   */
  onSample?: (rate: number) => void;
}

/**
 * Measure the fraction of frames dropped vs a target fps over a rolling
 * window. Uses `requestAnimationFrame`; falls back to a no-op tracker when
 * not running in a browser.
 */
export function createFrameDropTracker(
  options: FrameDropTrackerOptions = {},
): FrameDropTracker {
  const targetFps = options.targetFps ?? 60;
  const windowMs = options.windowMs ?? 5000;

  const canRun =
    typeof requestAnimationFrame === "function" &&
    typeof cancelAnimationFrame === "function";

  if (!canRun) {
    return { rate: 0, stop: () => undefined };
  }

  const frameTimes: number[] = [];
  let rafHandle: number | null = null;
  let rate = 0;

  const tick = (t: number) => {
    frameTimes.push(t);
    const cutoff = t - windowMs;
    while (frameTimes.length > 0 && frameTimes[0] < cutoff) {
      frameTimes.shift();
    }
    const expected = (windowMs / 1000) * targetFps;
    const observed = frameTimes.length;
    const dropped = Math.max(0, expected - observed);
    rate = expected > 0 ? Math.min(1, dropped / expected) : 0;
    options.onSample?.(rate);
    rafHandle = requestAnimationFrame(tick);
  };

  rafHandle = requestAnimationFrame(tick);

  return {
    get rate() {
      return rate;
    },
    stop() {
      if (rafHandle !== null) {
        cancelAnimationFrame(rafHandle);
        rafHandle = null;
      }
    },
  };
}

// --- Memory sampling (Chromium `performance.memory`) ------------------------

type MemoryLike = { usedJSHeapSize?: number };

/** Read current JS heap usage in MB. Returns `null` outside Chromium. */
export function readMemoryUsageMb(): number | null {
  if (typeof performance === "undefined") {
    return null;
  }
  const mem = (performance as unknown as { memory?: MemoryLike }).memory;
  const used = mem?.usedJSHeapSize;
  if (typeof used !== "number" || !Number.isFinite(used)) {
    return null;
  }
  return used / (1024 * 1024);
}

export interface MemorySamplerOptions {
  intervalMs?: number;
  onSample?: (mb: number) => void;
}

/**
 * Periodically poll `performance.memory.usedJSHeapSize` and invoke
 * `onSample`. Returns a `stop()` handle. If the API is unavailable, the
 * returned stop is a no-op and no sampling occurs.
 */
export function startMemorySampler(
  options: MemorySamplerOptions = {},
): () => void {
  const initial = readMemoryUsageMb();
  if (initial === null) {
    return () => undefined;
  }
  const interval = options.intervalMs ?? 5000;
  options.onSample?.(initial);

  if (typeof setInterval !== "function") {
    return () => undefined;
  }

  const handle = setInterval(() => {
    const mb = readMemoryUsageMb();
    if (mb !== null) {
      options.onSample?.(mb);
    }
  }, interval);

  return () => clearInterval(handle);
}

// --- Error-recovery tracking -----------------------------------------------

export interface ErrorRecoveryTracker {
  /** Call when an ERROR frame is observed (or any error state begins). */
  markErrorAt(tsMs?: number): void;
  /** Call when the session has recovered (e.g. back to `listening`). */
  markRecoveredAt(tsMs?: number): number | null;
  /** Whether an error window is currently open. */
  readonly isPending: boolean;
}

export function createErrorRecoveryTracker(): ErrorRecoveryTracker {
  let errorAt: number | null = null;

  return {
    get isPending() {
      return errorAt !== null;
    },
    markErrorAt(tsMs?: number) {
      errorAt = typeof tsMs === "number" ? tsMs : nowMs();
    },
    markRecoveredAt(tsMs?: number) {
      if (errorAt === null) {
        return null;
      }
      const end = typeof tsMs === "number" ? tsMs : nowMs();
      const delta = Math.max(0, end - errorAt);
      errorAt = null;
      return delta;
    },
  };
}
