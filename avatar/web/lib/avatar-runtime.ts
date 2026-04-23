import { createEmptyAvatarState, deriveGestureScaffold, deriveVisemeScaffold, type AvatarCueSource, type AvatarRuntimeAdapter, type AvatarRuntimeInput, type AvatarRuntimeKind, type AvatarStatePayload, type AvatarVoiceState, type AvatarConnectionState, type VoiceBridgeFrame } from "@/lib/avatar-state";
import { ElevenLabsAlignmentAdapter } from "@/lib/runtimes/elevenlabs-alignment";

function normalizeVoiceState(state: string): AvatarVoiceState {
  if (
    state === "connecting" ||
    state === "listening" ||
    state === "speaking" ||
    state === "idle" ||
    state === "error"
  ) {
    return state;
  }

  return "idle";
}

function deriveConnectionState(lkState: string): AvatarConnectionState {
  return lkState === "connected" ? "connected" : "connecting";
}

export function pickAvatarAdapter(
  kind: string | undefined = process.env.NEXT_PUBLIC_AVATAR_RUNTIME,
): AvatarRuntimeAdapter {
  switch (kind) {
    case "elevenlabs-alignment":
      return new ElevenLabsAlignmentAdapter();
    case "derived-text":
      return new DerivedTextAdapter();
    case "deterministic":
    default:
      return new DeterministicAdapter();
  }
}

export class DeterministicAdapter implements AvatarRuntimeAdapter {
  readonly kind: AvatarRuntimeKind = "deterministic-fallback";
  readonly cueSource: AvatarCueSource = "none";

  project(input: AvatarRuntimeInput): AvatarStatePayload {
    const voiceState = normalizeVoiceState(input.lk.state);
    const connectionState = deriveConnectionState(input.lk.state);

    return {
      ...createEmptyAvatarState(),
      connectionState,
      voiceState,
      runtime: {
        kind: this.kind,
        ready: Boolean(input.lk.audioTrack) || Boolean(input.lk.videoTrack),
        fallbackActive: true,
        cueSource: this.cueSource,
      },
      transcript: {
        latestUserText: input.lk.latestUserText,
        latestAgentText: input.lk.latestAgentText,
      },
      media: {
        audioTrackReady: Boolean(input.lk.audioTrack),
        videoTrackReady: Boolean(input.lk.videoTrack),
      },
      room: {
        roomName: input.lk.roomName,
        identity: input.lk.identity,
      },
      error: input.error,
      utterance: input.utterance,
      cues: {
        visemes: [],
        gestures: deriveGestureScaffold(voiceState),
      },
      metrics: input.timing as Record<string, unknown>,
      trackDebug: {},
    };
  }

  ingestBridgeFrame(_frame: VoiceBridgeFrame): void {
    // no-op for deterministic fallback
  }
}

export class DerivedTextAdapter implements AvatarRuntimeAdapter {
  readonly kind: AvatarRuntimeKind = "deterministic-fallback";
  readonly cueSource: AvatarCueSource = "derived-text";
  private replyBuffer: string = "";
  private utteranceStart: number | null = null;

  project(input: AvatarRuntimeInput): AvatarStatePayload {
    const voiceState = normalizeVoiceState(input.lk.state);
    const connectionState = deriveConnectionState(input.lk.state);

    const visemes =
      voiceState === "speaking"
        ? deriveVisemeScaffold(this.replyBuffer, voiceState)
        : [];

    return {
      ...createEmptyAvatarState(),
      connectionState,
      voiceState,
      runtime: {
        kind: this.kind,
        ready: Boolean(input.lk.audioTrack) || Boolean(input.lk.videoTrack),
        fallbackActive: true,
        cueSource: this.cueSource,
      },
      transcript: {
        latestUserText: input.lk.latestUserText,
        latestAgentText: this.replyBuffer || input.lk.latestAgentText,
      },
      media: {
        audioTrackReady: Boolean(input.lk.audioTrack),
        videoTrackReady: Boolean(input.lk.videoTrack),
      },
      room: {
        roomName: input.lk.roomName,
        identity: input.lk.identity,
      },
      error: input.error,
      utterance:
        this.replyBuffer && this.utteranceStart
          ? {
              id: "derived",
              startedAtMs: this.utteranceStart,
              text: this.replyBuffer,
              isFinal: false,
            }
          : undefined,
      cues: {
        visemes,
        gestures: deriveGestureScaffold(voiceState),
      },
      metrics: input.timing as Record<string, unknown>,
      trackDebug: {},
    };
  }

  ingestBridgeFrame(frame: VoiceBridgeFrame): void {
    switch (frame.type) {
      case "reply_delta":
        this.replyBuffer += frame.text;
        break;
      case "reply_text":
        this.replyBuffer = frame.text;
        this.utteranceStart = performance.now();
        break;
      case "turn_done":
        this.replyBuffer = "";
        this.utteranceStart = null;
        break;
      case "started":
        this.replyBuffer = "";
        this.utteranceStart = null;
        break;
      case "error":
        break;
    }
  }
}
