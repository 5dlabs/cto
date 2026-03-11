import { type ReactNode, useCallback, useEffect, useRef, useState } from 'react'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  AudioLines,
  CircleDot,
  ExternalLink,
  Link2,
  Loader2,
  Mic,
  MicOff,
  RefreshCw,
  ShieldAlert,
  Volume2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { BarVisualizer } from '@/components/ui/bar-visualizer'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import {
  MorganAvatarRoom,
  type MorganAvatarState,
} from '@/components/MorganAvatarRoom'
import * as tauri from '@/lib/tauri'

type PanelMode = 'live' | 'debug'

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

type TrackPublicationDebug = {
  participant?: string
  sid?: string | null
  name?: string | null
  source?: string | null
  kind?: string | null
  subscribed?: boolean | null
  muted?: boolean | null
  enabled?: boolean | null
}

type RemoteParticipantDebug = {
  identity: string
  kind?: string | null
  publishOnBehalf?: string | null
  tracks: TrackPublicationDebug[]
}

type AvatarTrackDebug = {
  roomState?: string
  microphoneEnabled?: boolean
  selectedAudioTrack?: TrackPublicationDebug | null
  selectedVideoTrack?: TrackPublicationDebug | null
  remoteParticipants?: RemoteParticipantDebug[]
  localMicrophoneTrack?: TrackPublicationDebug | null
  recentTranscripts?: DebugTranscript[]
  recentEvents?: DebugEvent[]
}

const DEFAULT_AVATAR_STATE: MorganAvatarState = {
  connectionState: 'idle',
  callActive: false,
  voiceState: 'idle',
  latestUserText: '',
  latestAgentText: '',
  microphoneEnabled: false,
  audioTrackReady: false,
  videoTrackReady: false,
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

function sameRecord(
  left: Record<string, unknown> | undefined,
  right: Record<string, unknown> | undefined
): boolean {
  return JSON.stringify(left ?? null) === JSON.stringify(right ?? null)
}

function sameAvatarState(left: MorganAvatarState, right: MorganAvatarState): boolean {
  return (
    left.connectionState === right.connectionState &&
    left.callActive === right.callActive &&
    left.voiceState === right.voiceState &&
    left.latestUserText === right.latestUserText &&
    left.latestAgentText === right.latestAgentText &&
    left.microphoneEnabled === right.microphoneEnabled &&
    left.audioTrackReady === right.audioTrackReady &&
    left.videoTrackReady === right.videoTrackReady &&
    left.roomName === right.roomName &&
    left.identity === right.identity &&
    left.error === right.error &&
    sameRecord(left.metrics, right.metrics) &&
    sameRecord(left.trackDebug, right.trackDebug)
  )
}

function normalizeTrackDebug(input: Record<string, unknown> | undefined): AvatarTrackDebug {
  return (input ?? {}) as AvatarTrackDebug
}

export function AgentsView() {
  const avatarBrowserUrl =
    ((import.meta as { env?: Record<string, string> }).env?.VITE_MORGAN_AVATAR_BROWSER_URL as string) ??
    'http://localhost:3000'
  const avatarTokenEndpoint =
    ((import.meta as { env?: Record<string, string> }).env?.VITE_MORGAN_AVATAR_TOKEN_ENDPOINT as string) ??
    '/avatar-api/token'
  const [avatarLoaded, setAvatarLoaded] = useState(false)
  const [avatarSessionKey, setAvatarSessionKey] = useState(0)
  const [panelMode, setPanelMode] = useState<PanelMode>('live')
  const [avatarState, setAvatarState] = useState<MorganAvatarState>(DEFAULT_AVATAR_STATE)
  const [bridgeStatus, setBridgeStatus] = useState<tauri.OpenClawBridgeStatus | null>(null)
  const [backendDiagnostics, setBackendDiagnostics] = useState<tauri.MorganDiagnostics | null>(null)
  const [bridgeBooting, setBridgeBooting] = useState(true)
  const [runtimeEnvironment, setRuntimeEnvironment] = useState<tauri.RuntimeEnvironment | null>(
    null
  )
  const [clusterStatus, setClusterStatus] = useState<tauri.ClusterInfo | null>(null)
  const [lastError, setLastError] = useState<string | null>(null)
  const [sharedContextValue, setSharedContextValue] = useState('')
  const [sharedContextStatus, setSharedContextStatus] = useState<string | null>(null)
  const [sharedContextSending, setSharedContextSending] = useState(false)
  const [micEnabled, setMicEnabled] = useState(false)
  const [micLevel, setMicLevel] = useState(0)
  const audioContextRef = useRef<AudioContext | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const animationRef = useRef<number | null>(null)

  useEffect(() => {
    let cancelled = false

    const syncLocalState = async () => {
      try {
        const [gateway, runtime, cluster] = await Promise.allSettled([
          tauri.openclawGetLocalBridgeStatus(),
          tauri.scanRuntimeEnvironment(),
          tauri.getClusterStatus(),
        ])
        const diagnostics = await tauri
          .openclawGetMorganDiagnostics()
          .catch(() => null)

        if (!cancelled) {
          if (gateway.status === 'fulfilled') {
            setBridgeStatus(gateway.value)
            setLastError(null)
          } else {
            setLastError(String(gateway.reason))
          }

          if (runtime.status === 'fulfilled') {
            setRuntimeEnvironment(runtime.value)
          }

          if (cluster.status === 'fulfilled') {
            setClusterStatus(cluster.value)
          }

          setBackendDiagnostics(diagnostics)
        }
      } finally {
        if (!cancelled) {
          setBridgeBooting(false)
        }
      }
    }

    void syncLocalState()
    const poll = window.setInterval(() => {
      void syncLocalState()
    }, 4000)

    return () => {
      cancelled = true
      window.clearInterval(poll)
    }
  }, [])

  useEffect(() => {
    return () => {
      if (animationRef.current) {
        window.cancelAnimationFrame(animationRef.current)
      }
      streamRef.current?.getTracks().forEach((track) => track.stop())
      audioContextRef.current?.close().catch(() => {})
    }
  }, [])

  const startMicMonitor = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      const context = new window.AudioContext()
      const analyser = context.createAnalyser()
      analyser.fftSize = 256
      const source = context.createMediaStreamSource(stream)
      source.connect(analyser)
      const data = new Uint8Array(analyser.frequencyBinCount)

      streamRef.current = stream
      audioContextRef.current = context
      setMicEnabled(true)

      const frame = () => {
        analyser.getByteFrequencyData(data)
        const average =
          data.reduce((sum, value) => sum + value, 0) / (data.length * 255)
        setMicLevel(clamp(average * 2.2, 0, 1))
        animationRef.current = window.requestAnimationFrame(frame)
      }

      animationRef.current = window.requestAnimationFrame(frame)
    } catch (error) {
      setLastError(`Microphone unavailable: ${String(error)}`)
    }
  }

  const stopMicMonitor = async () => {
    if (animationRef.current) {
      window.cancelAnimationFrame(animationRef.current)
      animationRef.current = null
    }
    streamRef.current?.getTracks().forEach((track) => track.stop())
    streamRef.current = null
    if (audioContextRef.current) {
      await audioContextRef.current.close().catch(() => {})
      audioContextRef.current = null
    }
    setMicLevel(0)
    setMicEnabled(false)
  }

  const reloadAvatar = () => {
    setAvatarLoaded(false)
    setAvatarState(DEFAULT_AVATAR_STATE)
    setAvatarSessionKey((value) => value + 1)
  }

  const startBridge = async () => {
    setBridgeBooting(true)
    setLastError(null)
    try {
      const status = await tauri.openclawStartLocalBridge()
      setBridgeStatus(status)
    } catch (error) {
      setLastError(String(error))
    } finally {
      setBridgeBooting(false)
    }
  }

  const stopBridge = async () => {
    try {
      const status = await tauri.openclawStopLocalBridge()
      setBridgeStatus(status)
    } catch (error) {
      setLastError(String(error))
    }
  }

  const handleAvatarStateChange = useCallback((nextState: MorganAvatarState) => {
    setAvatarLoaded(true)
    setAvatarState((current) =>
      sameAvatarState(current, nextState) ? current : nextState
    )
    if (nextState.error) {
      setLastError((current) =>
        current === nextState.error ? current : (nextState.error ?? null)
      )
    }
  }, [])

  const openAvatarInBrowser = async () => {
    await openUrl(avatarBrowserUrl)
  }

  const sendSharedContext = async () => {
    const roomName = avatarState.roomName?.trim()
    const content = sharedContextValue.trim()

    if (!roomName) {
      setSharedContextStatus('Join the call first so CTO can target the active Morgan room.')
      return
    }

    if (!content) {
      setSharedContextStatus('Paste a URL, brief, or note before sending shared context.')
      return
    }

    setSharedContextSending(true)
    setSharedContextStatus(null)

    try {
      const response = await tauri.openclawSendAvatarContext(roomName, content)
      setSharedContextStatus(
        response.content === 'CONTEXT_STORED'
          ? 'Shared with Morgan. Speak naturally and refer to “this link” or “this brief.”'
          : response.content
      )
      setSharedContextValue('')
    } catch (error) {
      setSharedContextStatus(`Morgan did not accept the shared context: ${String(error)}`)
    } finally {
      setSharedContextSending(false)
    }
  }

  const runningRuntime = runtimeEnvironment?.runtimes.find((runtime) => runtime.running)
  const installedRuntime = runtimeEnvironment?.runtimes.find((runtime) => runtime.installed)
  const runtimeSummary = runningRuntime
    ? `${runningRuntime.runtime} running`
    : installedRuntime
      ? `${installedRuntime.runtime} installed`
      : 'No local container runtime'
  const clusterSummary = clusterStatus?.running
    ? `${clusterStatus.name} running`
    : clusterStatus?.exists
      ? `${clusterStatus.name} created`
      : 'Cluster not created'
  const gatewaySummary = bridgeStatus?.connected
    ? 'Connected'
    : bridgeStatus?.running
      ? 'Port-forward fallback active'
      : 'Ingress offline'
  const callActive = Boolean(avatarState.callActive)
  const sessionTone =
    !callActive
      ? 'slate'
      : avatarState.voiceState === 'speaking'
      ? 'violet'
      : avatarState.voiceState === 'listening'
        ? 'emerald'
        : 'slate'
  const liveSessionTitle =
    !callActive
      ? 'Morgan is off call'
      : avatarState.voiceState === 'speaking'
      ? 'Morgan is responding'
      : avatarState.voiceState === 'listening'
        ? 'Morgan is listening'
        : avatarState.connectionState === 'connected'
          ? 'Morgan is ready'
          : runtimeEnvironment && !runtimeEnvironment.docker_available
            ? 'Docker is required'
            : clusterStatus && !clusterStatus.exists
              ? 'Local stack not created'
              : clusterStatus && !clusterStatus.running
                ? 'Local cluster offline'
                : bridgeBooting
                  ? 'Checking Morgan'
                  : 'Waiting for Morgan'
  const liveSessionDescription =
    callActive && avatarState.connectionState === 'connected'
      ? `${avatarState.roomName ?? 'morgan'} · ${avatarState.identity ?? 'guest'}`
      : avatarState.connectionState === 'idle'
        ? 'Start a call when you want Morgan live. Off call, he stays disconnected and does not listen.'
      : bridgeStatus?.connected
        ? 'The live stage is ready. Start a call on the stage when you want Morgan live.'
        : runtimeEnvironment && !runtimeEnvironment.docker_available
          ? 'Start Docker so CTO can keep the local stack online.'
          : clusterStatus && !clusterStatus.exists
            ? 'Bootstrap the local kind cluster and deploy Morgan.'
            : clusterStatus && !clusterStatus.running
              ? 'The cluster exists but is not reachable yet.'
              : 'The avatar surface will connect as soon as the local ingress is ready.'
  const sessionStatusLabel =
    !callActive
      ? 'off call'
      : avatarState.voiceState === 'speaking'
      ? 'speaking'
      : avatarState.voiceState === 'listening'
        ? 'listening'
        : avatarState.connectionState === 'connected'
          ? 'ready'
          : avatarState.connectionState === 'connecting'
          ? 'joining'
            : 'standby'
  const userSignalLevel = avatarState.latestUserText
    ? 0.82
    : callActive && avatarState.microphoneEnabled
      ? 0.22
      : 0
  const agentSignalLevel = avatarState.voiceState === 'speaking'
    ? 0.84
    : avatarState.latestAgentText &&
        avatarState.latestAgentText !== 'Morgan is ready when you are.'
      ? 0.34
      : 0
  const userSignalDetail = avatarState.latestUserText
    ? 'Transcript received'
    : callActive && avatarState.microphoneEnabled
      ? 'Push-to-talk live'
      : 'Off call'
  const agentSignalDetail = avatarState.voiceState === 'speaking'
    ? 'Speaking now'
    : !callActive
      ? 'Waiting for call start'
    : avatarState.latestAgentText &&
        avatarState.latestAgentText !== 'Morgan is ready when you are.'
      ? 'Latest reply ready'
      : 'Waiting for OpenClaw turn'
  const avatarDebug = normalizeTrackDebug(avatarState.trackDebug)
  const recentEvents = avatarDebug.recentEvents ?? []
  const recentTranscripts = avatarDebug.recentTranscripts ?? []
  const remoteParticipants = avatarDebug.remoteParticipants ?? []
  const sessionSnapshot = [
    {
      label: 'Connection',
      value: avatarState.connectionState,
    },
    {
      label: 'Voice state',
      value: avatarState.voiceState,
    },
    {
      label: 'Room state',
      value: avatarDebug.roomState ?? 'unknown',
    },
    {
      label: 'Room',
      value: avatarState.roomName ?? 'morgan',
    },
    {
      label: 'Identity',
      value: avatarState.identity ?? 'guest',
    },
    {
      label: 'Mic publication',
      value: avatarDebug.microphoneEnabled ? 'enabled' : 'disabled',
    },
    {
      label: 'Audio track',
      value: avatarState.audioTrackReady ? 'ready' : 'pending',
    },
    {
      label: 'Video track',
      value: avatarState.videoTrackReady ? 'ready' : 'pending',
    },
  ]
  const timingSnapshot = Object.entries(avatarState.metrics ?? {}).map(([label, rawValue]) => {
    const value =
      typeof rawValue === 'number'
        ? `${Math.round(rawValue)} ms`
        : rawValue === null || rawValue === undefined
          ? 'pending'
          : String(rawValue)

    return { label, value }
  })

  return (
    <div className="flex h-full flex-col overflow-hidden bg-[#090f1a] text-slate-100">
      <div className="border-b border-white/10 bg-[radial-gradient(circle_at_top,#10324f_0%,rgba(7,13,24,0.96)_52%,rgba(7,13,24,1)_100%)] px-6 py-4">
        <div className="flex flex-wrap items-center gap-3">
          <Avatar
            size="lg"
            className="ring-1 ring-cyan-300/20 shadow-[0_18px_44px_-28px_rgba(34,211,238,0.65)]"
          >
            <AvatarFallback className="bg-cyan-400/15 text-cyan-50">MO</AvatarFallback>
          </Avatar>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <StatusBadge label="Avatar" tone="cyan" />
              <h1 className="text-lg font-semibold tracking-tight">Morgan</h1>
            </div>
            <p className="mt-1 text-sm text-slate-300">
              Call-style surface for the local Morgan persona running in your private cluster.
            </p>
          </div>
        </div>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 p-4 lg:grid-cols-[minmax(0,1.2fr)_380px]">
        <section className="min-h-0">
          <div className="h-[min(78vh,760px)] min-h-[560px] w-full">
            <MorganAvatarRoom
              key={avatarSessionKey}
              compact
              autoConnect={false}
              tokenEndpoint={avatarTokenEndpoint}
              onStateChange={handleAvatarStateChange}
            />
          </div>
        </section>

        <aside className="min-h-0 overflow-hidden rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-4">
          <Tabs
            value={panelMode}
            onValueChange={(value) => setPanelMode(value as PanelMode)}
            className="flex h-full min-h-0 flex-col"
          >
            <div className="flex items-center justify-between gap-3">
              <div>
                <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">
                  Control deck
                </p>
                <p className="mt-1 text-sm text-slate-300">
                  Keep the live view clean, and push the operational noise into debug.
                </p>
              </div>
              <TabsList
                className="rounded-full border border-white/10 bg-white/[0.05]"
                variant="default"
              >
                <TabsTrigger value="live" className="rounded-full px-3 text-xs">
                  Live
                </TabsTrigger>
                <TabsTrigger value="debug" className="rounded-full px-3 text-xs">
                  Debug
                </TabsTrigger>
              </TabsList>
            </div>

            <Separator className="my-4 bg-white/8" />

            <TabsContent value="live" className="mt-0 min-h-0 flex-1 overflow-auto pr-1">
              <div className="space-y-4">
                <GlassCard
                  eyebrow="Session"
                  title={liveSessionTitle}
                  description={liveSessionDescription}
                >
                  <div className="grid gap-3">
                    <div className="rounded-[22px] border border-white/10 bg-black/20 p-4">
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <p className="text-[11px] uppercase tracking-[0.3em] text-cyan-100/70">
                            Current turn
                          </p>
                          <p className="mt-2 text-sm text-slate-200">
                            {!callActive
                              ? 'Morgan is standing by. Start a call on the stage when you want him live.'
                              : avatarState.voiceState === 'speaking'
                              ? 'Morgan is actively responding on the live stage.'
                              : avatarState.voiceState === 'listening'
                                ? 'Hold the talk button while you speak.'
                                : avatarState.connectionState === 'connected'
                                  ? 'The call is live. Hold to talk when you want Morgan to listen.'
                                  : 'The live stage will settle as soon as the local stack is ready.'}
                          </p>
                        </div>
                        <StatusBadge label={sessionStatusLabel} tone={sessionTone} />
                      </div>
                      <div className="mt-4 grid grid-cols-2 gap-3">
                        <SignalTile
                          label="Room"
                          value={avatarState.roomName ?? 'morgan'}
                          tone="slate"
                        />
                        <SignalTile
                          label="Identity"
                          value={avatarState.identity ?? 'guest'}
                          tone="slate"
                        />
                        <SignalTile
                          label="Stack"
                          value={bridgeStatus?.connected ? 'Local cluster live' : gatewaySummary}
                          tone={bridgeStatus?.connected ? 'emerald' : 'slate'}
                        />
                        <SignalTile
                          label="Stage"
                          value={avatarLoaded ? 'Attached' : 'Waiting'}
                          tone={avatarLoaded ? 'violet' : 'slate'}
                        />
                      </div>
                    </div>
                  </div>
                </GlassCard>

                <GlassCard eyebrow="Audio" title="Session signals">
                  <AudioVisualizerCard
                    icon={<AudioLines className="h-4 w-4 text-emerald-200" />}
                    label="You"
                    detail={userSignalDetail}
                    level={userSignalLevel}
                    tone="emerald"
                    state={
                      avatarState.connectionState === 'connecting'
                        ? 'connecting'
                        : avatarState.microphoneEnabled
                          ? 'listening'
                          : undefined
                    }
                  />
                  <AudioVisualizerCard
                    icon={<Link2 className="h-4 w-4 text-fuchsia-200" />}
                    label="Morgan"
                    detail={agentSignalDetail}
                    level={agentSignalLevel}
                    tone="violet"
                    state={
                      avatarState.connectionState === 'connecting'
                        ? 'connecting'
                        : avatarState.voiceState === 'speaking'
                          ? 'speaking'
                          : avatarState.connectionState === 'connected'
                            ? 'thinking'
                            : undefined
                    }
                  />
                </GlassCard>

                <GlassCard eyebrow="Exchange" title="Latest turn">
                  <div className="space-y-3">
                    <TranscriptBlock
                      label="You"
                      icon={<CircleDot className="h-3.5 w-3.5 text-emerald-200" />}
                      tone="emerald"
                      text={
                        avatarState.latestUserText ||
                        (callActive
                          ? 'Your latest utterance will appear here when Morgan hears you.'
                          : 'Start a call and your voice transcript will appear here.')
                      }
                    />
                    <TranscriptBlock
                      label="Morgan"
                      icon={<Volume2 className="h-3.5 w-3.5 text-fuchsia-200" />}
                      tone="violet"
                      text={
                        avatarState.latestAgentText ||
                        (callActive
                          ? 'Morgan’s latest spoken reply will appear here.'
                          : 'Start a call and Morgan’s spoken reply will appear here.')
                      }
                    />
                  </div>
                </GlassCard>

                <GlassCard eyebrow="Shared context" title="Paste links or briefs for this call">
                  <div className="space-y-3">
                    <p className="text-sm leading-6 text-slate-300">
                      Send Morgan the URL or structured payload here, then say the task out loud.
                      He will receive both in the same room-backed session.
                    </p>
                    <Textarea
                      value={sharedContextValue}
                      onChange={(event) => setSharedContextValue(event.target.value)}
                      placeholder="Paste a URL, PRD snippet, or notes for Morgan to use in the current call."
                      className="min-h-[112px] border-white/10 bg-black/20 text-slate-100 placeholder:text-slate-500"
                    />
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-xs text-slate-400">
                        {avatarState.roomName
                          ? `Targets room ${avatarState.roomName}.`
                          : 'Start a call first, then send the pasted context.'}
                      </p>
                      <Button
                        size="sm"
                        onClick={() => void sendSharedContext()}
                        disabled={sharedContextSending || !avatarState.roomName}
                      >
                        {sharedContextSending ? (
                          <>
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            Sending
                          </>
                        ) : (
                          'Send to Morgan'
                        )}
                      </Button>
                    </div>
                    {sharedContextStatus ? (
                      <div className="rounded-[18px] border border-white/10 bg-black/20 px-3 py-2 text-sm text-slate-200">
                        {sharedContextStatus}
                      </div>
                    ) : null}
                  </div>
                </GlassCard>
              </div>
            </TabsContent>

            <TabsContent value="debug" className="mt-0 min-h-0 flex-1 overflow-auto pr-1">
              <div className="space-y-4">
              <GlassCard eyebrow="Debug" title="Local prerequisites">
                <div className="space-y-3 text-sm text-slate-200">
                  <DebugRow label="Container runtime" value={runtimeSummary} />
                  <DebugRow label="Kind cluster" value={clusterSummary} />
                  <DebugRow label="Morgan ingress" value={gatewaySummary} />
                </div>
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Avatar source">
                <div className="space-y-3 text-sm text-slate-200">
                  <DebugRow label="Browser URL" value={avatarBrowserUrl} />
                  <DebugRow label="Token endpoint" value={avatarTokenEndpoint} />
                  <div className="flex flex-wrap gap-2">
                    <Button variant="secondary" size="sm" onClick={reloadAvatar}>
                      <RefreshCw className="mr-2 h-4 w-4" />
                      Reload
                    </Button>
                    <Button variant="outline" size="sm" onClick={openAvatarInBrowser}>
                      <ExternalLink className="mr-2 h-4 w-4" />
                      Open in browser
                    </Button>
                  </div>
                </div>
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Gateway access">
                <div className="space-y-3 text-sm text-slate-200">
                  <DebugRow
                    label="Status"
                    value={
                      bridgeStatus?.connected
                        ? 'Connected'
                        : bridgeStatus?.running
                          ? 'Fallback port-forward active'
                          : 'Ingress only'
                    }
                  />
                  <DebugRow
                    label="Kind service"
                    value={
                      bridgeStatus?.service && bridgeStatus?.namespace
                        ? `${bridgeStatus.namespace}/${bridgeStatus.service}`
                        : 'Unresolved'
                    }
                  />
                  <DebugRow label="Ingress host" value="http://morgan.localhost" />
                  <DebugRow
                    label="Fallback URL"
                    value={bridgeStatus?.localUrl ?? 'http://localhost:18789'}
                  />
                  <div className="flex flex-wrap gap-2">
                    <Button size="sm" onClick={() => void startBridge()}>
                      Start fallback
                    </Button>
                    <Button variant="secondary" size="sm" onClick={() => void stopBridge()}>
                      Stop fallback
                    </Button>
                  </div>
                </div>
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Morgan backend">
                <div className="space-y-3 text-sm text-slate-200">
                  <DebugRow
                    label="Gateway health"
                    value={backendDiagnostics?.healthy ? 'healthy' : 'unknown'}
                  />
                  <DebugRow
                    label="Primary model"
                    value={backendDiagnostics?.modelPrimary ?? 'unresolved'}
                  />
                  <DebugRow
                    label="Fallbacks"
                    value={
                      backendDiagnostics?.modelFallbacks.length
                        ? backendDiagnostics.modelFallbacks.join(', ')
                        : 'none'
                    }
                  />
                  <DebugRow
                    label="Catalog source"
                    value={backendDiagnostics?.catalogSource ?? 'values'}
                  />
                  <DebugRow
                    label="Catalog generated"
                    value={backendDiagnostics?.catalogGeneratedAt ?? 'unknown'}
                  />
                  <DebugRow
                    label="Catalog coverage"
                    value={
                      backendDiagnostics
                        ? `${backendDiagnostics.catalogProviderCount} providers · ${backendDiagnostics.catalogModelCount} models`
                        : 'unknown'
                    }
                  />
                </div>
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Latest backend issues">
                {backendDiagnostics?.recentErrors.length ? (
                  <DebugFeed
                    entries={backendDiagnostics.recentErrors.map((entry, index) => ({
                      key: `backend-error-${index}`,
                      tone: 'slate' as const,
                      eyebrow: 'morgan backend',
                      body: entry,
                      meta: 'last 10m',
                    }))}
                  />
                ) : (
                  <EmptyDebugState text="No recent Morgan backend errors detected." />
                )}
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Local mic monitor">
                <LevelRow
                  icon={
                    micEnabled ? (
                      <Mic className="h-4 w-4 text-emerald-200" />
                    ) : (
                      <MicOff className="h-4 w-4 text-slate-400" />
                    )
                  }
                  label="Raw input"
                  detail={micEnabled ? 'Sampling locally' : 'Disabled'}
                  level={micLevel}
                  colorClass="from-emerald-300 via-teal-300 to-cyan-300"
                />
                <div className="mt-3 flex gap-2">
                  {!micEnabled ? (
                    <Button size="sm" onClick={() => void startMicMonitor()}>
                      <Mic className="mr-2 h-4 w-4" />
                      Monitor mic
                    </Button>
                  ) : (
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => void stopMicMonitor()}
                    >
                      <MicOff className="mr-2 h-4 w-4" />
                      Stop monitor
                    </Button>
                  )}
                </div>
              </GlassCard>

              {lastError ? (
                <GlassCard eyebrow="Debug" title="Latest error">
                  <div className="rounded-[20px] border border-rose-400/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-100">
                    <div className="flex items-start gap-3">
                      <ShieldAlert className="mt-0.5 h-4 w-4 shrink-0" />
                      <p>{lastError}</p>
                    </div>
                  </div>
                </GlassCard>
              ) : null}

              <GlassCard eyebrow="Debug" title="Session snapshot">
                <div className="grid gap-3 sm:grid-cols-2">
                  {sessionSnapshot.map((entry) => (
                    <DebugRow key={entry.label} label={entry.label} value={entry.value} />
                  ))}
                </div>
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Timing">
                {timingSnapshot.length > 0 ? (
                  <div className="grid gap-3 sm:grid-cols-2">
                    {timingSnapshot.map((entry) => (
                      <DebugRow key={entry.label} label={entry.label} value={entry.value} />
                    ))}
                  </div>
                ) : (
                  <EmptyDebugState text="Timing telemetry will appear once the room starts connecting." />
                )}
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Track publication">
                <div className="space-y-3">
                  <TrackPublicationCard
                    title="Local microphone"
                    publication={avatarDebug.localMicrophoneTrack ?? null}
                    emptyLabel="No local microphone publication yet."
                  />
                  <TrackPublicationCard
                    title="Selected audio track"
                    publication={avatarDebug.selectedAudioTrack ?? null}
                    emptyLabel="No remote audio track selected yet."
                  />
                  <TrackPublicationCard
                    title="Selected video track"
                    publication={avatarDebug.selectedVideoTrack ?? null}
                    emptyLabel="No remote video track selected yet."
                  />
                </div>
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Recent transcripts">
                {recentTranscripts.length > 0 ? (
                  <DebugFeed
                    entries={recentTranscripts.map((entry, index) => ({
                      key: `${entry.at}-${entry.identity}-${index}`,
                      tone: entry.source === 'user' ? 'emerald' : 'violet',
                      eyebrow: `${entry.source} · ${entry.identity}`,
                      body: entry.text,
                      meta: `${formatEventTime(entry.at)}${entry.final === null ? '' : entry.final ? ' · final' : ' · interim'}`,
                    }))}
                  />
                ) : (
                  <EmptyDebugState text="No transcript events have reached the desktop UI yet." />
                )}
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Recent client events">
                {recentEvents.length > 0 ? (
                  <DebugFeed
                    entries={recentEvents.map((entry, index) => ({
                      key: `${entry.at}-${entry.type}-${index}`,
                      tone: eventTone(entry.type),
                      eyebrow: entry.type,
                      body: entry.detail,
                      meta: formatEventTime(entry.at),
                    }))}
                  />
                ) : (
                  <EmptyDebugState text="Client-side room events will appear here as you connect and speak." />
                )}
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Remote participants">
                {remoteParticipants.length > 0 ? (
                  <div className="space-y-3">
                    {remoteParticipants.map((participant) => (
                      <div
                        key={participant.identity}
                        className="rounded-[20px] border border-white/8 bg-black/20 p-4"
                      >
                        <div className="flex flex-wrap items-center gap-2">
                          <StatusBadge
                            label={participant.identity}
                            tone="slate"
                          />
                          <span className="text-xs text-slate-400">
                            {participant.kind ?? 'unknown'}
                          </span>
                          {participant.publishOnBehalf ? (
                            <span className="text-xs text-slate-500">
                              behalf: {participant.publishOnBehalf}
                            </span>
                          ) : null}
                        </div>
                        <div className="mt-3 space-y-2">
                          {participant.tracks.length > 0 ? (
                            participant.tracks.map((track, index) => (
                              <TrackLine
                                key={`${participant.identity}-${track.sid ?? index}`}
                                track={track}
                              />
                            ))
                          ) : (
                            <p className="text-sm text-slate-400">No track publications.</p>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <EmptyDebugState text="No remote participants are attached to the room yet." />
                )}
              </GlassCard>

              <GlassCard eyebrow="Debug" title="Raw avatar telemetry">
                <pre className="overflow-x-auto rounded-[20px] bg-black/35 p-4 text-[11px] leading-5 text-slate-300">
                  {JSON.stringify(avatarState, null, 2)}
                </pre>
              </GlassCard>
              </div>
            </TabsContent>
          </Tabs>
        </aside>
      </div>
    </div>
  )
}

function GlassCard({
  eyebrow,
  title,
  description,
  children,
}: {
  eyebrow: string
  title: string
  description?: string
  children?: ReactNode
}) {
  return (
    <section className="rounded-[24px] border border-white/10 bg-white/[0.045] p-4 shadow-[0_12px_50px_-36px_rgba(14,165,233,0.7)]">
      <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-200/70">
        {eyebrow}
      </p>
      <h2 className="mt-2 text-base font-semibold text-white">{title}</h2>
      {description ? (
        <p className="mt-2 text-sm leading-6 text-slate-300">{description}</p>
      ) : null}
      {children ? <div className="mt-4">{children}</div> : null}
    </section>
  )
}

function TranscriptBlock({
  label,
  icon,
  text,
  tone,
}: {
  label: string
  icon: ReactNode
  text: string
  tone: 'emerald' | 'violet'
}) {
  const toneClass =
    tone === 'emerald'
      ? 'border-emerald-400/20 bg-emerald-950/35'
      : 'border-fuchsia-400/20 bg-fuchsia-950/25'

  return (
    <div className={`rounded-[20px] border p-4 ${toneClass}`}>
      <div className="flex items-center gap-2 text-[11px] uppercase tracking-[0.28em] text-slate-300">
        {icon}
        <span>{label}</span>
      </div>
      <p className="mt-3 text-sm leading-7 text-slate-100">{text}</p>
    </div>
  )
}

function AudioVisualizerCard({
  icon,
  label,
  detail,
  level,
  tone,
  state,
}: {
  icon: ReactNode
  label: string
  detail: string
  level: number
  tone: 'emerald' | 'violet'
  state?: 'connecting' | 'initializing' | 'listening' | 'speaking' | 'thinking'
}) {
  const frameClass =
    tone === 'emerald'
      ? 'border-emerald-400/20 bg-emerald-950/20'
      : 'border-fuchsia-400/20 bg-fuchsia-950/20'
  const accentClass =
    tone === 'emerald' ? 'text-emerald-100' : 'text-fuchsia-100'

  return (
    <div className={`rounded-[20px] border p-3 ${frameClass}`}>
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-full border border-white/10 bg-white/5">
          {icon}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center justify-between gap-3">
            <p className="text-sm font-medium text-white">{label}</p>
            <p className={`text-xs ${accentClass}`}>{Math.round(level * 100)}%</p>
          </div>
          <p className="text-xs text-slate-400">{detail}</p>
        </div>
      </div>
      <div className="mt-3 overflow-hidden rounded-[18px] border border-white/8 bg-black/20">
        <BarVisualizer
          state={state}
          barCount={18}
          demo
          minHeight={16}
          maxHeight={92}
          centerAlign
          className="h-24 rounded-[18px] border-0 bg-transparent px-3 py-4"
        />
      </div>
    </div>
  )
}

function SignalTile({
  label,
  value,
  tone,
}: {
  label: string
  value: string
  tone: 'emerald' | 'violet' | 'slate'
}) {
  const toneClass =
    tone === 'emerald'
      ? 'border-emerald-400/20 bg-emerald-950/25 text-emerald-100'
      : tone === 'violet'
        ? 'border-fuchsia-400/20 bg-fuchsia-950/25 text-fuchsia-100'
        : 'border-white/8 bg-black/20 text-slate-200'

  return (
    <div className={`rounded-[18px] border px-3 py-3 ${toneClass}`}>
      <p className="text-[11px] uppercase tracking-[0.28em] text-slate-400">{label}</p>
      <p className="mt-2 text-sm font-medium">{value}</p>
    </div>
  )
}

function LevelRow({
  icon,
  label,
  detail,
  level,
  colorClass,
}: {
  icon: ReactNode
  label: string
  detail: string
  level: number
  colorClass: string
}) {
  return (
    <div className="rounded-[20px] border border-white/8 bg-black/20 p-3">
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 items-center justify-center rounded-full border border-white/10 bg-white/5">
          {icon}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center justify-between gap-3">
            <p className="text-sm font-medium text-white">{label}</p>
            <p className="text-xs text-slate-400">{Math.round(level * 100)}%</p>
          </div>
          <p className="text-xs text-slate-400">{detail}</p>
        </div>
      </div>
      <div className="mt-3 h-2 overflow-hidden rounded-full bg-slate-800">
        <div
          className={`h-full bg-gradient-to-r ${colorClass} transition-[width] duration-150`}
          style={{ width: `${clamp(level * 100, 0, 100)}%` }}
        />
      </div>
    </div>
  )
}

function DebugRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-4 rounded-[18px] border border-white/8 bg-black/20 px-3 py-2">
      <span className="text-slate-400">{label}</span>
      <span className="text-right text-slate-100">{value}</span>
    </div>
  )
}

function TrackPublicationCard({
  title,
  publication,
  emptyLabel,
}: {
  title: string
  publication: TrackPublicationDebug | null
  emptyLabel: string
}) {
  return (
    <div className="rounded-[20px] border border-white/8 bg-black/20 p-4">
      <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">{title}</p>
      {publication ? (
        <div className="mt-3 grid gap-3 sm:grid-cols-2">
          <DebugRow label="Participant" value={publication.participant ?? 'unknown'} />
          <DebugRow label="Source" value={publication.source ?? 'unknown'} />
          <DebugRow label="Track SID" value={publication.sid ?? 'pending'} />
          <DebugRow label="Track name" value={publication.name ?? 'unnamed'} />
          <DebugRow label="Kind" value={publication.kind ?? 'unknown'} />
          <DebugRow
            label="Flags"
            value={[
              publication.subscribed ? 'subscribed' : 'unsubscribed',
              publication.enabled ? 'enabled' : 'disabled',
              publication.muted ? 'muted' : 'unmuted',
            ].join(' · ')}
          />
        </div>
      ) : (
        <p className="mt-3 text-sm text-slate-400">{emptyLabel}</p>
      )}
    </div>
  )
}

function TrackLine({ track }: { track: TrackPublicationDebug }) {
  return (
    <div className="rounded-[16px] border border-white/8 bg-slate-950/60 px-3 py-2">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm font-medium text-slate-100">
          {track.name ?? track.source ?? 'track'}
        </span>
        <span className="text-xs text-slate-500">{track.kind ?? 'unknown'}</span>
      </div>
      <p className="mt-1 text-xs text-slate-400">
        {[
          track.source ?? 'unknown source',
          track.subscribed ? 'subscribed' : 'unsubscribed',
          track.enabled ? 'enabled' : 'disabled',
          track.muted ? 'muted' : 'unmuted',
        ].join(' · ')}
      </p>
    </div>
  )
}

function DebugFeed({
  entries,
}: {
  entries: Array<{
    key: string
    tone: 'emerald' | 'violet' | 'slate'
    eyebrow: string
    body: string
    meta: string
  }>
}) {
  return (
    <div className="space-y-3">
      {entries
        .slice()
        .reverse()
        .map((entry) => (
          <div
            key={entry.key}
            className={`rounded-[20px] border px-4 py-3 ${debugFeedTone(entry.tone)}`}
          >
            <div className="flex items-start justify-between gap-3">
              <p className="text-[11px] uppercase tracking-[0.28em] text-slate-300">
                {entry.eyebrow}
              </p>
              <span className="text-[11px] text-slate-500">{entry.meta}</span>
            </div>
            <p className="mt-2 text-sm leading-6 text-slate-100">{entry.body}</p>
          </div>
        ))}
    </div>
  )
}

function EmptyDebugState({ text }: { text: string }) {
  return (
    <div className="rounded-[20px] border border-dashed border-white/10 bg-black/15 px-4 py-6 text-sm text-slate-400">
      {text}
    </div>
  )
}

function debugFeedTone(tone: 'emerald' | 'violet' | 'slate'): string {
  if (tone === 'emerald') {
    return 'border-emerald-400/20 bg-emerald-950/20'
  }

  if (tone === 'violet') {
    return 'border-fuchsia-400/20 bg-fuchsia-950/20'
  }

  return 'border-white/8 bg-black/20'
}

function eventTone(type: string): 'emerald' | 'violet' | 'slate' {
  if (type.includes('transcript') || type.includes('mic')) {
    return 'emerald'
  }

  if (type.includes('voice')) {
    return 'violet'
  }

  return 'slate'
}

function formatEventTime(value: string): string {
  const date = new Date(value)

  if (Number.isNaN(date.getTime())) {
    return value
  }

  return date.toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function StatusBadge({
  label,
  tone,
}: {
  label: string
  tone: 'cyan' | 'emerald' | 'violet' | 'amber' | 'slate'
}) {
  const toneClass =
    tone === 'cyan'
      ? 'border-cyan-300/30 bg-cyan-400/10 text-cyan-100'
      : tone === 'emerald'
        ? 'border-emerald-300/30 bg-emerald-400/10 text-emerald-100'
        : tone === 'violet'
          ? 'border-fuchsia-300/30 bg-fuchsia-400/10 text-fuchsia-100'
          : tone === 'amber'
            ? 'border-amber-300/30 bg-amber-400/10 text-amber-100'
            : 'border-white/12 bg-white/10 text-slate-300'

  return (
    <span
      className={`rounded-full border px-3 py-1 text-[11px] font-medium uppercase tracking-[0.22em] ${toneClass}`}
    >
      {label}
    </span>
  )
}
