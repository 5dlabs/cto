import { describe, expect, it, vi } from "vitest";

import { AVATAR_SESSION_PROTOCOL } from "../avatar-state";
import { createAvatarSessionSubscriber } from "../avatar-session-subscriber";

function validFrame(overrides: Record<string, unknown> = {}) {
  return {
    protocol: AVATAR_SESSION_PROTOCOL,
    type: "SESSION_STATE" as const,
    state: "listening" as const,
    session_id: "s1",
    agent_name: "morgan",
    timestamp_ms: 1700000000000,
    ...overrides,
  };
}

describe("createAvatarSessionSubscriber", () => {
  it("dispatches valid frames and tracks latest state in order", () => {
    const sub = createAvatarSessionSubscriber();
    const listener = vi.fn();
    sub.subscribe(listener);

    sub.ingest(validFrame({ state: "connecting" }));
    sub.ingest(validFrame({ state: "connected" }));
    sub.ingest(validFrame({ state: "listening" }));

    expect(listener).toHaveBeenCalledTimes(3);
    expect(listener.mock.calls.map((c) => c[0].state)).toEqual([
      "connecting",
      "connected",
      "listening",
    ]);
    expect(sub.getLatest()).toBe("listening");
  });

  it("parses JSON string payloads and ignores malformed JSON", () => {
    const sub = createAvatarSessionSubscriber();
    const listener = vi.fn();
    sub.subscribe(listener);

    sub.ingest(JSON.stringify(validFrame({ state: "speaking" })));
    sub.ingest("{not json");
    sub.ingest("null");

    expect(listener).toHaveBeenCalledTimes(1);
    expect(sub.getLatest()).toBe("speaking");
  });

  it("ignores wrong-protocol, wrong-shape, and invalid-state frames", () => {
    const sub = createAvatarSessionSubscriber();
    const listener = vi.fn();
    sub.subscribe(listener);

    sub.ingest(validFrame({ protocol: "other/v1" }));
    sub.ingest(validFrame({ type: "OTHER" }));
    sub.ingest(validFrame({ state: "bogus" }));
    sub.ingest(validFrame({ session_id: 42 }));
    sub.ingest(42);
    sub.ingest(null);

    expect(listener).not.toHaveBeenCalled();
    expect(sub.getLatest()).toBeNull();
  });

  it("ignores ERROR frames (sibling workstream owns them)", () => {
    const sub = createAvatarSessionSubscriber();
    const listener = vi.fn();
    sub.subscribe(listener);

    sub.ingest({
      protocol: AVATAR_SESSION_PROTOCOL,
      type: "ERROR",
      session_id: "s1",
      code: "boom",
      message: "m",
      recoverable: false,
      timestamp_ms: 1,
    });

    expect(listener).not.toHaveBeenCalled();
    expect(sub.getLatest()).toBeNull();
  });

  it("attach/detach wires MessageEvent dispatch and cleanup", () => {
    const sub = createAvatarSessionSubscriber();
    const listener = vi.fn();
    sub.subscribe(listener);

    const target = new EventTarget();
    sub.attach(target);

    const ev = new MessageEvent("message", {
      data: JSON.stringify(validFrame({ state: "connecting" })),
    });
    target.dispatchEvent(ev);
    expect(listener).toHaveBeenCalledTimes(1);

    sub.detach();
    target.dispatchEvent(
      new MessageEvent("message", {
        data: JSON.stringify(validFrame({ state: "connected" })),
      }),
    );
    expect(listener).toHaveBeenCalledTimes(1);
  });

  it("re-attach replaces prior listener (no double-dispatch)", () => {
    const sub = createAvatarSessionSubscriber();
    const listener = vi.fn();
    sub.subscribe(listener);

    const t1 = new EventTarget();
    const t2 = new EventTarget();

    sub.attach(t1);
    sub.attach(t2);

    t1.dispatchEvent(
      new MessageEvent("message", {
        data: JSON.stringify(validFrame({ state: "connecting" })),
      }),
    );
    expect(listener).not.toHaveBeenCalled();

    t2.dispatchEvent(
      new MessageEvent("message", {
        data: JSON.stringify(validFrame({ state: "connected" })),
      }),
    );
    expect(listener).toHaveBeenCalledTimes(1);
  });
});
