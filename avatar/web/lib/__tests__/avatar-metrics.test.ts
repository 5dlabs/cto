import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  AVATAR_METRIC_TARGETS,
  createErrorRecoveryTracker,
  createFrameDropTracker,
  createMetricsRecorder,
  latencyMs,
  MetricsRecorder,
  nowMs,
  readMemoryUsageMb,
  startMemorySampler,
} from "../avatar-metrics";

describe("MetricsRecorder", () => {
  it("records and snapshots values", () => {
    const r = new MetricsRecorder();
    r.record("connection_latency_ms", 1234);
    r.record("audio_latency_ms", 567);
    expect(r.snapshot()).toEqual({
      connection_latency_ms: 1234,
      audio_latency_ms: 567,
    });
  });

  it("overwrites previous values (last write wins)", () => {
    const r = createMetricsRecorder();
    r.record("memory_usage_mb", 100);
    r.record("memory_usage_mb", 150);
    expect(r.get("memory_usage_mb")).toBe(150);
  });

  it("ignores undefined, null, NaN, and Infinity", () => {
    const r = new MetricsRecorder();
    r.record("frame_drop_rate", undefined);
    r.record("frame_drop_rate", null);
    r.record("frame_drop_rate", Number.NaN);
    r.record("frame_drop_rate", Number.POSITIVE_INFINITY);
    expect(r.snapshot()).toEqual({});
  });

  it("snapshot is a plain object independent of recorder state", () => {
    const r = new MetricsRecorder();
    r.record("viseme_sync_ms", 10);
    const snap = r.snapshot();
    r.record("viseme_sync_ms", 99);
    expect(snap.viseme_sync_ms).toBe(10);
  });

  it("reset clears values", () => {
    const r = new MetricsRecorder();
    r.record("memory_usage_mb", 42);
    r.reset();
    expect(r.snapshot()).toEqual({});
  });
});

describe("latencyMs", () => {
  it("returns non-negative delta for ordered timestamps", () => {
    expect(latencyMs(1000, 1500)).toBe(500);
  });

  it("clamps negatives to 0 (clock skew guard)", () => {
    expect(latencyMs(1500, 1000)).toBe(0);
  });

  it("returns null for missing or non-finite inputs", () => {
    expect(latencyMs(null, 1000)).toBeNull();
    expect(latencyMs(1000, null)).toBeNull();
    expect(latencyMs(undefined, 1000)).toBeNull();
    expect(latencyMs(Number.NaN, 1000)).toBeNull();
    expect(latencyMs(1000, Number.POSITIVE_INFINITY)).toBeNull();
  });
});

describe("nowMs", () => {
  it("returns a finite number", () => {
    const t = nowMs();
    expect(Number.isFinite(t)).toBe(true);
    expect(t).toBeGreaterThanOrEqual(0);
  });
});

describe("AVATAR_METRIC_TARGETS", () => {
  it("matches the protocol thresholds", () => {
    expect(AVATAR_METRIC_TARGETS.connection_latency_ms).toBe(2000);
    expect(AVATAR_METRIC_TARGETS.audio_latency_ms).toBe(1500);
    expect(AVATAR_METRIC_TARGETS.viseme_sync_ms).toBe(50);
    expect(AVATAR_METRIC_TARGETS.frame_drop_rate).toBe(0.01);
    expect(AVATAR_METRIC_TARGETS.error_recovery_ms).toBe(5000);
    expect(AVATAR_METRIC_TARGETS.memory_usage_mb).toBe(500);
  });
});

describe("createErrorRecoveryTracker", () => {
  it("returns null when recovering without an error mark", () => {
    const t = createErrorRecoveryTracker();
    expect(t.isPending).toBe(false);
    expect(t.markRecoveredAt(1000)).toBeNull();
  });

  it("returns delta between error and recovery and clears pending state", () => {
    const t = createErrorRecoveryTracker();
    t.markErrorAt(1000);
    expect(t.isPending).toBe(true);
    expect(t.markRecoveredAt(3500)).toBe(2500);
    expect(t.isPending).toBe(false);
    // second recovery without new error returns null
    expect(t.markRecoveredAt(4000)).toBeNull();
  });

  it("clamps backwards clocks to 0", () => {
    const t = createErrorRecoveryTracker();
    t.markErrorAt(1000);
    expect(t.markRecoveredAt(500)).toBe(0);
  });
});

describe("readMemoryUsageMb", () => {
  const origPerf = globalThis.performance;
  afterEach(() => {
    Object.defineProperty(globalThis, "performance", {
      value: origPerf,
      configurable: true,
      writable: true,
    });
  });

  it("returns null when performance.memory is unavailable", () => {
    Object.defineProperty(globalThis, "performance", {
      value: { now: () => 0 },
      configurable: true,
      writable: true,
    });
    expect(readMemoryUsageMb()).toBeNull();
  });

  it("converts usedJSHeapSize bytes to MB", () => {
    Object.defineProperty(globalThis, "performance", {
      value: { now: () => 0, memory: { usedJSHeapSize: 10 * 1024 * 1024 } },
      configurable: true,
      writable: true,
    });
    expect(readMemoryUsageMb()).toBeCloseTo(10, 5);
  });
});

describe("startMemorySampler", () => {
  const origPerf = globalThis.performance;
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    Object.defineProperty(globalThis, "performance", {
      value: origPerf,
      configurable: true,
      writable: true,
    });
  });

  it("returns a no-op stop when performance.memory is missing", () => {
    Object.defineProperty(globalThis, "performance", {
      value: { now: () => 0 },
      configurable: true,
      writable: true,
    });
    const samples: number[] = [];
    const stop = startMemorySampler({ onSample: (mb) => samples.push(mb) });
    stop();
    expect(samples).toEqual([]);
  });

  it("emits initial + interval samples when API is present", () => {
    let used = 5 * 1024 * 1024;
    Object.defineProperty(globalThis, "performance", {
      value: {
        now: () => 0,
        get memory() {
          return { usedJSHeapSize: used };
        },
      },
      configurable: true,
      writable: true,
    });
    const samples: number[] = [];
    const stop = startMemorySampler({
      intervalMs: 1000,
      onSample: (mb) => samples.push(mb),
    });
    expect(samples).toEqual([5]);
    used = 7 * 1024 * 1024;
    vi.advanceTimersByTime(1000);
    expect(samples).toEqual([5, 7]);
    stop();
    used = 9 * 1024 * 1024;
    vi.advanceTimersByTime(5000);
    expect(samples).toEqual([5, 7]);
  });
});

describe("createFrameDropTracker", () => {
  let rafCallbacks: Array<(t: number) => void> = [];
  let nextHandle = 1;

  beforeEach(() => {
    rafCallbacks = [];
    nextHandle = 1;
    vi.stubGlobal("requestAnimationFrame", (cb: (t: number) => void) => {
      rafCallbacks.push(cb);
      return nextHandle++;
    });
    vi.stubGlobal("cancelAnimationFrame", () => {
      rafCallbacks = [];
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("reports 0 drop rate when frames arrive at target cadence", () => {
    const tracker = createFrameDropTracker({ targetFps: 60, windowMs: 1000 });
    // Drive 60 frames over 1s — expected ≈ observed
    let t = 0;
    for (let i = 0; i < 60; i += 1) {
      t += 1000 / 60;
      const cb = rafCallbacks.shift();
      cb?.(t);
    }
    expect(tracker.rate).toBeLessThan(0.02);
    tracker.stop();
  });

  it("reports elevated drop rate when far below target cadence", () => {
    const tracker = createFrameDropTracker({ targetFps: 60, windowMs: 1000 });
    // Drive 10 frames over 1s — heavy drop
    let t = 0;
    for (let i = 0; i < 10; i += 1) {
      t += 100;
      const cb = rafCallbacks.shift();
      cb?.(t);
    }
    expect(tracker.rate).toBeGreaterThan(0.5);
    tracker.stop();
  });
});
