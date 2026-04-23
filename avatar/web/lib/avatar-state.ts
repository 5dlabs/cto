export type AvatarConnectionState = "idle" | "connecting" | "connected" | "error";
export type AvatarVoiceState = "idle" | "connecting" | "listening" | "speaking" | "error";
export type AvatarRuntimeKind = "deterministic-fallback" | "remote-video" | "talkinghead";

export type AvatarVisemeCue = {
  atMs: number;
  value: string;
  weight?: number;
};

export type AvatarGestureCue = {
  name: "idle" | "listen" | "speak" | "think" | "acknowledge";
  intensity?: number;
};

export type AvatarStatePayload = {
  connectionState: AvatarConnectionState;
  voiceState: AvatarVoiceState;
  runtime: {
    kind: AvatarRuntimeKind;
    ready: boolean;
    fallbackActive: boolean;
  };
  transcript: {
    latestUserText: string;
    latestAgentText: string;
  };
  media: {
    audioTrackReady: boolean;
    videoTrackReady: boolean;
  };
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

export function createEmptyAvatarState(): AvatarStatePayload {
  return {
    connectionState: "idle",
    voiceState: "idle",
    runtime: {
      kind: "deterministic-fallback",
      ready: false,
      fallbackActive: true,
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

export function deriveVisemeScaffold(text: string, voiceState: AvatarVoiceState): AvatarVisemeCue[] {
  if (voiceState !== "speaking" || !text.trim()) {
    return [];
  }

  return text
    .trim()
    .slice(0, 8)
    .split("")
    .map((char, index) => ({
      atMs: index * 120,
      value: /[aeiou]/i.test(char) ? "open" : /[bmp]/i.test(char) ? "closed" : "mid",
      weight: 0.55,
    }));
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
