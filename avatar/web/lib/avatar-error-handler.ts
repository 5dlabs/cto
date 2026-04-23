// Recoverable ERROR frame handler for cto-avatar-session/v1.
// See docs/specs/avatar-session-protocol.md §"Failure Modes" and
// §"Graceful Degradation Path".
//
// This module is intentionally transport-agnostic: callers feed raw frames
// via `ingest()` and provide an `onReconnect` callback that drives the
// underlying connection lifecycle (LiveKit room, voice-bridge WS, etc.).
// It does not own the socket — it only tracks the recovery state machine.

import { isErrorFrame, type ErrorFrame } from "./avatar-state";

export type AvatarErrorPhase = "healthy" | "recovering" | "fatal";

export type AvatarErrorState =
  | { phase: "healthy" }
  | { phase: "recovering"; lastError: ErrorFrame; attempt: number }
  | { phase: "fatal"; lastError: ErrorFrame };

export type AvatarErrorHandlerOptions = {
  onReconnect: () => void | Promise<void>;
  onStateChange?: (state: AvatarErrorState) => void;
  // Schedule matches spec: 3 retries exponential (500ms, 1500ms, 4500ms).
  backoffMs?: readonly number[];
  // Overridable for tests.
  scheduleTimeout?: (fn: () => void, ms: number) => ReturnType<typeof setTimeout>;
  clearScheduledTimeout?: (handle: ReturnType<typeof setTimeout>) => void;
};

const DEFAULT_BACKOFF_MS: readonly number[] = [500, 1500, 4500];

export class AvatarErrorHandler {
  private readonly onReconnect: () => void | Promise<void>;
  private readonly onStateChange?: (state: AvatarErrorState) => void;
  private readonly backoffMs: readonly number[];
  private readonly schedule: (fn: () => void, ms: number) => ReturnType<typeof setTimeout>;
  private readonly clear: (handle: ReturnType<typeof setTimeout>) => void;

  private state: AvatarErrorState = { phase: "healthy" };
  private pendingTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(options: AvatarErrorHandlerOptions) {
    this.onReconnect = options.onReconnect;
    this.onStateChange = options.onStateChange;
    this.backoffMs = options.backoffMs ?? DEFAULT_BACKOFF_MS;
    this.schedule =
      options.scheduleTimeout ??
      ((fn, ms) => setTimeout(fn, ms));
    this.clear =
      options.clearScheduledTimeout ??
      ((handle) => clearTimeout(handle));
  }

  getState(): AvatarErrorState {
    return this.state;
  }

  /**
   * Feed a raw inbound frame. Non-ERROR frames and malformed payloads are
   * ignored. While in `recovering`, observing any non-ERROR frame is treated
   * as implicit recovery success and clears state to `healthy`.
   */
  ingest(frame: unknown): void {
    if (isErrorFrame(frame)) {
      this.handleError(frame);
      return;
    }
    if (this.state.phase === "recovering") {
      // A valid non-ERROR frame after a recoverable error means the session
      // is alive again.
      this.notifyRecovered();
    }
  }

  /**
   * Explicitly mark the session as recovered. Call from the caller when the
   * underlying connection reports a successful (re)connection. Safe to call
   * in any phase.
   */
  notifyRecovered(): void {
    this.cancelPending();
    if (this.state.phase !== "healthy") {
      this.setState({ phase: "healthy" });
    }
  }

  /** Cancel any pending reconnect and reset to healthy. */
  reset(): void {
    this.cancelPending();
    this.setState({ phase: "healthy" });
  }

  private handleError(frame: ErrorFrame): void {
    if (!frame.recoverable) {
      this.cancelPending();
      this.setState({ phase: "fatal", lastError: frame });
      return;
    }

    const nextAttempt =
      this.state.phase === "recovering" ? this.state.attempt + 1 : 1;

    if (nextAttempt > this.backoffMs.length) {
      // Exhausted retries — escalate to fatal with the last error kept.
      this.cancelPending();
      this.setState({ phase: "fatal", lastError: frame });
      return;
    }

    const delay = this.backoffMs[nextAttempt - 1];
    this.cancelPending();
    this.setState({ phase: "recovering", lastError: frame, attempt: nextAttempt });
    this.pendingTimer = this.schedule(() => {
      this.pendingTimer = null;
      try {
        const maybe = this.onReconnect();
        if (maybe && typeof (maybe as Promise<void>).catch === "function") {
          (maybe as Promise<void>).catch(() => {
            // Caller is responsible for surfacing failure; next ERROR frame
            // (if any) will advance the attempt counter.
          });
        }
      } catch {
        // swallow — mirrors promise rejection path
      }
    }, delay);
  }

  private cancelPending(): void {
    if (this.pendingTimer != null) {
      this.clear(this.pendingTimer);
      this.pendingTimer = null;
    }
  }

  private setState(next: AvatarErrorState): void {
    this.state = next;
    this.onStateChange?.(next);
  }
}

export function createAvatarErrorHandler(
  options: AvatarErrorHandlerOptions,
): AvatarErrorHandler {
  return new AvatarErrorHandler(options);
}
