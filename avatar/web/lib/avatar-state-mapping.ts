// Pure mapping layer: agent state → avatar state projection.
// See docs/specs/avatar-session-protocol.md §"Agent-State-to-Avatar-State Mapping".
//
// This module is intentionally decoupled from avatar-runtime.ts. The runtime
// today derives voiceState directly from LiveKit connection state strings via
// normalizeVoiceState(). Once the session-state protocol (cto-avatar-session/v1)
// becomes the source of truth, runtime adapters should call
// mapAgentStateToAvatar() instead of inlining the projection.
//
// TODO(avatar-runtime): migrate DeterministicFallbackRuntime /
// RemoteVideoRuntime in avatar-runtime.ts to consume
// AgentState via mapAgentStateToAvatar rather than re-deriving voiceState +
// gestures from lk.state directly.

import type {
  AvatarCueSource,
  AvatarGestureCue,
  AvatarSessionState,
  AvatarVoiceState,
} from "./avatar-state";

export type AgentState =
  | "idle"
  | "connecting"
  | "listening"
  | "thinking"
  | "speaking"
  | "error";

export interface AgentStateMapping {
  voiceState: AvatarVoiceState;
  gesture: AvatarGestureCue;
  cueSource: AvatarCueSource;
}

/**
 * Pure, total mapping from the 6-state agent model to the avatar projection.
 * Matches the canonical table in docs/specs/avatar-session-protocol.md exactly.
 *
 * Intensities for idle/listen/speak/error mirror deriveGestureScaffold() in
 * avatar-state.ts. The "thinking" row is unique to this mapping (agent model
 * has no matching voiceState); it maps voiceState="listening" per spec with
 * a think gesture at intensity 0.6 — stronger than the 0.35 used for
 * connecting (which is more passive).
 */
export function mapAgentStateToAvatar(agentState: AgentState): AgentStateMapping {
  switch (agentState) {
    case "idle":
      return {
        voiceState: "idle",
        gesture: { name: "idle", intensity: 0.3 },
        cueSource: "none",
      };
    case "connecting":
      return {
        voiceState: "connecting",
        gesture: { name: "think", intensity: 0.35 },
        cueSource: "none",
      };
    case "listening":
      return {
        voiceState: "listening",
        gesture: { name: "listen", intensity: 0.7 },
        cueSource: "none",
      };
    case "thinking":
      return {
        voiceState: "listening",
        gesture: { name: "think", intensity: 0.6 },
        cueSource: "none",
      };
    case "speaking":
      return {
        voiceState: "speaking",
        gesture: { name: "speak", intensity: 0.9 },
        cueSource: "elevenlabs-alignment",
      };
    case "error":
      return {
        voiceState: "error",
        gesture: { name: "acknowledge", intensity: 0.2 },
        cueSource: "none",
      };
    default: {
      // Exhaustiveness check — TS compile error if AgentState grows without
      // a matching case above.
      const _exhaustive: never = agentState;
      throw new Error(`Unhandled AgentState: ${String(_exhaustive)}`);
    }
  }
}

/**
 * Bridge the 8-state session lifecycle (cto-avatar-session/v1) down to the
 * 6-state agent model. Pure and total.
 *
 * - reconnecting → connecting (re-establishing transport)
 * - disconnecting → idle (winding down; no activity to project)
 * - connected (without further activity signal) → idle
 * - idle/listening/speaking/error pass through unchanged
 */
export function mapSessionStateToAgentState(
  sessionState: AvatarSessionState,
): AgentState {
  switch (sessionState) {
    case "idle":
      return "idle";
    case "connecting":
      return "connecting";
    case "reconnecting":
      return "connecting";
    case "connected":
      return "idle";
    case "disconnecting":
      return "idle";
    case "listening":
      return "listening";
    case "speaking":
      return "speaking";
    case "error":
      return "error";
    default: {
      const _exhaustive: never = sessionState;
      throw new Error(`Unhandled AvatarSessionState: ${String(_exhaustive)}`);
    }
  }
}
