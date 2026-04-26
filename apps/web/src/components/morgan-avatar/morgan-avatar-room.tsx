'use client';

import { AudioTrack, LiveKitRoom, VideoTrack, useVoiceAssistant } from '@livekit/components-react';
import Image from 'next/image';
import { useCallback, useEffect, useRef, useState } from 'react';

type TokenResponse = {
  identity: string;
  participantName?: string;
  roomName: string;
  serverUrl: string;
  token: string;
};

type MorganAvatarRoomProps = {
  autoConnect?: boolean;
  embedded?: boolean;
};

function MorganMediaStage({ embedded }: { embedded: boolean }) {
  const { state, audioTrack, videoTrack, agentTranscriptions } = useVoiceAssistant();
  const latestTranscript = agentTranscriptions.at(-1)?.text?.trim();
  const stateLabel = state ? String(state).replace(/_/g, ' ') : 'waiting';

  return (
    <section className={embedded ? 'h-full min-h-0 overflow-hidden' : 'grid gap-5'}>
      <div className="relative flex min-h-[520px] overflow-hidden rounded-[2rem] border border-cyan-300/15 bg-slate-950 shadow-[0_28px_120px_-52px_rgba(14,165,233,0.75)]">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,#155e75_0%,rgba(2,6,23,0.82)_32%,rgba(2,6,23,0.98)_100%)]" />
        {videoTrack ? (
          <VideoTrack trackRef={videoTrack} className="relative z-10 h-full w-full object-cover" />
        ) : (
          <div className="relative z-10 flex h-full w-full flex-col items-center justify-center px-8 text-center">
            <Image
              src="/agents/morgan-avatar-512.png"
              alt="Morgan"
              width={176}
              height={176}
              className="h-44 w-44 rounded-full border border-cyan-300/25 object-cover shadow-[0_0_80px_rgba(34,211,238,0.22)]"
            />
            <p className="mt-8 text-[11px] tracking-[0.34em] text-cyan-200/70 uppercase">
              Morgan avatar
            </p>
            <h2 className="mt-4 text-3xl font-semibold tracking-tight text-white">
              Morgan is joining the room.
            </h2>
            <p className="mt-4 max-w-md text-sm leading-7 text-slate-300">
              The self-hosted LiveKit room is ready; the Morgan avatar worker will publish video and
              audio here.
            </p>
          </div>
        )}
        <div className="absolute inset-x-4 bottom-4 z-20 rounded-2xl border border-white/10 bg-slate-950/80 px-4 py-3 text-sm text-slate-100 backdrop-blur">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <span className="rounded-full border border-cyan-300/25 bg-cyan-300/10 px-3 py-1 text-[11px] tracking-[0.22em] text-cyan-100 uppercase">
              {stateLabel}
            </span>
            <span className="text-xs text-slate-300">Self-hosted LiveKit</span>
          </div>
          {latestTranscript ? (
            <p className="mt-3 leading-6 text-slate-100">{latestTranscript}</p>
          ) : null}
        </div>
      </div>
      {audioTrack ? <AudioTrack trackRef={audioTrack} /> : null}
    </section>
  );
}

export function MorganAvatarRoom({ autoConnect = false, embedded = false }: MorganAvatarRoomProps) {
  const [connection, setConnection] = useState<TokenResponse | null>(null);
  const [connectionRequested, setConnectionRequested] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const autoConnectTriggeredRef = useRef(false);

  const reset = useCallback(() => {
    setConnection(null);
    setConnectionRequested(false);
    setError(null);
  }, []);

  const failConnection = useCallback((message: string) => {
    setConnection(null);
    setConnectionRequested(false);
    setError(message);
  }, []);

  const connect = useCallback(async () => {
    setError(null);
    setConnectionRequested(true);

    try {
      const response = await fetch('/api/avatar/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ participantName: 'Morgan visitor' }),
      });

      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as { error?: string } | null;
        throw new Error(payload?.error ?? 'Unable to create a LiveKit token.');
      }

      setConnection((await response.json()) as TokenResponse);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : 'Unable to create a LiveKit token.';
      setError(message);
      setConnectionRequested(false);
    }
  }, []);

  useEffect(() => {
    if (!autoConnect || autoConnectTriggeredRef.current || connection || connectionRequested) {
      return;
    }

    autoConnectTriggeredRef.current = true;
    const timer = window.setTimeout(() => void connect(), 0);
    return () => window.clearTimeout(timer);
  }, [autoConnect, connect, connection, connectionRequested]);

  if (!connection) {
    return (
      <section
        className={
          embedded
            ? 'relative h-full min-h-0 overflow-hidden bg-slate-950/80'
            : 'min-h-screen bg-slate-950 px-6 py-16 text-white'
        }
      >
        <div
          className={
            embedded
              ? 'absolute inset-0 bg-[radial-gradient(circle_at_top,#155e75_0%,rgba(2,6,23,0.82)_30%,rgba(2,6,23,0.98)_100%)]'
              : ''
          }
        />
        <div
          className={
            embedded
              ? 'relative flex h-full items-center justify-center px-8 text-center'
              : 'mx-auto flex max-w-4xl items-center justify-center text-center'
          }
        >
          <div className="max-w-md">
            <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-cyan-300/30 bg-cyan-400/10">
              <div className="h-10 w-10 rounded-full bg-cyan-300/80 blur-[1px]" />
            </div>
            <p className="mt-8 text-[11px] tracking-[0.34em] text-cyan-200/70 uppercase">
              Morgan avatar
            </p>
            <h1 className="mt-4 text-3xl font-semibold tracking-tight text-white">
              {error
                ? 'Morgan could not join the room.'
                : connectionRequested
                  ? 'Bringing Morgan online.'
                  : 'Talk to Morgan live.'}
            </h1>
            <p className="mt-4 text-sm leading-7 text-slate-300">
              {error ??
                'This uses the production self-hosted LiveKit route and dispatches the Morgan avatar worker in the CTO cluster.'}
            </p>
            <button
              type="button"
              onClick={() => void connect()}
              disabled={connectionRequested}
              className="mt-8 rounded-full border border-cyan-300/35 bg-cyan-400/10 px-5 py-3 text-sm font-medium text-cyan-100 transition hover:bg-cyan-400/15 disabled:cursor-wait disabled:opacity-70"
            >
              {connectionRequested
                ? 'Connecting…'
                : error
                  ? 'Retry connection'
                  : 'Start LiveKit session'}
            </button>
          </div>
        </div>
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
      onDisconnected={reset}
      onError={(cause) =>
        failConnection(cause.message || 'Unable to keep the Morgan session connected.')
      }
      onMediaDeviceFailure={(failure, kind) => {
        const source = kind === 'audioinput' ? 'microphone' : (kind ?? 'media device');
        const detail = failure ? ` (${String(failure)})` : '';
        failConnection(`Access to the ${source} was denied. Allow it for CTO and retry.${detail}`);
      }}
      className={
        embedded ? 'h-full min-h-0 overflow-hidden' : 'min-h-screen bg-slate-950 p-6 text-white'
      }
    >
      <MorganMediaStage embedded={embedded} />
    </LiveKitRoom>
  );
}
