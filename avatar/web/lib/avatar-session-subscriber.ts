// Subscriber for cto-avatar-session/v1 SESSION_STATE frames.
//
// Parses incoming WebSocket messages, drops anything that is not a valid
// SESSION_STATE frame (malformed JSON, wrong protocol, ERROR frames, or
// frames with unknown state enum values), and fans valid frames out to
// registered subscribers. ERROR frames are deliberately ignored here — the
// ERROR-frame workstream owns that lane.
//
// No WebSocket consumer wires this in today; the subscriber is forward
// looking so a future client can do `subscriber.attach(ws)` without
// spreading parsing logic through the UI.

import type { AvatarSessionState, SessionStateFrame } from "./avatar-state";
import { isSessionStateFrame } from "./avatar-state";

export type AvatarSessionSubscriberListener = (frame: SessionStateFrame) => void;

export type AvatarSessionSubscriber = {
  /** Attach to a WebSocket (or any EventTarget with `message` events). */
  attach(target: EventTarget): void;
  /** Detach from the currently-attached target. No-op if not attached. */
  detach(): void;
  /** Parse a raw payload (`MessageEvent.data` or pre-parsed value). */
  ingest(data: unknown): void;
  /** Most recent valid state, or `null` before any valid frame. */
  getLatest(): AvatarSessionState | null;
  /** Subscribe to valid frames. Returns an unsubscribe callback. */
  subscribe(listener: AvatarSessionSubscriberListener): () => void;
};

export function createAvatarSessionSubscriber(): AvatarSessionSubscriber {
  const listeners = new Set<AvatarSessionSubscriberListener>();
  let latest: AvatarSessionState | null = null;
  let attached: EventTarget | null = null;
  let handler: ((event: Event) => void) | null = null;

  function ingest(data: unknown): void {
    let parsed: unknown = data;
    if (typeof data === "string") {
      try {
        parsed = JSON.parse(data);
      } catch {
        return;
      }
    }
    if (!isSessionStateFrame(parsed)) {
      return;
    }
    latest = parsed.state;
    for (const listener of listeners) {
      try {
        listener(parsed);
      } catch {
        // Subscriber errors are isolated — never block the pipeline.
      }
    }
  }

  function detach(): void {
    if (attached && handler) {
      attached.removeEventListener("message", handler);
    }
    attached = null;
    handler = null;
  }

  function attach(target: EventTarget): void {
    // Re-attach should replace, not duplicate, prior listener.
    detach();
    attached = target;
    handler = (event: Event) => {
      const data =
        event && typeof event === "object" && "data" in event
          ? (event as MessageEvent).data
          : undefined;
      ingest(data);
    };
    target.addEventListener("message", handler);
  }

  function subscribe(listener: AvatarSessionSubscriberListener): () => void {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  }

  function getLatest(): AvatarSessionState | null {
    return latest;
  }

  return { attach, detach, ingest, getLatest, subscribe };
}
