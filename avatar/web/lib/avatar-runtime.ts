import { createEmptyAvatarState, deriveGestureScaffold, deriveVisemeScaffold, type AvatarCueSource, type AvatarRuntimeAdapter, type AvatarRuntimeInput, type AvatarRuntimeKind, type AvatarStatePayload, type AvatarVoiceState, type AvatarConnectionState, type VoiceBridgeFrame } from "@/lib/avatar-state";
import { createMetricsRecorder, latencyMs, MetricsRecorder } from "@/lib/avatar-metrics";

/**
 * Build the `metrics` bag for the projected payload. Preserves any fields
 * already present on `input.timing`, adds derived protocol metrics, then
 * overlays the recorder snapshot (recorder wins on collision).
 */
function buildMetricsBag(
  input: AvatarRuntimeInput,
  recorder?: MetricsRecorder,
): Record<string, unknown> {
  const base: Record<string, unknown> = { ...(input.timing as Record<string, unknown>) };

  const connLatency = latencyMs(
    input.timing.connectionRequestedAt,
    input.timing.roomConnectedAt,
  );
  if (connLatency !== null) {
    base.connection_latency_ms = connLatency;
  }

  const audioLatency = latencyMs(
    input.timing.roomConnectedAt,
    input.timing.audioReadyAt,
  );
  if (audioLatency !== null) {
    base.audio_latency_ms = audioLatency;
  }

  if (recorder) {
    Object.assign(base, recorder.snapshot());
  }
  return base;
}

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
    case "talkinghead":
      return new TalkingHeadAdapter();
    case "derived-text":
      return new DerivedTextAdapter();
    case "deterministic":
    default:
      return new DeterministicAdapter();
  }
}

/**
 * Adapter for the 3D TalkingHead runtime. Lip-sync visemes are driven
 * in real time by HeadAudio inside `TalkingHeadView` from the incoming
 * LiveKit audio stream, so we don't emit viseme cues here — the
 * projected payload only advertises the runtime kind plus agent
 * transcript/voice state for the telemetry panel.
 */
export class TalkingHeadAdapter implements AvatarRuntimeAdapter {
  readonly kind: AvatarRuntimeKind = "talkinghead";
  readonly cueSource: AvatarCueSource = "none";
  private readonly metrics: MetricsRecorder;

  constructor(metrics: MetricsRecorder = createMetricsRecorder()) {
    this.metrics = metrics;
  }

  project(input: AvatarRuntimeInput): AvatarStatePayload {
    const voiceState = normalizeVoiceState(input.lk.state);
    const connectionState = deriveConnectionState(input.lk.state);

    return {
      ...createEmptyAvatarState(),
      connectionState,
      voiceState,
      runtime: {
        kind: this.kind,
        ready: Boolean(input.lk.audioTrack),
        fallbackActive: false,
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
      metrics: buildMetricsBag(input, this.metrics),
      trackDebug: {},
    };
  }
}

export class DeterministicAdapter implements AvatarRuntimeAdapter {
  readonly kind: AvatarRuntimeKind = "deterministic-fallback";
  readonly cueSource: AvatarCueSource = "none";
  private readonly metrics: MetricsRecorder;

  constructor(metrics: MetricsRecorder = createMetricsRecorder()) {
    this.metrics = metrics;
  }

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
      metrics: buildMetricsBag(input, this.metrics),
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
  private readonly metrics: MetricsRecorder;

  constructor(metrics: MetricsRecorder = createMetricsRecorder()) {
    this.metrics = metrics;
  }

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
      metrics: buildMetricsBag(input, this.metrics),
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
