import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  AVATAR_SESSION_PROTOCOL,
  type ErrorFrame,
} from "../avatar-state";
import { createAvatarErrorHandler } from "../avatar-error-handler";

const recoverable: ErrorFrame = {
  protocol: AVATAR_SESSION_PROTOCOL,
  type: "ERROR",
  session_id: "sess-1",
  code: "NETWORK_DISCONNECT",
  message: "socket closed",
  recoverable: true,
  timestamp_ms: 1_700_000_000_000,
};

const fatal: ErrorFrame = {
  protocol: AVATAR_SESSION_PROTOCOL,
  type: "ERROR",
  session_id: "sess-1",
  code: "AUTH_FAILED",
  message: "bad token",
  recoverable: false,
  timestamp_ms: 1_700_000_000_000,
};

describe("avatar-error-handler", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("schedules backoff and invokes onReconnect on recoverable error", () => {
    const onReconnect = vi.fn();
    const handler = createAvatarErrorHandler({ onReconnect });

    handler.ingest(recoverable);
    const state = handler.getState();
    expect(state.phase).toBe("recovering");
    if (state.phase === "recovering") {
      expect(state.attempt).toBe(1);
      expect(state.lastError.code).toBe("NETWORK_DISCONNECT");
    }
    expect(onReconnect).not.toHaveBeenCalled();

    vi.advanceTimersByTime(499);
    expect(onReconnect).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(onReconnect).toHaveBeenCalledTimes(1);
  });

  it("goes fatal immediately on non-recoverable error (no retry scheduled)", () => {
    const onReconnect = vi.fn();
    const handler = createAvatarErrorHandler({ onReconnect });

    handler.ingest(fatal);
    expect(handler.getState().phase).toBe("fatal");

    vi.advanceTimersByTime(10_000);
    expect(onReconnect).not.toHaveBeenCalled();
  });

  it("advances attempt on repeated recoverable errors and escalates to fatal after exhausting schedule", () => {
    const onReconnect = vi.fn();
    const handler = createAvatarErrorHandler({ onReconnect });

    // Attempt 1 (500ms)
    handler.ingest(recoverable);
    vi.advanceTimersByTime(500);
    // Attempt 2 (1500ms)
    handler.ingest(recoverable);
    let s = handler.getState();
    expect(s.phase).toBe("recovering");
    if (s.phase === "recovering") expect(s.attempt).toBe(2);
    vi.advanceTimersByTime(1500);
    // Attempt 3 (4500ms)
    handler.ingest(recoverable);
    s = handler.getState();
    if (s.phase === "recovering") expect(s.attempt).toBe(3);
    vi.advanceTimersByTime(4500);
    expect(onReconnect).toHaveBeenCalledTimes(3);

    // 4th recoverable error exhausts the schedule -> fatal.
    handler.ingest(recoverable);
    expect(handler.getState().phase).toBe("fatal");
  });

  it("clears state to healthy after observing a non-ERROR frame post-recovery", () => {
    const onReconnect = vi.fn();
    const handler = createAvatarErrorHandler({ onReconnect });

    handler.ingest(recoverable);
    vi.advanceTimersByTime(500);
    expect(handler.getState().phase).toBe("recovering");

    handler.ingest({
      protocol: AVATAR_SESSION_PROTOCOL,
      type: "SESSION_STATE",
      session_id: "sess-1",
      state: "listening",
      agent_name: "rex",
      timestamp_ms: 1_700_000_000_001,
    });

    expect(handler.getState().phase).toBe("healthy");
  });

  it("ignores malformed frames in any phase", () => {
    const onReconnect = vi.fn();
    const handler = createAvatarErrorHandler({ onReconnect });

    handler.ingest(null);
    handler.ingest(undefined);
    handler.ingest({ type: "ERROR" });
    handler.ingest({ protocol: "other/v1", type: "ERROR" });
    handler.ingest("not-an-object");

    expect(handler.getState().phase).toBe("healthy");
    expect(onReconnect).not.toHaveBeenCalled();
  });

  it("notifyRecovered cancels pending reconnect", () => {
    const onReconnect = vi.fn();
    const handler = createAvatarErrorHandler({ onReconnect });

    handler.ingest(recoverable);
    expect(handler.getState().phase).toBe("recovering");
    handler.notifyRecovered();
    expect(handler.getState().phase).toBe("healthy");

    vi.advanceTimersByTime(10_000);
    expect(onReconnect).not.toHaveBeenCalled();
  });
});
