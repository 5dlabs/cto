"use client";

import "@livekit/components-styles";

import {
  AudioTrack,
  LiveKitRoom,
  TrackToggle,
  useRoomContext,
  useTranscriptions,
  useVoiceAssistant,
} from "@livekit/components-react";
import AvatarRuntimeSurface from "@/components/AvatarRuntimeSurface";
import LiveKitAudioBridge from "@/components/LiveKitAudioBridge";
import type { TalkingHeadHandle } from "@/components/TalkingHeadView";
import { MORGAN_DEFAULT_GLB_URL } from "@/config/morgan";
import { pickAvatarAdapter } from "@/lib/avatar-runtime";
import {
  type AvatarRuntimeAdapter,
  type AvatarRuntimeInput,
  type AvatarStatePayload,
} from "@/lib/avatar-state";
import { Track } from "livekit-client";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

type TokenResponse = {
  identity: string;
  roomName: string;
  serverUrl: string;
  token: string;
};

type RoomProps = {
  autoConnect?: boolean;
  compact?: boolean;
};

type AgentTelemetryProps = {
  compact: boolean;
  connectionRequestedAt: number | null;
  roomConnectedAt: number | null;
};



function formatLatency(ms: number | null): string {
  if (ms === null) {
    return "waiting";
  }

  return `${Math.round(ms)} ms`;
}

function emitHostAvatarState(payload: AvatarStatePayload) {
  if (typeof window === "undefined" || window.parent === window) {
    return;
  }

  window.parent.postMessage(
    {
      type: "cto-avatar-state",
      payload,
    },
    "*",
  );
}

function StatusPill({
  label,
  tone = "neutral",
}: {
  label: string;
  tone?: "neutral" | "emerald" | "violet";
}) {
  const toneClass =
    tone === "emerald"
      ? "border-emerald-400/35 bg-emerald-400/12 text-emerald-100"
      : tone === "violet"
        ? "border-fuchsia-400/35 bg-fuchsia-400/12 text-fuchsia-100"
        : "border-white/12 bg-white/10 text-slate-200";

  return (
    <span
      className={`rounded-full border px-3 py-1 text-[11px] font-medium uppercase tracking-[0.22em] ${toneClass}`}
    >
      {label}
    </span>
  );
}

function ActivityGlyph({
  active,
  tone,
}: {
  active: boolean;
  tone: "emerald" | "violet";
}) {
  const activeClass =
    tone === "emerald" ? "bg-emerald-300/90" : "bg-fuchsia-300/90";
  const idleClass = "bg-white/10";

  return (
    <div className="flex items-end gap-1">
      {[0, 1, 2, 3, 4].map((bar) => (
        <span
          key={bar}
          className={`w-1.5 rounded-full transition-all duration-200 ${
            active ? activeClass : idleClass
          }`}
          style={{
            height: `${active ? 12 + ((bar % 3) + 1) * 8 : 8}px`,
            opacity: active ? 0.65 + bar * 0.06 : 0.5,
          }}
        />
      ))}
    </div>
  );
}

function InsightCard({
  eyebrow,
  tone,
  body,
}: {
  eyebrow: string;
  tone: "emerald" | "violet";
  body: string;
}) {
  const eyebrowClass =
    tone === "emerald" ? "text-emerald-200/90" : "text-fuchsia-200/90";
  const ringClass =
    tone === "emerald"
      ? "border-emerald-400/20 bg-emerald-950/35"
      : "border-fuchsia-400/20 bg-fuchsia-950/25";

  return (
    <div className={`rounded-[1.6rem] border p-4 ${ringClass}`}>
      <p className={`text-[11px] uppercase tracking-[0.28em] ${eyebrowClass}`}>
        {eyebrow}
      </p>
      <p className="mt-3 text-sm leading-6 text-slate-100">{body}</p>
    </div>
  );
}

function AgentTelemetry({
  compact,
  connectionRequestedAt,
  roomConnectedAt,
}: AgentTelemetryProps) {
  const room = useRoomContext();
  const { state, audioTrack, videoTrack, agentTranscriptions } = useVoiceAssistant();
  const allTranscriptions = useTranscriptions({ room });
  const [audioReadyAt, setAudioReadyAt] = useState<number | null>(null);
  const [videoReadyAt, setVideoReadyAt] = useState<number | null>(null);
  const [speakingAt, setSpeakingAt] = useState<number | null>(null);

  const userTranscriptions = useMemo(
    () =>
      allTranscriptions.filter(
        (transcription) =>
          transcription.participantInfo.identity === room.localParticipant.identity,
      ),
    [allTranscriptions, room.localParticipant.identity],
  );
  const latestUserText = userTranscriptions.at(-1)?.text?.trim() ?? "";
  const latestTranscript =
    agentTranscriptions.at(-1)?.text?.trim() ?? "Morgan is ready when you are.";

  useEffect(() => {
    if (audioTrack && audioReadyAt === null) {
      const frame = window.requestAnimationFrame(() => {
        setAudioReadyAt(performance.now());
      });
      return () => window.cancelAnimationFrame(frame);
    }
  }, [audioReadyAt, audioTrack]);

  useEffect(() => {
    if (videoTrack && videoReadyAt === null) {
      const frame = window.requestAnimationFrame(() => {
        setVideoReadyAt(performance.now());
      });
      return () => window.cancelAnimationFrame(frame);
    }
  }, [videoReadyAt, videoTrack]);

  useEffect(() => {
    if (state === "speaking" && speakingAt === null) {
      const frame = window.requestAnimationFrame(() => {
        setSpeakingAt(performance.now());
      });
      return () => window.cancelAnimationFrame(frame);
    }
  }, [speakingAt, state]);

  const trackDebug = useMemo(() => {
    return {
      selectedAudioTrack: audioTrack
        ? {
            participant: audioTrack.participant.identity,
            source: audioTrack.source,
            sid: audioTrack.publication.trackSid,
            name: audioTrack.publication.trackName,
            kind: audioTrack.publication.kind,
            subscribed: audioTrack.publication.isSubscribed,
            muted: audioTrack.publication.isMuted,
            enabled: audioTrack.publication.isEnabled,
          }
        : null,
      selectedVideoTrack: videoTrack
        ? {
            participant: videoTrack.participant.identity,
            source: videoTrack.source,
            sid: videoTrack.publication.trackSid,
            name: videoTrack.publication.trackName,
            kind: videoTrack.publication.kind,
            subscribed: videoTrack.publication.isSubscribed,
            muted: videoTrack.publication.isMuted,
            enabled: videoTrack.publication.isEnabled,
          }
        : null,
      remoteParticipants: Array.from(room.remoteParticipants.values()).map((participant) => ({
        identity: participant.identity,
        kind: participant.kind,
        publishOnBehalf: participant.attributes["lk.publish_on_behalf"] ?? null,
        tracks: Array.from(participant.trackPublications.values()).map((publication) => ({
          sid: publication.trackSid,
          name: publication.trackName,
          source: publication.source,
          kind: publication.kind,
          subscribed: publication.isSubscribed,
          muted: publication.isMuted,
          enabled: publication.isEnabled,
        })),
      })),
    };
  }, [audioTrack, room, videoTrack]);

  const metrics = useMemo(() => {
    return {
      connectionRequestedMs: connectionRequestedAt,
      roomConnectedMs:
        connectionRequestedAt !== null && roomConnectedAt
          ? roomConnectedAt - connectionRequestedAt
          : null,
      audioTrackReadyMs:
        connectionRequestedAt !== null && audioReadyAt
          ? audioReadyAt - connectionRequestedAt
          : null,
      videoTrackReadyMs:
        connectionRequestedAt !== null && videoReadyAt
          ? videoReadyAt - connectionRequestedAt
          : null,
      firstSpeakingStateMs:
        connectionRequestedAt !== null && speakingAt
          ? speakingAt - connectionRequestedAt
          : null,
      agentState: state,
      latestTranscript,
    };
  }, [
    audioReadyAt,
    connectionRequestedAt,
    latestTranscript,
    roomConnectedAt,
    speakingAt,
    state,
    videoReadyAt,
  ]);

  const glbUrl =
    process.env.NEXT_PUBLIC_AVATAR_GLB_URL ?? MORGAN_DEFAULT_GLB_URL;

  const adapter = useMemo<AvatarRuntimeAdapter>(
    () => pickAvatarAdapter(process.env.NEXT_PUBLIC_AVATAR_RUNTIME),
    [],
  );

  const talkingHeadRef = useRef<TalkingHeadHandle | null>(null);

  const runtimeInput = useMemo<AvatarRuntimeInput>(
    () => ({
      lk: {
        state,
        audioTrack,
        videoTrack,
        latestUserText,
        latestAgentText: latestTranscript,
        roomName: room.name || undefined,
        identity: room.localParticipant.identity || undefined,
      },
      timing: {
        connectionRequestedAt,
        roomConnectedAt,
        audioReadyAt,
        videoReadyAt,
        speakingAt,
      },
    }),
    [
      audioTrack,
      connectionRequestedAt,
      latestTranscript,
      latestUserText,
      room.localParticipant.identity,
      room.name,
      roomConnectedAt,
      speakingAt,
      state,
      videoTrack,
      videoReadyAt,
      audioReadyAt,
    ],
  );

  const avatarState = useMemo<AvatarStatePayload>(
    () => adapter.project(runtimeInput),
    [adapter, runtimeInput],
  );

  useEffect(() => {
    emitHostAvatarState(avatarState);
  }, [avatarState]);

  if (compact) {
    return (
      <>
        {adapter.kind === "talkinghead" ? (
          <LiveKitAudioBridge talkingHeadRef={talkingHeadRef} />
        ) : null}
        <section className="grid gap-5">
        <div className="relative overflow-hidden rounded-[2.2rem] border border-white/10 bg-black/30 shadow-[0_30px_120px_-48px_rgba(14,165,233,0.75)]">
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,#155e75_0%,rgba(2,6,23,0.78)_34%,rgba(2,6,23,0.96)_100%)]" />

          <div className="absolute left-5 top-5 z-10 flex flex-wrap gap-2">
            <StatusPill label="Morgan" />
            <StatusPill
              label={String(state)}
              tone={state === "speaking" ? "violet" : state === "listening" ? "emerald" : "neutral"}
            />
            <StatusPill
              label={audioTrack ? "audio live" : "audio pending"}
              tone={audioTrack ? "emerald" : "neutral"}
            />
            <StatusPill
              label={videoTrack ? "video live" : "video pending"}
              tone={videoTrack ? "violet" : "neutral"}
            />
          </div>

          <div className="absolute inset-x-0 bottom-0 z-10 flex items-end justify-between gap-4 p-5">
            <div className="rounded-[1.4rem] border border-white/10 bg-slate-950/70 px-4 py-3 backdrop-blur-md">
              <p className="text-[11px] uppercase tracking-[0.3em] text-cyan-100/75">
                {state === "listening" ? "Listening" : "Live session"}
              </p>
              <div className="mt-3 flex items-center gap-3 text-sm text-slate-100">
                <ActivityGlyph
                  active={state === "listening" || Boolean(latestUserText)}
                  tone="emerald"
                />
                <span>
                  {latestUserText ||
                    (state === "connecting"
                      ? "Joining the room"
                      : "Waiting for your first turn")}
                </span>
              </div>
            </div>

            <div className="rounded-full border border-white/10 bg-slate-950/70 px-4 py-2 text-xs text-slate-200 backdrop-blur-md">
              Audio {formatLatency(connectionRequestedAt !== null && audioReadyAt ? audioReadyAt - connectionRequestedAt : null)}
            </div>
          </div>

          <div className="aspect-[10/13] w-full bg-linear-to-b from-slate-900 via-slate-950 to-black">
            <AvatarRuntimeSurface
              compact
              state={avatarState}
              videoTrack={videoTrack}
              talkingHeadRef={talkingHeadRef}
              glbUrl={glbUrl}
            />
          </div>
        </div>
      </section>
      </>
    );
  }

  return (
    <>
      {adapter.kind === "talkinghead" ? (
        <LiveKitAudioBridge talkingHeadRef={talkingHeadRef} />
      ) : null}
      <section className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_340px]">
      <div className="overflow-hidden rounded-[2rem] border border-white/10 bg-black/40 shadow-2xl shadow-black/25">
        <div className="aspect-[9/14] w-full bg-linear-to-b from-slate-900 via-slate-950 to-black">
          <AvatarRuntimeSurface
            state={avatarState}
            videoTrack={videoTrack}
            talkingHeadRef={talkingHeadRef}
            glbUrl={glbUrl}
          />
        </div>
      </div>

      <aside className="flex flex-col gap-4 rounded-[2rem] border border-white/10 bg-slate-950/80 p-5 text-sm text-slate-200">
        <div>
          <p className="text-xs uppercase tracking-[0.24em] text-cyan-300/80">
            Agent State
          </p>
          <p className="mt-2 text-xl font-semibold capitalize text-white">{state}</p>
        </div>

        <InsightCard
          eyebrow={state === "listening" ? "Listening now" : "Heard you"}
          tone="emerald"
          body={
            latestUserText ||
            (state === "listening"
              ? "Say something and your words will appear here."
              : "Your words will appear here once you speak.")
          }
        />

        <InsightCard eyebrow="Morgan said" tone="violet" body={latestTranscript} />

        <div className="grid gap-2 rounded-2xl bg-white/5 p-4">
          <div className="flex items-center justify-between">
            <span>Room connected</span>
            <span>
              {formatLatency(
                connectionRequestedAt !== null && roomConnectedAt
                  ? roomConnectedAt - connectionRequestedAt
                  : null,
              )}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span>Audio track ready</span>
            <span>
              {formatLatency(
                connectionRequestedAt !== null && audioReadyAt
                  ? audioReadyAt - connectionRequestedAt
                  : null,
              )}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span>Video track ready</span>
            <span>
              {formatLatency(
                connectionRequestedAt !== null && videoReadyAt
                  ? videoReadyAt - connectionRequestedAt
                  : null,
              )}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span>First speaking state</span>
            <span>
              {formatLatency(
                connectionRequestedAt !== null && speakingAt
                  ? speakingAt - connectionRequestedAt
                  : null,
              )}
            </span>
          </div>
        </div>

        <div className="rounded-2xl bg-white/5 p-4">
          <p className="text-xs uppercase tracking-[0.24em] text-cyan-300/80">
            Track Debug
          </p>
          <pre className="mt-3 overflow-x-auto text-[11px] leading-5 text-slate-300">
            {JSON.stringify(trackDebug, null, 2)}
          </pre>
        </div>
      </aside>
    </section>
    </>
  );
}

function SessionControls({
  compact,
  onReset,
}: {
  compact: boolean;
  onReset: () => void;
}) {
  const room = useRoomContext();

  const disconnect = useCallback(async () => {
    await room.disconnect();
    onReset();
  }, [onReset, room]);

  return (
    <div
      className={
        compact
          ? "flex flex-wrap items-center gap-3 rounded-[1.6rem] border border-white/10 bg-white/5 px-4 py-3"
          : "mt-6 flex flex-wrap items-center gap-3"
      }
    >
      <TrackToggle
        source={Track.Source.Microphone}
        initialState
        className="rounded-full border border-white/10 bg-white/5 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10"
      >
        Mic
      </TrackToggle>
      <button
        type="button"
        onClick={disconnect}
        className="rounded-full border border-white/10 bg-white/5 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10"
      >
        Disconnect
      </button>
    </div>
  );
}

function AssistantAudioRenderer({ enabled = true }: { enabled?: boolean }) {
  const { audioTrack } = useVoiceAssistant();

  if (!enabled || !audioTrack) {
    return null;
  }

  return (
    <div style={{ display: "none" }}>
      <AudioTrack trackRef={audioTrack} />
    </div>
  );
}

export default function Room({
  autoConnect = false,
  compact = false,
}: RoomProps) {
  const [connection, setConnection] = useState<TokenResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [connectionRequestedAt, setConnectionRequestedAt] = useState<number | null>(null);
  const [roomConnectedAt, setRoomConnectedAt] = useState<number | null>(null);
  const autoConnectTriggeredRef = useRef(false);

  const reset = useCallback(() => {
    setConnection(null);
    setError(null);
    setConnectionRequestedAt(null);
    setRoomConnectedAt(null);
  }, []);

  const connect = useCallback(async () => {
    setError(null);
    setConnectionRequestedAt(performance.now());

    const response = await fetch("/api/token", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      const payload = (await response.json().catch(() => null)) as { error?: string } | null;
      throw new Error(payload?.error ?? "Unable to create a LiveKit token.");
    }

    const payload = (await response.json()) as TokenResponse;
    setConnection(payload);
  }, []);

  const handleConnect = useCallback(async () => {
    try {
      await connect();
    } catch (cause) {
      const message =
        cause instanceof Error ? cause.message : "Unable to create a LiveKit token.";
      setError(message);
      setConnectionRequestedAt(null);
    }
  }, [connect]);

  const failEmbeddedMedia = useCallback((message: string) => {
    setConnection(null);
    setConnectionRequestedAt(null);
    setRoomConnectedAt(null);
    setError(message);
  }, []);

  const handleRoomError = useCallback(
    (cause: Error) => {
      if (cause.name === "NotAllowedError") {
        failEmbeddedMedia(
          "Microphone access is blocked in the embedded avatar view. Allow microphone access for CTO and reload the avatar tab.",
        );
        return;
      }

      failEmbeddedMedia(cause.message || "Unable to keep the Morgan session connected.");
    },
    [failEmbeddedMedia],
  );

  const handleMediaDeviceFailure = useCallback(
    (failure?: unknown, kind?: MediaDeviceKind) => {
      const source = kind === "audioinput" ? "microphone" : kind ?? "media device";
      const detail = failure ? ` (${String(failure)})` : "";
      failEmbeddedMedia(
        `Access to the ${source} was denied in the embedded avatar view. Allow it for CTO and retry.${detail}`,
      );
    },
    [failEmbeddedMedia],
  );

  const adapter = useMemo<AvatarRuntimeAdapter>(
    () => pickAvatarAdapter(process.env.NEXT_PUBLIC_AVATAR_RUNTIME),
    [],
  );

  useEffect(() => {
    emitHostAvatarState(
      adapter.project({
        lk: {
          state: error ? "error" : connection ? "connected" : "idle",
          audioTrack: null,
          videoTrack: null,
          latestUserText: "",
          latestAgentText: "",
          roomName: connection?.roomName,
          identity: connection?.identity,
        },
        timing: {
          connectionRequestedAt,
          roomConnectedAt: null,
          audioReadyAt: null,
          videoReadyAt: null,
          speakingAt: null,
        },
        error: error ?? undefined,
      }),
    );
  }, [adapter, connection, connectionRequestedAt, error]);

  useEffect(() => {
    if (
      !autoConnect ||
      autoConnectTriggeredRef.current ||
      connection ||
      connectionRequestedAt !== null
    ) {
      return;
    }

    autoConnectTriggeredRef.current = true;
    const timer = window.setTimeout(() => {
      void handleConnect();
    }, 0);

    return () => window.clearTimeout(timer);
  }, [autoConnect, connection, connectionRequestedAt, handleConnect]);

  if (!connection) {
    if (compact) {
      return (
        <section className="relative overflow-hidden rounded-[2.2rem] border border-white/10 bg-slate-950/80 shadow-[0_28px_120px_-52px_rgba(14,165,233,0.75)]">
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,#155e75_0%,rgba(2,6,23,0.82)_30%,rgba(2,6,23,0.98)_100%)]" />
          <div className="relative flex min-h-[760px] items-center justify-center px-8 py-12 text-center">
            <div className="max-w-md">
              <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-cyan-300/30 bg-cyan-400/10">
                <div className="h-10 w-10 rounded-full bg-cyan-300/80 blur-[1px]" />
              </div>
              <p className="mt-8 text-[11px] uppercase tracking-[0.34em] text-cyan-200/70">
                Morgan avatar
              </p>
              <h2 className="mt-4 text-3xl font-semibold tracking-tight text-white">
                {error
                  ? "Morgan couldn’t join the room."
                  : connectionRequestedAt !== null
                    ? "Bringing Morgan online."
                    : "Preparing Morgan."}
              </h2>
              <p className="mt-4 text-sm leading-7 text-slate-300">
                {error ??
                  "LiveKit is connecting and LemonSlice is warming up. Morgan should appear automatically as soon as the session is ready."}
              </p>
              {error ? (
                <button
                  type="button"
                  onClick={() => void handleConnect()}
                  className="mt-8 rounded-full border border-cyan-300/35 bg-cyan-400/10 px-5 py-3 text-sm font-medium text-cyan-100 transition hover:bg-cyan-400/15"
                >
                  Retry connection
                </button>
              ) : (
                <div className="mt-8 inline-flex items-center gap-3 rounded-full border border-white/10 bg-white/5 px-4 py-2 text-xs uppercase tracking-[0.24em] text-slate-300">
                  <span className="h-2.5 w-2.5 rounded-full bg-cyan-300 shadow-[0_0_18px_rgba(103,232,249,0.9)]" />
                  Connecting
                </div>
              )}
            </div>
          </div>
        </section>
      );
    }

    return (
      <section className="rounded-[2rem] border border-white/10 bg-slate-950/80 p-8 shadow-2xl shadow-black/25">
        <div className="max-w-2xl">
          <p className="text-xs uppercase tracking-[0.28em] text-cyan-300/80">
            Morgan Talking Avatar
          </p>
          <h2 className="mt-4 text-4xl font-semibold tracking-tight text-white">
            Live voice conversation with a lip-synced Morgan avatar.
          </h2>
          <p className="mt-4 max-w-xl text-base leading-7 text-slate-300">
            This baseline client joins a LiveKit room, streams your microphone, plays Morgan’s
            speech, and exposes the room and media timings we need for latency tuning.
          </p>
        </div>

        <div className="mt-8 flex flex-wrap items-center gap-4">
          <button
            type="button"
            onClick={() => void handleConnect()}
            className="rounded-full bg-cyan-400 px-6 py-3 text-sm font-semibold text-slate-950 transition hover:bg-cyan-300"
          >
            Talk to Morgan
          </button>
          <span className="text-sm text-slate-400">
            Uses `/api/token` to mint a short-lived LiveKit room token.
          </span>
        </div>

        {error ? (
          <p className="mt-4 rounded-2xl border border-rose-500/40 bg-rose-500/10 px-4 py-3 text-sm text-rose-100">
            {error}
          </p>
        ) : null}
      </section>
    );
  }

  return (
    <LiveKitRoom
      token={connection.token}
      serverUrl={connection.serverUrl}
      connect
      audio
      video={false}
      onConnected={() => setRoomConnectedAt(performance.now())}
      onDisconnected={reset}
      onError={handleRoomError}
      onMediaDeviceFailure={handleMediaDeviceFailure}
      className={compact ? "grid gap-5" : "grid gap-6"}
    >
      {!compact ? (
        <div className="flex flex-wrap items-center justify-between gap-3 rounded-[2rem] border border-white/10 bg-white/5 px-5 py-4 text-sm text-slate-300">
          <div>
            <p className="font-medium text-white">Room</p>
            <p>{connection.roomName}</p>
          </div>
          <div>
            <p className="font-medium text-white">Identity</p>
            <p>{connection.identity}</p>
          </div>
        </div>
      ) : null}

      <AgentTelemetry
        compact={compact}
        connectionRequestedAt={connectionRequestedAt}
        roomConnectedAt={roomConnectedAt}
      />
      <SessionControls compact={compact} onReset={reset} />
      {/*
        LiveKit's native audio element plays Morgan's speech AND keeps
        the underlying MediaStreamTrack "flowing" in Chrome, which is
        required for our Web Audio graph (MediaStreamAudioSourceNode →
        HeadAudio viseme detector) to receive samples. Keep this
        mounted for every runtime.
      */}
      <AssistantAudioRenderer />
    </LiveKitRoom>
  );
}
