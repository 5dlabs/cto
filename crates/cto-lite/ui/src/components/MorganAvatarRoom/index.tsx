import '@livekit/components-styles'

import {
  LiveKitRoom,
  RoomAudioRenderer,
  VideoTrack,
  useAudioPlayback,
  useLocalParticipant,
  useRoomContext,
  useTranscriptions,
  useVoiceAssistant,
} from '@livekit/components-react'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  VoiceButton,
  type VoiceButtonState,
} from '@/components/ui/voice-button'
import { LiveWaveform } from '@/components/ui/live-waveform'

export type MorganAvatarState = {
  connectionState: 'idle' | 'connecting' | 'connected' | 'error'
  voiceState: string
  latestUserText: string
  latestAgentText: string
  audioTrackReady: boolean
  videoTrackReady: boolean
  roomName?: string
  identity?: string
  error?: string
  metrics?: Record<string, unknown>
  trackDebug?: Record<string, unknown>
}

type TokenResponse = {
  identity: string
  roomName: string
  serverUrl: string
  token: string
}

type MorganAvatarRoomProps = {
  autoConnect?: boolean
  compact?: boolean
  tokenEndpoint: string
  onStateChange?: (state: MorganAvatarState) => void
}

const DEFAULT_STATE: MorganAvatarState = {
  connectionState: 'idle',
  voiceState: 'idle',
  latestUserText: '',
  latestAgentText: '',
  audioTrackReady: false,
  videoTrackReady: false,
}

function emitState(
  onStateChange: MorganAvatarRoomProps['onStateChange'],
  payload: MorganAvatarState
) {
  onStateChange?.(payload)
}

function formatLatency(ms: number | null): string {
  if (ms === null) {
    return 'waiting'
  }

  return `${Math.round(ms)} ms`
}

function StatusPill({
  label,
  tone = 'neutral',
}: {
  label: string
  tone?: 'neutral' | 'emerald' | 'violet'
}) {
  const toneClass =
    tone === 'emerald'
      ? 'border-emerald-400/35 bg-emerald-400/12 text-emerald-100'
      : tone === 'violet'
        ? 'border-fuchsia-400/35 bg-fuchsia-400/12 text-fuchsia-100'
        : 'border-white/12 bg-white/10 text-slate-200'

  return (
    <span
      className={`rounded-full border px-3 py-1 text-[11px] font-medium uppercase tracking-[0.22em] ${toneClass}`}
    >
      {label}
    </span>
  )
}

function ActivityGlyph({
  active,
  tone,
}: {
  active: boolean
  tone: 'emerald' | 'violet'
}) {
  const activeClass =
    tone === 'emerald' ? 'bg-emerald-300/90' : 'bg-fuchsia-300/90'
  const idleClass = 'bg-white/10'

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
  )
}

function AvatarTelemetry({
  compact,
  connectionRequestedAt,
  roomConnectedAt,
  onStateChange,
}: {
  compact: boolean
  connectionRequestedAt: number | null
  roomConnectedAt: number | null
  onStateChange?: (state: MorganAvatarState) => void
}) {
  const room = useRoomContext()
  const { state, audioTrack, videoTrack, agentTranscriptions } = useVoiceAssistant()
  const allTranscriptions = useTranscriptions({ room })
  const [audioReadyAt, setAudioReadyAt] = useState<number | null>(null)
  const [videoReadyAt, setVideoReadyAt] = useState<number | null>(null)
  const [speakingAt, setSpeakingAt] = useState<number | null>(null)

  const userTranscriptions = useMemo(
    () =>
      allTranscriptions.filter(
        (transcription) =>
          transcription.participantInfo.identity === room.localParticipant.identity
      ),
    [allTranscriptions, room.localParticipant.identity]
  )
  const latestUserText =
    userTranscriptions.length > 0
      ? userTranscriptions[userTranscriptions.length - 1]?.text?.trim() ?? ''
      : ''
  const latestTranscript =
    agentTranscriptions.length > 0
      ? agentTranscriptions[agentTranscriptions.length - 1]?.text?.trim() ??
        'Morgan is ready when you are.'
      : 'Morgan is ready when you are.'

  useEffect(() => {
    if (audioTrack && audioReadyAt === null) {
      const frame = window.requestAnimationFrame(() => {
        setAudioReadyAt(performance.now())
      })
      return () => window.cancelAnimationFrame(frame)
    }
  }, [audioReadyAt, audioTrack])

  useEffect(() => {
    if (videoTrack && videoReadyAt === null) {
      const frame = window.requestAnimationFrame(() => {
        setVideoReadyAt(performance.now())
      })
      return () => window.cancelAnimationFrame(frame)
    }
  }, [videoReadyAt, videoTrack])

  useEffect(() => {
    if (state === 'speaking' && speakingAt === null) {
      const frame = window.requestAnimationFrame(() => {
        setSpeakingAt(performance.now())
      })
      return () => window.cancelAnimationFrame(frame)
    }
  }, [speakingAt, state])

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
        publishOnBehalf: participant.attributes['lk.publish_on_behalf'] ?? null,
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
    }
  }, [audioTrack, room, videoTrack])

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
    }
  }, [
    audioReadyAt,
    connectionRequestedAt,
    latestTranscript,
    roomConnectedAt,
    speakingAt,
    state,
    videoReadyAt,
  ])

  useEffect(() => {
    emitState(onStateChange, {
      connectionState: 'connected',
      voiceState: String(state),
      latestUserText,
      latestAgentText: latestTranscript,
      audioTrackReady: Boolean(audioTrack),
      videoTrackReady: Boolean(videoTrack),
      roomName: room.name || undefined,
      identity: room.localParticipant.identity || undefined,
      metrics,
      trackDebug,
    })
  }, [
    audioTrack,
    latestTranscript,
    latestUserText,
    metrics,
    onStateChange,
    room.localParticipant.identity,
    room.name,
    state,
    trackDebug,
    videoTrack,
  ])

  return (
    <section className="grid gap-5">
      <div className="relative overflow-hidden rounded-[2.2rem] border border-white/10 bg-black/30 shadow-[0_30px_120px_-48px_rgba(14,165,233,0.75)]">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,#155e75_0%,rgba(2,6,23,0.78)_34%,rgba(2,6,23,0.96)_100%)]" />
        <div className="absolute inset-x-[18%] top-[-14%] h-[32%] rounded-full bg-cyan-300/14 blur-[110px]" />
        <div className="absolute inset-x-[26%] bottom-[-10%] h-[26%] rounded-full bg-fuchsia-400/10 blur-[120px]" />

        <div className="absolute left-5 top-5 z-10 flex flex-wrap gap-2">
          <StatusPill label="Morgan" />
          <StatusPill
            label={String(state)}
            tone={state === 'speaking' ? 'violet' : state === 'listening' ? 'emerald' : 'neutral'}
          />
          <StatusPill
            label={audioTrack ? 'audio live' : 'audio pending'}
            tone={audioTrack ? 'emerald' : 'neutral'}
          />
          <StatusPill
            label={videoTrack ? 'video live' : 'video pending'}
            tone={videoTrack ? 'violet' : 'neutral'}
          />
        </div>

        <div className="absolute inset-x-0 bottom-0 z-10 flex items-end justify-between gap-4 p-5">
          <div className="max-w-[62%] rounded-[1.6rem] border border-white/10 bg-slate-950/70 px-4 py-3 backdrop-blur-md">
            <p className="text-[11px] uppercase tracking-[0.3em] text-cyan-100/75">
              {state === 'listening' ? 'Listening' : 'Live session'}
            </p>
            <div className="mt-3 flex items-center gap-3 text-sm text-slate-100">
              <ActivityGlyph
                active={state === 'listening' || Boolean(latestUserText)}
                tone="emerald"
              />
                <span>
                {latestUserText ||
                  (state === 'connecting'
                    ? 'Joining the room'
                    : 'Waiting for your first turn')}
              </span>
            </div>
            <div className="mt-4 overflow-hidden rounded-[1.15rem] border border-white/8 bg-black/30 px-3 py-2">
              <LiveWaveform
                active={false}
                processing={state !== 'idle'}
                mode="static"
                height={28}
                fadeEdges={false}
                barWidth={3}
                barGap={2}
                className="h-7 rounded-[0.9rem] bg-transparent"
              />
            </div>
          </div>

          <div className="flex flex-col items-end gap-3">
            <VoiceEntryButton compact />
            <div className="flex flex-wrap justify-end gap-2">
              <div className="rounded-full border border-white/10 bg-slate-950/70 px-4 py-2 text-xs text-slate-200 backdrop-blur-md">
                Audio{' '}
                {formatLatency(
                  connectionRequestedAt !== null && audioReadyAt
                    ? audioReadyAt - connectionRequestedAt
                    : null
                )}
              </div>
              <div className="rounded-full border border-white/10 bg-slate-950/70 px-4 py-2 text-xs text-slate-200 backdrop-blur-md">
                Room{' '}
                {formatLatency(
                  connectionRequestedAt !== null && roomConnectedAt
                    ? roomConnectedAt - connectionRequestedAt
                    : null
                )}
              </div>
            </div>
          </div>
        </div>

        <div className="aspect-[10/13] w-full bg-linear-to-b from-slate-900 via-slate-950 to-black">
          {videoTrack ? (
            <VideoTrack
              trackRef={videoTrack}
              className="h-full w-full scale-[1.06] object-cover object-[center_18%]"
            />
          ) : (
            <div className="flex h-full items-center justify-center px-8 text-center text-sm text-slate-300">
              {state === 'connecting'
                ? 'Connecting Morgan to the room.'
                : 'Morgan will appear here as soon as LemonSlice joins the session.'}
            </div>
          )}
        </div>
      </div>

      {!compact ? null : (
        <div className="rounded-[1.6rem] border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-300">
          Press <span className="font-medium text-white">Talk</span> once to enter the call,
          unlock audio, and arm your mic.
        </div>
      )}
    </section>
  )
}

function AssistantAudioRenderer() {
  return <RoomAudioRenderer />
}

function VoiceEntryButton({ compact }: { compact: boolean }) {
  const { canPlayAudio, startAudio } = useAudioPlayback()
  const { isMicrophoneEnabled, localParticipant, lastMicrophoneError } =
    useLocalParticipant()
  const [busy, setBusy] = useState(false)

  const buttonState: VoiceButtonState = busy
    ? 'processing'
    : lastMicrophoneError
      ? 'error'
      : isMicrophoneEnabled
        ? 'recording'
        : 'idle'

  const label = isMicrophoneEnabled ? 'In call' : 'Talk'
  const trailing = busy ? 'Joining' : isMicrophoneEnabled ? 'Live' : 'Press'

  return (
    <VoiceButton
      state={buttonState}
      label={label}
      trailing={trailing}
      onClick={() => {
        void (async () => {
          setBusy(true)
          try {
            if (!canPlayAudio) {
              await startAudio()
            }
            await localParticipant.setMicrophoneEnabled(!isMicrophoneEnabled)
          } finally {
            setBusy(false)
          }
        })()
      }}
      title={lastMicrophoneError?.message}
      variant="ghost"
      className={`border px-1.5 ${
        isMicrophoneEnabled
          ? 'border-emerald-400/35 bg-emerald-400/12 text-emerald-100 hover:bg-emerald-400/18'
          : 'border-cyan-300/35 bg-cyan-400/10 text-cyan-100 hover:bg-cyan-400/15'
      } ${compact ? 'h-10 min-w-[124px]' : 'h-11 min-w-[156px]'}`}
      waveformClassName={
        isMicrophoneEnabled
          ? 'border-emerald-300/30 bg-emerald-950/40'
          : 'border-cyan-300/20 bg-cyan-950/30'
      }
    />
  )
}

function SessionControls({
  compact,
  onReset,
}: {
  compact: boolean
  onReset: () => void
}) {
  const room = useRoomContext()

  const disconnect = useCallback(async () => {
    await room.disconnect()
    onReset()
  }, [onReset, room])

  if (compact) {
    return null
  }

  return (
    <div className="mt-6 flex flex-wrap items-center gap-3">
      <VoiceEntryButton compact={compact} />
      <button
        type="button"
        onClick={() => {
          void disconnect()
        }}
        className="rounded-full border border-white/10 bg-white/5 px-4 py-3 text-sm font-medium text-white transition hover:bg-white/10"
      >
        Disconnect
      </button>
    </div>
  )
}

export function MorganAvatarRoom({
  autoConnect = true,
  compact = true,
  tokenEndpoint,
  onStateChange,
}: MorganAvatarRoomProps) {
  const [connection, setConnection] = useState<TokenResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [connectionRequestedAt, setConnectionRequestedAt] = useState<number | null>(null)
  const [roomConnectedAt, setRoomConnectedAt] = useState<number | null>(null)
  const connectInFlightRef = useRef(false)

  const reset = useCallback(() => {
    connectInFlightRef.current = false
    setConnection(null)
    setError(null)
    setConnectionRequestedAt(null)
    setRoomConnectedAt(null)
    emitState(onStateChange, DEFAULT_STATE)
  }, [onStateChange])

  const connect = useCallback(async () => {
    setError(null)
    setConnectionRequestedAt(performance.now())

    const response = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    })

    if (!response.ok) {
      const payload = (await response.json().catch(() => null)) as { error?: string } | null
      throw new Error(payload?.error ?? 'Unable to create a LiveKit token.')
    }

    const payload = (await response.json()) as TokenResponse
    setConnection(payload)
  }, [tokenEndpoint])

  const failEmbeddedMedia = useCallback(
    (message: string) => {
      connectInFlightRef.current = false
      setConnection(null)
      setConnectionRequestedAt(null)
      setRoomConnectedAt(null)
      setError(message)
      emitState(onStateChange, {
        ...DEFAULT_STATE,
        connectionState: 'error',
        error: message,
      })
    },
    [onStateChange]
  )

  const handleConnect = useCallback(async () => {
    if (connectInFlightRef.current || connection) {
      return
    }

    connectInFlightRef.current = true

    try {
      await connect()
    } catch (cause) {
      connectInFlightRef.current = false
      const message =
        cause instanceof Error ? cause.message : 'Unable to create a LiveKit token.'
      setError(message)
      setConnectionRequestedAt(null)
      emitState(onStateChange, {
        ...DEFAULT_STATE,
        connectionState: 'error',
        error: message,
      })
    }
  }, [connect, connection, onStateChange])

  const handleRoomError = useCallback(
    (cause: Error) => {
      if (cause.name === 'NotAllowedError') {
        failEmbeddedMedia(
          'Microphone access is blocked in CTO. Allow microphone access for CTO and retry.'
        )
        return
      }

      failEmbeddedMedia(cause.message || 'Unable to keep the Morgan session connected.')
    },
    [failEmbeddedMedia]
  )

  const handleMediaDeviceFailure = useCallback(
    (failure?: unknown, kind?: MediaDeviceKind) => {
      const source = kind === 'audioinput' ? 'microphone' : kind ?? 'media device'
      const detail = failure ? ` (${String(failure)})` : ''
      failEmbeddedMedia(`Access to the ${source} was denied in CTO.${detail}`)
    },
    [failEmbeddedMedia]
  )

  useEffect(() => {
    emitState(onStateChange, {
      ...DEFAULT_STATE,
      connectionState: error
        ? 'error'
        : connection
          ? 'connected'
          : connectionRequestedAt !== null
            ? 'connecting'
            : 'idle',
      voiceState: connection ? 'connecting' : 'idle',
      roomName: connection?.roomName,
      identity: connection?.identity,
      error: error ?? undefined,
    })
  }, [connection, connectionRequestedAt, error, onStateChange])

  useEffect(() => {
    if (!autoConnect || connection || connectionRequestedAt !== null || error) {
      return
    }

    void handleConnect()
  }, [autoConnect, connection, connectionRequestedAt, error, handleConnect])

  if (!connection) {
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
                ? 'Morgan could not join the room.'
                : connectionRequestedAt !== null
                  ? 'Bringing Morgan online.'
                  : 'Preparing Morgan.'}
            </h2>
            <p className="mt-4 text-sm leading-7 text-slate-300">
              {error ??
                'Connecting LiveKit, LemonSlice, and the Morgan voice session inside CTO.'}
            </p>
          </div>
        </div>
      </section>
    )
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
      className="grid gap-5"
    >
      <AvatarTelemetry
        compact={compact}
        connectionRequestedAt={connectionRequestedAt}
        roomConnectedAt={roomConnectedAt}
        onStateChange={onStateChange}
      />
      <SessionControls compact={compact} onReset={reset} />
      <AssistantAudioRenderer />
    </LiveKitRoom>
  )
}
