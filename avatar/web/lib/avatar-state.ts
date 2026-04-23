export const AVATAR_STATE_PROTOCOL = "cto-avatar-state/v1" as const;

export type AvatarConnectionState = "idle" | "connecting" | "connected" | "error";
export type AvatarVoiceState = "idle" | "connecting" | "listening" | "speaking" | "error";
export type AvatarRuntimeKind = "deterministic-fallback" | "remote-video" | "talkinghead";
export type AvatarCueSource =
  | "none"
  | "derived-text"
  | "elevenlabs-alignment"
  | "ovrlipsync-wasm";

export type OvrLipSyncViseme =
  | "sil"
  | "PP"
  | "FF"
  | "TH"
  | "DD"
  | "kk"
  | "CH"
  | "SS"
  | "nn"
  | "RR"
  | "aa"
  | "E"
  | "I"
  | "O"
  | "U";

export type AvatarVisemeCue = {
  atMs: number;
  durationMs?: number;
  value: OvrLipSyncViseme;
  weight?: number;
};

export type AvatarGestureCue = {
  name: "idle" | "listen" | "speak" | "think" | "acknowledge";
  intensity?: number;
};

export type AvatarUtterance = {
  id: string;
  startedAtMs: number;
  text: string;
  isFinal: boolean;
};

export type VoiceBridgeFrame =
  | { type: "started"; session_id: string; agent: string }
  | { type: "transcript"; text: string; agent: string }
  | { type: "reply_delta"; text: string; agent: string }
  | { type: "reply_text"; text: string; agent: string }
  | { type: "turn_done"; agent: string }
  | { type: "error"; error: string }
  | {
      type: "alignment";
      atMs: number;
      chars: string[];
      char_start_ms: number[];
      char_end_ms: number[];
      agent: string;
    };

export type AvatarStatePayload = {
  protocol: typeof AVATAR_STATE_PROTOCOL;
  connectionState: AvatarConnectionState;
  voiceState: AvatarVoiceState;
  runtime: {
    kind: AvatarRuntimeKind;
    ready: boolean;
    fallbackActive: boolean;
    cueSource: AvatarCueSource;
  };
  transcript: {
    latestUserText: string;
    latestAgentText: string;
  };
  media: {
    audioTrackReady: boolean;
    videoTrackReady: boolean;
  };
  utterance?: AvatarUtterance;
  cues: {
    visemes: AvatarVisemeCue[];
    gestures: AvatarGestureCue[];
  };
  room?: {
    roomName?: string;
    identity?: string;
  };
  error?: string;
  metrics?: Record<string, unknown>;
  trackDebug?: Record<string, unknown>;
};

export type AvatarRuntimeInput = {
  lk: {
    state: string;
    audioTrack: unknown | null;
    videoTrack: unknown | null;
    latestUserText: string;
    latestAgentText: string;
    roomName?: string;
    identity?: string;
  };
  timing: {
    connectionRequestedAt: number | null;
    roomConnectedAt: number | null;
    audioReadyAt: number | null;
    videoReadyAt: number | null;
    speakingAt: number | null;
  };
  utterance?: AvatarUtterance;
  error?: string;
};

export interface AvatarRuntimeAdapter {
  readonly kind: AvatarRuntimeKind;
  readonly cueSource: AvatarCueSource;

  project(input: AvatarRuntimeInput): AvatarStatePayload;

  ingestBridgeFrame?(frame: VoiceBridgeFrame): void;
}

export function createEmptyAvatarState(): AvatarStatePayload {
  return {
    protocol: AVATAR_STATE_PROTOCOL,
    connectionState: "idle",
    voiceState: "idle",
    runtime: {
      kind: "deterministic-fallback",
      ready: false,
      fallbackActive: true,
      cueSource: "none",
    },
    transcript: {
      latestUserText: "",
      latestAgentText: "",
    },
    media: {
      audioTrackReady: false,
      videoTrackReady: false,
    },
    cues: {
      visemes: [],
      gestures: [],
    },
  };
}

const VISAME_MAP: Record<string, OvrLipSyncViseme> = {
  a: "aa",
  e: "E",
  i: "I",
  o: "O",
  u: "U",
  p: "PP",
  b: "PP",
  m: "PP",
  f: "FF",
  v: "FF",
  t: "TH",
  d: "DD",
  k: "kk",
  g: "kk",
  s: "SS",
  z: "SS",
  n: "nn",
  l: "nn",
  r: "RR",
  ch: "CH",
  sh: "CH",
};

export function deriveVisemeScaffold(text: string, voiceState: AvatarVoiceState): AvatarVisemeCue[] {
  if (voiceState !== "speaking" || !text.trim()) {
    return [];
  }

  const chars = text.trim().split("");
  return chars
    .map((char, index) => ({
      atMs: index * 120,
      value: (VISAME_MAP[char.toLowerCase()] ?? "sil") as OvrLipSyncViseme,
      weight: /[aeiou]/i.test(char) ? 0.85 : 0.55,
    }))
    .filter((cue, idx, arr) => idx === 0 || cue.value !== arr[idx - 1].value);
}

export function deriveGestureScaffold(voiceState: AvatarVoiceState): AvatarGestureCue[] {
  switch (voiceState) {
    case "listening":
      return [{ name: "listen", intensity: 0.7 }];
    case "speaking":
      return [{ name: "speak", intensity: 0.9 }];
    case "connecting":
      return [{ name: "think", intensity: 0.35 }];
    case "idle":
      return [{ name: "idle", intensity: 0.3 }];
    case "error":
      return [{ name: "acknowledge", intensity: 0.2 }];
    default:
      return [];
  }
}

// ---------------------------------------------------------------------------
// cto-avatar-session/v1 — host-bound session frames
// See docs/specs/avatar-session-protocol.md §"Frame Types".
// Types + type guards only; no emitters or consumers yet.
// ---------------------------------------------------------------------------

export const AVATAR_SESSION_PROTOCOL = "cto-avatar-session/v1" as const;

export type AvatarSessionState =
  | "idle"
  | "connecting"
  | "connected"
  | "listening"
  | "speaking"
  | "reconnecting"
  | "error"
  | "disconnecting";

export type SessionStateFrame = {
  protocol: typeof AVATAR_SESSION_PROTOCOL;
  type: "SESSION_STATE";
  session_id: string;
  state: AvatarSessionState;
  agent_name: string;
  timestamp_ms: number;
};

export type ErrorFrame = {
  protocol: typeof AVATAR_SESSION_PROTOCOL;
  type: "ERROR";
  session_id: string;
  code: string;
  message: string;
  recoverable: boolean;
  timestamp_ms: number;
};

export type AvatarSessionFrame = SessionStateFrame | ErrorFrame;

const AVATAR_SESSION_STATES: ReadonlySet<AvatarSessionState> = new Set([
  "idle",
  "connecting",
  "connected",
  "listening",
  "speaking",
  "reconnecting",
  "error",
  "disconnecting",
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function isSessionStateFrame(value: unknown): value is SessionStateFrame {
  if (!isRecord(value)) return false;
  if (value.protocol !== AVATAR_SESSION_PROTOCOL) return false;
  if (value.type !== "SESSION_STATE") return false;
  if (typeof value.session_id !== "string") return false;
  if (typeof value.agent_name !== "string") return false;
  if (typeof value.timestamp_ms !== "number") return false;
  if (typeof value.state !== "string") return false;
  if (!AVATAR_SESSION_STATES.has(value.state as AvatarSessionState)) return false;
  return true;
}

export function isErrorFrame(value: unknown): value is ErrorFrame {
  if (!isRecord(value)) return false;
  if (value.protocol !== AVATAR_SESSION_PROTOCOL) return false;
  if (value.type !== "ERROR") return false;
  if (typeof value.session_id !== "string") return false;
  if (typeof value.code !== "string") return false;
  if (typeof value.message !== "string") return false;
  if (typeof value.recoverable !== "boolean") return false;
  if (typeof value.timestamp_ms !== "number") return false;
  return true;
}

export function isAvatarSessionFrame(value: unknown): value is AvatarSessionFrame {
  return isSessionStateFrame(value) || isErrorFrame(value);
}
