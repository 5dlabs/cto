import {
  createEmptyAvatarState,
  deriveGestureScaffold,
  type AvatarConnectionState,
  type AvatarCueSource,
  type AvatarRuntimeAdapter,
  type AvatarRuntimeInput,
  type AvatarRuntimeKind,
  type AvatarStatePayload,
  type AvatarVoiceState,
  type OvrLipSyncViseme,
  type VoiceBridgeFrame,
} from "@/lib/avatar-state";

/**
 * Map a single character to its closest OVRLipSync viseme.
 *
 * Covers the 15 visemes:
 *   sil, PP, FF, TH, DD, kk, CH, SS, nn, RR, aa, E, I, O, U
 */
function charToViseme(ch: string): OvrLipSyncViseme {
  const lower = ch.toLowerCase();
  if (/[a]/.test(lower)) return "aa";
  if (/[e]/.test(lower)) return "E";
  if (/[i]/.test(lower)) return "I";
  if (/[o]/.test(lower)) return "O";
  if (/[u]/.test(lower)) return "U";
  if (/[pbm]/.test(lower)) return "PP";
  if (/[fv]/.test(lower)) return "FF";
  if (/[td]/.test(lower)) return "DD";
  if (/[kghq]/.test(lower)) return "kk";
  if (/[csz]/.test(lower)) return "SS";
  if (/[n]/.test(lower)) return "nn";
  if (/[r]/.test(lower)) return "RR";
  if (/[l]/.test(lower)) return "nn";
  if (/[j]/.test(lower)) return "CH";
  if (/[w]/.test(lower)) return "U";
  return "sil";
}

export class ElevenLabsAlignmentAdapter implements AvatarRuntimeAdapter {
  readonly kind: AvatarRuntimeKind = "deterministic-fallback";
  readonly cueSource: AvatarCueSource = "elevenlabs-alignment";

  private replyBuffer = "";
  private utteranceStart: number | null = null;
  private pendingVisemes: Array<{
    atMs: number;
    value: OvrLipSyncViseme;
  }> = [];

  project(input: AvatarRuntimeInput): AvatarStatePayload {
    const voiceState = input.lk.state as AvatarVoiceState;
    const connectionState = input.lk.state === "connected"
      ? "connected" as AvatarConnectionState
      : "connecting" as AvatarConnectionState;

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
              id: "alignment",
              startedAtMs: this.utteranceStart,
              text: this.replyBuffer,
              isFinal: true,
            }
          : undefined,
      cues: {
        visemes: this.pendingVisemes.map((v) => ({
          atMs: v.atMs,
          value: v.value,
          weight: 1.0,
        })),
        gestures: deriveGestureScaffold(voiceState),
      },
      metrics: input.timing as Record<string, unknown>,
      trackDebug: {},
    };
  }

  ingestBridgeFrame(frame: VoiceBridgeFrame): void {
    switch (frame.type) {
      case "reply_text":
        this.replyBuffer = frame.text;
        this.utteranceStart = performance.now();
        break;
      case "reply_delta":
        this.replyBuffer += frame.text;
        break;
      case "turn_done":
        this.replyBuffer = "";
        this.utteranceStart = null;
        this.pendingVisemes = [];
        break;
      case "started":
        this.replyBuffer = "";
        this.utteranceStart = null;
        this.pendingVisemes = [];
        break;
      case "error":
        break;
    }

    // Consume alignment frames when they arrive
    if ("type" in frame && frame.type === "alignment") {
      const alignFrame = frame as {
        type: "alignment";
        atMs: number;
        chars: string[];
        char_start_ms: number[];
        char_end_ms: number[];
        agent: string;
      };

      this.pendingVisemes = alignFrame.chars.map((ch, idx) => ({
        atMs: alignFrame.char_start_ms[idx] ?? 0,
        value: charToViseme(ch),
      }));
    }
  }
}
