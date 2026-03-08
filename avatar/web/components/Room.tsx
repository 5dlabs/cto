"use client";

import "@livekit/components-styles";

import {
  LiveKitRoom,
  RoomAudioRenderer,
  TrackToggle,
  VideoTrack,
  useRoomContext,
  useTranscriptions,
  useVoiceAssistant,
} from "@livekit/components-react";
import { Track } from "livekit-client";
import { useCallback, useEffect, useMemo, useState } from "react";

type TokenResponse = {
  identity: string;
  roomName: string;
  serverUrl: string;
  token: string;
};

type AgentTelemetryProps = {
  connectionRequestedAt: number | null;
  roomConnectedAt: number | null;
};

function formatLatency(ms: number | null): string {
  if (ms === null) {
    return "waiting";
  }
  return `${Math.round(ms)} ms`;
}

function AgentTelemetry({
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
        (t) => t.participantInfo.identity === room.localParticipant.identity,
      ),
    [allTranscriptions, room.localParticipant.identity],
  );
  const latestUserText = userTranscriptions.at(-1)?.text?.trim() ?? "";
  const recentUserTexts = useMemo(
    () => userTranscriptions.slice(-3).map((t) => t.text?.trim()).filter(Boolean),
    [userTranscriptions],
  );

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

  const latestTranscript = agentTranscriptions.at(-1)?.text ?? "Waiting for Morgan to speak.";

  const exportedMetrics = useMemo(() => {
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

  const copyMetrics = useCallback(async () => {
    await navigator.clipboard.writeText(JSON.stringify(exportedMetrics, null, 2));
  }, [exportedMetrics]);

  return (
    <section className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
      <div className="overflow-hidden rounded-[2rem] border border-white/10 bg-black/40 shadow-2xl shadow-black/25">
        <div className="aspect-[9/14] w-full bg-linear-to-b from-slate-900 via-slate-950 to-black">
          {videoTrack ? (
            <VideoTrack
              trackRef={videoTrack}
              className="h-full w-full object-cover"
            />
          ) : (
            <div className="flex h-full items-center justify-center px-8 text-center text-sm text-slate-300">
              {state === "connecting"
                ? "Connecting Morgan to the room."
                : "Avatar video will appear here once LemonSlice joins the session."}
            </div>
          )}
        </div>
      </div>

      <aside className="flex flex-col gap-4 rounded-[2rem] border border-white/10 bg-slate-950/80 p-5 text-sm text-slate-200">
        <div>
          <p className="text-xs uppercase tracking-[0.24em] text-cyan-300/80">
            Agent State
          </p>
          <p className="mt-2 text-xl font-semibold capitalize text-white">{state}</p>
        </div>

        <div className="rounded-2xl bg-emerald-950/50 border border-emerald-500/30 p-4">
          <p className="text-xs uppercase tracking-[0.24em] text-emerald-300/90">
            {state === "listening" ? "Listening to you…" : "Heard you"}
          </p>
          {latestUserText ? (
            <p className="mt-3 font-medium text-emerald-100">{latestUserText}</p>
          ) : (
            <p className="mt-3 text-slate-400 italic">
              {state === "listening"
                ? "Say something — you’ll see your words here when we hear you."
                : "Your words will appear here once you speak."}
            </p>
          )}
          {recentUserTexts.length > 1 && (
            <ul className="mt-2 space-y-1 text-xs text-slate-400">
              {recentUserTexts.slice(0, -1).map((text, i) => (
                <li key={i} className="truncate">
                  “…{text}”
                </li>
              ))}
            </ul>
          )}
        </div>

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
            Morgan said
          </p>
          <p className="mt-3 leading-6 text-slate-100">{latestTranscript}</p>
        </div>

        <p className="rounded-xl bg-amber-950/40 border border-amber-500/20 px-3 py-2 text-xs text-amber-200/90">
          Tip: Say one short sentence, then wait 10–15 seconds for Morgan’s reply (OpenClaw can be slow). Check “Heard you” to confirm we’re receiving your speech.
        </p>

        <button
          type="button"
          onClick={copyMetrics}
          className="rounded-full border border-cyan-400/40 px-4 py-3 text-sm font-medium text-cyan-200 transition hover:border-cyan-300 hover:text-white"
        >
          Copy client metrics
        </button>
      </aside>
    </section>
  );
}

function SessionControls({ onReset }: { onReset: () => void }) {
  const room = useRoomContext();

  const disconnect = useCallback(async () => {
    await room.disconnect();
    onReset();
  }, [onReset, room]);

  return (
    <div className="mt-6 flex flex-wrap items-center gap-3">
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

export default function Room() {
  const [connection, setConnection] = useState<TokenResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [connectionRequestedAt, setConnectionRequestedAt] = useState<number | null>(null);
  const [roomConnectedAt, setRoomConnectedAt] = useState<number | null>(null);

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

  if (!connection) {
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
            onClick={handleConnect}
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
      className="grid gap-6"
    >
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

      <AgentTelemetry
        connectionRequestedAt={connectionRequestedAt}
        roomConnectedAt={roomConnectedAt}
      />
      <SessionControls onReset={reset} />
      <RoomAudioRenderer />
    </LiveKitRoom>
  );
}
