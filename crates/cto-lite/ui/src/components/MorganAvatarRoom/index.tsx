import '@livekit/components-styles'

import {
  LiveKitRoom,
  RoomAudioRenderer,
  VideoTrack,
  useLocalParticipant,
  useRoomContext,
  useTranscriptions,
  useVoiceAssistant,
} from '@livekit/components-react'
import { Track } from 'livekit-client'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Mic } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import {
  VoiceButton,
  type VoiceButtonState,
} from '@/components/ui/voice-button'
import { LiveWaveform } from '@/components/ui/live-waveform'
import { cn } from '@/lib/utils'

export type MorganAvatarState = {
  connectionState: 'idle' | 'connecting' | 'connected' | 'error'
  callActive?: boolean
  voiceState: string
  latestUserText: string
  latestAgentText: string
  microphoneEnabled: boolean
  audioTrackReady: boolean
  videoTrackReady: boolean
  roomName?: string
  identity?: string
  error?: string
  metrics?: Record<string, unknown>
  trackDebug?: Record<string, unknown>
}

type DebugEvent = {
  at: string
  type: string
  detail: string
}

type DebugTranscript = {
  at: string
  identity: string
  source: 'user' | 'agent'
  text: string
  final: boolean | null
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
  roomName?: string
  mediaMode?: 'video' | 'voice'
  onStateChange?: (state: MorganAvatarState) => void
}

const DEFAULT_STATE: MorganAvatarState = {
  connectionState: 'idle',
  callActive: false,
  voiceState: 'idle',
  latestUserText: '',
  latestAgentText: '',
  microphoneEnabled: false,
  audioTrackReady: false,
  videoTrackReady: false,
}

function emitState(
  onStateChange: MorganAvatarRoomProps['onStateChange'],
  payload: MorganAvatarState
) {
  onStateChange?.(payload)
}

function pushDebugEvent(
  current: DebugEvent[],
  next: DebugEvent
): DebugEvent[] {
  return [...current, next].slice(-12)
}

function isoNow(): string {
  return new Date().toISOString()
}

type StageTone = 'neutral' | 'emerald' | 'violet' | 'cyan'

type StageChip = {
  label: string
  tone: StageTone
}

type StageViewModel = {
  callHeadline: string
  identityChip: StageChip
  voiceChip: StageChip
  roomChip: StageChip
  audioChip: StageChip
  videoChip: StageChip
  transcriptEyebrow: string
  transcriptText: string
  transcriptTone: 'emerald' | 'violet'
  placeholderCopy: string
}

function createStageViewModel({
  callActive,
  voiceState,
  latestUserText,
  latestAgentText,
  audioTrackReady,
  videoTrackReady,
}: Pick<
  MorganAvatarState,
  | 'callActive'
  | 'voiceState'
  | 'latestUserText'
  | 'latestAgentText'
  | 'audioTrackReady'
  | 'videoTrackReady'
>): StageViewModel {
  const normalizedVoiceState = voiceState.toLowerCase()
  if (!callActive) {
    return {
      callHeadline: 'Off call',
      identityChip: { label: 'Morgan', tone: 'cyan' },
      voiceChip: { label: 'Standby', tone: 'neutral' },
      roomChip: { label: 'Call idle', tone: 'neutral' },
      audioChip: { label: 'Mic off', tone: 'neutral' },
      videoChip: { label: 'Stage idle', tone: 'neutral' },
      transcriptEyebrow: 'Ready',
      transcriptText:
        'Start a call when you want Morgan live. While off call, he stays disconnected and does not listen.',
      transcriptTone: 'violet',
      placeholderCopy: 'Morgan stays off call until you start a session.',
    }
  }

  const voiceChip: StageChip =
    normalizedVoiceState === 'speaking'
      ? { label: 'Speaking', tone: 'violet' }
      : normalizedVoiceState === 'listening'
        ? { label: 'Listening', tone: 'emerald' }
        : normalizedVoiceState === 'connecting' || normalizedVoiceState === 'initializing'
          ? { label: 'Joining', tone: 'neutral' }
          : normalizedVoiceState === 'thinking'
            ? { label: 'Thinking', tone: 'neutral' }
            : { label: 'Ready', tone: 'neutral' }

  const roomChip: StageChip =
    normalizedVoiceState === 'connecting' || normalizedVoiceState === 'initializing'
      ? { label: 'Room joining', tone: 'neutral' }
      : { label: 'Room live', tone: 'violet' }

  const audioChip: StageChip = audioTrackReady
    ? { label: 'Audio live', tone: 'emerald' }
    : { label: 'Audio pending', tone: 'neutral' }

  const videoChip: StageChip = videoTrackReady
    ? { label: 'Video live', tone: 'violet' }
    : { label: 'Video pending', tone: 'neutral' }

  if (latestUserText) {
    return {
      callHeadline: 'In call',
      identityChip: { label: 'Morgan', tone: 'cyan' },
      voiceChip,
      roomChip,
      audioChip,
      videoChip,
      transcriptEyebrow: 'Heard',
      transcriptText: latestUserText,
      transcriptTone: 'emerald',
      placeholderCopy: 'Morgan will appear here once LemonSlice publishes video.',
    }
  }

  if (latestAgentText && latestAgentText !== 'Morgan is ready when you are.') {
    return {
      callHeadline: 'In call',
      identityChip: { label: 'Morgan', tone: 'cyan' },
      voiceChip,
      roomChip,
      audioChip,
      videoChip,
      transcriptEyebrow: 'Morgan',
      transcriptText: latestAgentText,
      transcriptTone: 'violet',
      placeholderCopy: 'Morgan will appear here once LemonSlice publishes video.',
    }
  }

  return {
    callHeadline: 'In call',
    identityChip: { label: 'Morgan', tone: 'cyan' },
    voiceChip,
    roomChip,
    audioChip,
    videoChip,
      transcriptEyebrow:
        normalizedVoiceState === 'connecting' || normalizedVoiceState === 'initializing'
          ? 'Joining'
        : normalizedVoiceState === 'listening'
          ? 'Listening'
          : 'Ready',
    transcriptText:
      normalizedVoiceState === 'connecting' || normalizedVoiceState === 'initializing'
        ? 'Bringing Morgan into the room.'
        : normalizedVoiceState === 'listening'
          ? 'The call is live and Morgan is listening.'
          : 'The call is live. Unmute if you want Morgan to listen.',
    transcriptTone:
      normalizedVoiceState === 'listening' ? 'emerald' : 'violet',
    placeholderCopy: 'Morgan will appear here once LemonSlice publishes video.',
  }
}

function StatusPill({
  label,
  tone = 'neutral',
}: {
  label: string
  tone?: StageTone
}) {
  const toneClass =
    tone === 'cyan'
      ? 'border-cyan-300/30 bg-cyan-400/12 text-cyan-100'
      : tone === 'emerald'
        ? 'border-emerald-400/35 bg-emerald-400/12 text-emerald-100'
        : tone === 'violet'
          ? 'border-fuchsia-400/35 bg-fuchsia-400/12 text-fuchsia-100'
          : 'border-white/12 bg-slate-950/72 text-slate-200'

  return (
    <Badge
      variant="outline"
      className={cn(
        'rounded-full border px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.24em] shadow-[0_10px_30px_-24px_rgba(15,23,42,0.95)] backdrop-blur-xl sm:text-[11px]',
        toneClass
      )}
    >
      {label}
    </Badge>
  )
}

function StageShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="relative overflow-hidden rounded-[2.2rem] border border-white/10 bg-slate-950/85 shadow-[0_30px_110px_-62px_rgba(34,211,238,0.52)]">
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,#12324a_0%,rgba(3,7,18,0.44)_28%,rgba(2,6,23,0.96)_100%)]" />
      <div className="absolute inset-x-[22%] top-[-10%] h-[18%] rounded-full bg-cyan-300/10 blur-[92px]" />
      <div className="relative z-[1] h-[min(58vh,42rem)] min-h-[18rem] w-full bg-linear-to-b from-slate-900 via-slate-950 to-black sm:h-[min(62vh,46rem)] sm:min-h-[22rem]">
        {children}
      </div>
    </div>
  )
}

function StageStateBadge({
  callActive,
  voiceState,
}: {
  callActive: boolean
  voiceState: string
}) {
  const normalized = voiceState.toLowerCase()
  const chip = !callActive
    ? { label: 'Off call', tone: 'neutral' as StageTone }
    : normalized === 'speaking'
      ? { label: 'Speaking', tone: 'violet' as StageTone }
      : normalized === 'listening'
        ? { label: 'Listening', tone: 'emerald' as StageTone }
        : normalized === 'connecting' || normalized === 'initializing'
          ? { label: 'Joining', tone: 'neutral' as StageTone }
          : normalized === 'thinking'
            ? { label: 'Thinking', tone: 'neutral' as StageTone }
            : { label: 'Ready', tone: 'cyan' as StageTone }

  return (
    <div className="absolute left-4 top-4 z-20 sm:left-5 sm:top-5">
      <StatusPill label={chip.label} tone={chip.tone} />
    </div>
  )
}

function StageControlDock({
  callActive,
  hasVideo,
  mediaMode,
  voiceState,
  onEndCall,
  compact,
}: {
  callActive: boolean
  hasVideo: boolean
  mediaMode: 'video' | 'voice'
  voiceState: string
  onEndCall: () => void
  compact: boolean
}) {
  const waitingLabel = !callActive
    ? mediaMode === 'voice'
      ? 'Start a call when you want Morgan live.'
      : 'Start a call when you want Morgan on screen.'
    : voiceState === 'connecting' || voiceState === 'initializing'
      ? 'Joining the room...'
      : voiceState === 'speaking'
        ? 'Morgan is speaking'
        : 'Mic live'

  return (
    <div className="absolute inset-x-4 bottom-4 z-20 flex flex-col items-center gap-3 sm:inset-x-5 sm:bottom-5">
      <div className="rounded-full border border-white/10 bg-slate-950/78 px-4 py-2 text-[11px] uppercase tracking-[0.24em] text-slate-200 shadow-[0_18px_48px_-32px_rgba(15,23,42,0.92)] backdrop-blur-xl">
        {waitingLabel}
      </div>
      <div className="flex flex-wrap items-center justify-center gap-3 rounded-[1.6rem] border border-white/10 bg-slate-950/82 px-3 py-3 shadow-[0_26px_70px_-40px_rgba(15,23,42,0.98)] backdrop-blur-xl sm:px-4">
        {callActive ? <CallMicButton compact={compact} /> : null}
        <CallSessionButton
          compact={compact}
          active={callActive}
          onClick={onEndCall}
        />
      </div>
      {callActive && mediaMode === 'video' && !hasVideo ? (
        <div className="rounded-full border border-white/10 bg-black/35 px-4 py-2 text-xs text-slate-300 backdrop-blur-xl">
          Morgan camera is warming up
        </div>
      ) : null}
    </div>
  )
}

function AvatarTelemetry({
  compact,
  onEndCall,
  connectionRequestedAt,
  roomConnectedAt,
  mediaMode,
  onStateChange,
}: {
  compact: boolean
  onEndCall: () => void
  connectionRequestedAt: number | null
  roomConnectedAt: number | null
  mediaMode: 'video' | 'voice'
  onStateChange?: (state: MorganAvatarState) => void
}) {
  const room = useRoomContext()
  const { state, audioTrack, videoTrack, agentTranscriptions } = useVoiceAssistant()
  const { isMicrophoneEnabled } = useLocalParticipant()
  const allTranscriptions = useTranscriptions({ room })
  const [audioReadyAt, setAudioReadyAt] = useState<number | null>(null)
  const [videoReadyAt, setVideoReadyAt] = useState<number | null>(null)
  const [speakingAt, setSpeakingAt] = useState<number | null>(null)
  const [debugEvents, setDebugEvents] = useState<DebugEvent[]>([])
  const previousVoiceStateRef = useRef<string | null>(null)
  const previousMicEnabledRef = useRef<boolean | null>(null)
  const previousRoomStateRef = useRef<string | null>(null)

  const latestUserText = useMemo(() => {
    const preferredIdentity = room.localParticipant.identity
    const userTranscript = [...allTranscriptions]
      .reverse()
      .find((transcription) => {
        const identity = transcription.participantInfo.identity ?? ''
        return identity === preferredIdentity || identity.startsWith('user-')
      })

    return userTranscript?.text?.trim() ?? ''
  }, [allTranscriptions, room.localParticipant.identity])
  const latestTranscript =
    agentTranscriptions.length > 0
      ? agentTranscriptions[agentTranscriptions.length - 1]?.text?.trim() ??
        'Morgan is ready when you are.'
      : 'Morgan is ready when you are.'
  const recentTranscriptEvents = useMemo<DebugTranscript[]>(() => {
    const userEvents = allTranscriptions.map((transcription) => ({
      at: isoNow(),
      identity: transcription.participantInfo.identity ?? 'unknown',
      source: 'user' as const,
      text: transcription.text?.trim() ?? '',
      final:
        typeof (transcription as { final?: boolean }).final === 'boolean'
          ? (transcription as { final?: boolean }).final ?? null
          : null,
    }))
    const agentEvents = agentTranscriptions.map((transcription) => ({
      at: isoNow(),
      identity: 'morgan-avatar',
      source: 'agent' as const,
      text: transcription.text?.trim() ?? '',
      final:
        typeof (transcription as { final?: boolean }).final === 'boolean'
          ? (transcription as { final?: boolean }).final ?? null
          : null,
    }))

    return [...userEvents, ...agentEvents]
      .filter((entry) => entry.text.length > 0)
      .slice(-10)
  }, [agentTranscriptions, allTranscriptions])

  useEffect(() => {
    const roomState = String(room.state)
    if (previousRoomStateRef.current === roomState) {
      return
    }

    previousRoomStateRef.current = roomState
    setDebugEvents((current) =>
      pushDebugEvent(current, {
        at: isoNow(),
        type: 'room-state',
        detail: roomState,
      })
    )
  }, [room.state])

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

  useEffect(() => {
    if (previousVoiceStateRef.current === String(state)) {
      return
    }

    previousVoiceStateRef.current = String(state)
    setDebugEvents((current) =>
      pushDebugEvent(current, {
        at: isoNow(),
        type: 'voice-state',
        detail: String(state),
      })
    )
  }, [state])

  useEffect(() => {
    if (previousMicEnabledRef.current === isMicrophoneEnabled) {
      return
    }

    previousMicEnabledRef.current = isMicrophoneEnabled
    setDebugEvents((current) =>
      pushDebugEvent(current, {
        at: isoNow(),
        type: 'mic-publication',
        detail: isMicrophoneEnabled ? 'enabled' : 'disabled',
      })
    )
  }, [isMicrophoneEnabled])

  useEffect(() => {
    if (!latestUserText) {
      return
    }

    setDebugEvents((current) =>
      pushDebugEvent(current, {
        at: isoNow(),
        type: 'user-transcript',
        detail: latestUserText,
      })
    )
  }, [latestUserText])

  useEffect(() => {
    if (!latestTranscript || latestTranscript === 'Morgan is ready when you are.') {
      return
    }

    setDebugEvents((current) =>
      pushDebugEvent(current, {
        at: isoNow(),
        type: 'agent-transcript',
        detail: latestTranscript,
      })
    )
  }, [latestTranscript])

  const trackDebug = useMemo(() => {
    return {
      roomState: String(room.state),
      microphoneEnabled: isMicrophoneEnabled,
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
      localMicrophoneTrack: room.localParticipant.getTrackPublication(Track.Source.Microphone)
        ? {
            sid:
              room.localParticipant.getTrackPublication(Track.Source.Microphone)?.trackSid ?? null,
            source:
              room.localParticipant.getTrackPublication(Track.Source.Microphone)?.source ?? null,
            muted:
              room.localParticipant.getTrackPublication(Track.Source.Microphone)?.isMuted ?? null,
            enabled:
              room.localParticipant.getTrackPublication(Track.Source.Microphone)?.isEnabled ?? null,
          }
        : null,
      recentTranscripts: recentTranscriptEvents,
      recentEvents: debugEvents,
    }
  }, [audioTrack, debugEvents, isMicrophoneEnabled, recentTranscriptEvents, room, videoTrack])

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
      roomState: String(room.state),
      microphoneEnabled: isMicrophoneEnabled,
      latestTranscript,
    }
  }, [
    audioReadyAt,
    connectionRequestedAt,
    isMicrophoneEnabled,
    latestTranscript,
    room.state,
    roomConnectedAt,
    speakingAt,
    state,
    videoReadyAt,
  ])
  const stageView = useMemo(
    () =>
      createStageViewModel({
        callActive: true,
        voiceState: String(state),
        latestUserText,
        latestAgentText: latestTranscript,
        audioTrackReady: Boolean(audioTrack),
        videoTrackReady: Boolean(videoTrack),
      }),
    [audioTrack, latestTranscript, latestUserText, state, videoTrack]
  )
  const voiceStage = (
    <div className="flex h-full items-center justify-center px-8">
      <div className="w-full max-w-[27rem] rounded-[2rem] border border-white/10 bg-slate-950/72 p-6 shadow-[0_24px_70px_-42px_rgba(34,211,238,0.4)] backdrop-blur-xl">
        <div className="mx-auto flex size-24 items-center justify-center rounded-full border border-cyan-300/20 bg-cyan-400/10 shadow-[0_0_0_1px_rgba(255,255,255,0.04),0_20px_80px_-40px_rgba(34,211,238,0.75)]">
          <div className="size-16 rounded-full bg-linear-to-br from-cyan-200/90 via-cyan-300/70 to-fuchsia-300/55 blur-[1px]" />
        </div>
        <p className="mt-6 text-center text-[11px] uppercase tracking-[0.34em] text-cyan-100/70">
          Voice channel
        </p>
        <p className="mt-3 text-center text-sm leading-7 text-slate-300">
          {stageView.transcriptText}
        </p>
        <div className="mt-5 overflow-hidden rounded-[1.2rem] border border-white/8 bg-black/30 px-3 py-3">
          <LiveWaveform
            active={String(state) === 'listening' || isMicrophoneEnabled}
            processing={String(state) === 'thinking' || String(state) === 'connecting'}
            mode="static"
            height={46}
            fadeEdges={false}
            barWidth={4}
            barGap={2}
            className="h-11 rounded-[0.9rem] bg-transparent"
          />
        </div>
      </div>
    </div>
  )

  useEffect(() => {
    emitState(onStateChange, {
      connectionState: 'connected',
      callActive: true,
      voiceState: String(state),
      latestUserText,
      latestAgentText: latestTranscript,
      microphoneEnabled: isMicrophoneEnabled,
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
    isMicrophoneEnabled,
    onStateChange,
    room.localParticipant.identity,
    room.name,
    state,
    trackDebug,
    videoTrack,
  ])

  return (
    <section className="grid h-full">
      <StageShell>
        <StageStateBadge callActive voiceState={String(state)} />
        <div className="h-full w-full">
          {mediaMode === 'voice' ? (
            voiceStage
          ) : videoTrack ? (
            <div className="morgan-stage-video relative h-full w-full overflow-hidden">
              <div className="absolute inset-0 bg-radial-[circle_at_center] from-cyan-400/10 via-transparent to-transparent" />
              <VideoTrack
                trackRef={videoTrack}
                className="h-full w-full"
              />
            </div>
          ) : (
            <div className="flex h-full items-center justify-center px-8 text-center text-sm text-slate-300">
              {stageView.placeholderCopy}
            </div>
          )}
        </div>
        <StageControlDock
          callActive
          hasVideo={Boolean(videoTrack)}
          mediaMode={mediaMode}
          voiceState={String(state)}
          compact={compact}
          onEndCall={() => {
            void room.disconnect()
            onEndCall()
          }}
        />
      </StageShell>
    </section>
  )
}

function AssistantAudioRenderer() {
  return <RoomAudioRenderer />
}

function AutoEnableMic() {
  const room = useRoomContext()
  const { isMicrophoneEnabled } = useLocalParticipant()
  const attemptedRef = useRef(false)

  useEffect(() => {
    if (String(room.state).toLowerCase() !== 'connected') {
      attemptedRef.current = false
      return
    }

    if (attemptedRef.current || isMicrophoneEnabled) {
      return
    }

    attemptedRef.current = true
    void room.localParticipant.setMicrophoneEnabled(true).catch((error) => {
      console.error('Auto-enable microphone error', error)
      attemptedRef.current = false
    })
  }, [isMicrophoneEnabled, room])

  return null
}

function CallMicButton({ compact }: { compact: boolean }) {
  const room = useRoomContext()
  const { isMicrophoneEnabled, lastMicrophoneError } = useLocalParticipant()
  const [pending, setPending] = useState(false)

  const setMic = useCallback(
    async (enabled: boolean) => {
      setPending(true)
      try {
        await room.localParticipant.setMicrophoneEnabled(enabled)
      } catch (error) {
        console.error('Call microphone toggle error', error)
      } finally {
        setPending(false)
      }
    },
    [room]
  )

  const state: VoiceButtonState = pending
    ? 'processing'
    : lastMicrophoneError
      ? 'error'
      : isMicrophoneEnabled
        ? 'recording'
        : 'idle'

  return (
    <VoiceButton
      state={state}
      label={isMicrophoneEnabled ? 'Mute' : 'Unmute'}
      trailing={pending ? '...' : isMicrophoneEnabled ? 'Live' : 'Muted'}
      title={lastMicrophoneError?.message}
      variant="ghost"
      className={`border px-2 ${
        isMicrophoneEnabled
          ? 'border-cyan-300/40 bg-cyan-400/14 text-cyan-50 hover:bg-cyan-400/18'
          : 'border-white/12 bg-slate-950/72 text-slate-100 hover:bg-white/10'
      } ${compact ? 'h-12 min-w-[188px]' : 'h-14 min-w-[220px]'}`}
      waveformClassName={
        isMicrophoneEnabled
          ? 'border-cyan-300/30 bg-cyan-950/40'
          : 'border-white/12 bg-slate-950/40'
      }
      onClick={(event) => {
        event.preventDefault()
        void setMic(!isMicrophoneEnabled)
      }}
    />
  )
}

function CallSessionButton({
  compact,
  active,
  pending = false,
  error = false,
  onClick,
}: {
  compact: boolean
  active: boolean
  pending?: boolean
  error?: boolean
  onClick: () => void
}) {
  const resolvedState: VoiceButtonState = pending
    ? 'processing'
    : error
      ? 'error'
      : active
        ? 'recording'
        : 'idle'
  const label = active ? 'End call' : 'Start call'
  const trailing = pending ? 'Joining' : active ? 'Live' : 'Connect'
  return (
    <VoiceButton
      state={resolvedState}
      label={label}
      trailing={trailing}
      onClick={onClick}
      variant="ghost"
      className={`border px-2 ${
        active
          ? 'border-emerald-400/35 bg-emerald-400/12 text-emerald-100 hover:bg-emerald-400/18'
          : 'border-cyan-300/35 bg-cyan-400/10 text-cyan-100 hover:bg-cyan-400/15'
      } ${compact ? 'h-12 min-w-[168px]' : 'h-14 min-w-[188px]'}`}
      waveformClassName={
        active
          ? 'border-emerald-300/30 bg-emerald-950/40'
          : 'border-cyan-300/20 bg-cyan-950/30'
      }
    />
  )
}

export function MorganAvatarRoom({
  autoConnect = false,
  compact = true,
  tokenEndpoint,
  roomName,
  mediaMode = 'video',
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
      body: JSON.stringify(roomName ? { roomName } : {}),
    })

    if (!response.ok) {
      const payload = (await response.json().catch(() => null)) as { error?: string } | null
      throw new Error(payload?.error ?? 'Unable to create a LiveKit token.')
    }

    const payload = (await response.json()) as TokenResponse
    setConnection(payload)
  }, [roomName, tokenEndpoint])

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
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,#12324a_0%,rgba(2,6,23,0.82)_30%,rgba(2,6,23,0.98)_100%)]" />
        <div className="absolute inset-0 backdrop-blur-[2px]" />
        <div className="relative flex min-h-[24rem] items-center justify-center px-6 py-10 text-center sm:min-h-[28rem] lg:px-8 lg:py-12">
          <div className="max-w-md">
            <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-cyan-300/30 bg-cyan-400/10">
              <Mic className="h-7 w-7 text-cyan-100" />
            </div>
            <p className="mt-8 text-[11px] uppercase tracking-[0.34em] text-cyan-200/70">
              {mediaMode === 'voice' ? 'Morgan voice' : 'Morgan avatar'}
            </p>
            <h2 className="mt-4 text-3xl font-semibold tracking-tight text-white">
              {error
                ? 'Morgan could not join the room.'
                : connectionRequestedAt !== null
                  ? 'Starting the call.'
                  : 'Morgan is off call.'}
            </h2>
            <p className="mt-4 text-sm leading-7 text-slate-300">
              {error ??
                (connectionRequestedAt !== null
                  ? 'Connecting the live Morgan session.'
                  : 'Start a call when you want Morgan live. While off call, the mic stays off and the room is disconnected.')}
            </p>
            <div className="mt-8 flex justify-center">
              <CallSessionButton
                compact={compact}
                active={false}
                pending={connectionRequestedAt !== null}
                error={Boolean(error)}
                onClick={() => {
                  void handleConnect()
                }}
              />
            </div>
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
      <AutoEnableMic />
      <AvatarTelemetry
        compact={compact}
        onEndCall={() => {
          void reset()
        }}
        connectionRequestedAt={connectionRequestedAt}
        roomConnectedAt={roomConnectedAt}
        mediaMode={mediaMode}
        onStateChange={onStateChange}
      />
      <AssistantAudioRenderer />
    </LiveKitRoom>
  )
}
