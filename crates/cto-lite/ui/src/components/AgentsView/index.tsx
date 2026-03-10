import { type ReactNode, useCallback, useEffect, useRef, useState } from 'react'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  AudioLines,
  CircleDot,
  ExternalLink,
  Link2,
  Mic,
  MicOff,
  RefreshCw,
  ShieldAlert,
  Volume2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { BarVisualizer } from '@/components/ui/bar-visualizer'
import { LiveWaveform } from '@/components/ui/live-waveform'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  MorganAvatarRoom,
  type MorganAvatarState,
} from '@/components/MorganAvatarRoom'
import * as tauri from '@/lib/tauri'

type PanelMode = 'live' | 'debug'

const DEFAULT_AVATAR_STATE: MorganAvatarState = {
  connectionState: 'idle',
  voiceState: 'idle',
  latestUserText: '',
  latestAgentText: '',
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
    left.voiceState === right.voiceState &&
    left.latestUserText === right.latestUserText &&
    left.latestAgentText === right.latestAgentText &&
    left.audioTrackReady === right.audioTrackReady &&
    left.videoTrackReady === right.videoTrackReady &&
    left.roomName === right.roomName &&
    left.identity === right.identity &&
    left.error === right.error &&
    sameRecord(left.metrics, right.metrics) &&
    sameRecord(left.trackDebug, right.trackDebug)
  )
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
  const [bridgeBooting, setBridgeBooting] = useState(true)
  const [runtimeEnvironment, setRuntimeEnvironment] = useState<tauri.RuntimeEnvironment | null>(
    null
  )
  const [clusterStatus, setClusterStatus] = useState<tauri.ClusterInfo | null>(null)
  const [lastError, setLastError] = useState<string | null>(null)
  const [micEnabled, setMicEnabled] = useState(false)
  const [micLevel, setMicLevel] = useState(0)
  const [agentLevel, setAgentLevel] = useState(0)
  const [userLevel, setUserLevel] = useState(0)
  const audioContextRef = useRef<AudioContext | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const animationRef = useRef<number | null>(null)
  const lastUserTextRef = useRef('')
  const lastAgentTextRef = useRef('')
  const lastUserActivityAtRef = useRef<number>(0)
  const lastAgentActivityAtRef = useRef<number>(0)

  useEffect(() => {
    let cancelled = false

    const syncLocalState = async () => {
      try {
        const [gateway, runtime, cluster] = await Promise.allSettled([
          tauri.openclawGetLocalBridgeStatus(),
          tauri.scanRuntimeEnvironment(),
          tauri.getClusterStatus(),
        ])

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
    if (
      avatarState.latestUserText &&
      avatarState.latestUserText !== lastUserTextRef.current
    ) {
      lastUserTextRef.current = avatarState.latestUserText
      lastUserActivityAtRef.current = Date.now()
    }
  }, [avatarState.latestUserText])

  useEffect(() => {
    if (
      avatarState.latestAgentText &&
      avatarState.latestAgentText !== lastAgentTextRef.current
    ) {
      lastAgentTextRef.current = avatarState.latestAgentText
      lastAgentActivityAtRef.current = Date.now()
    }
  }, [avatarState.latestAgentText])

  useEffect(() => {
    const tick = window.setInterval(() => {
      const now = Date.now()
      const userActive =
        avatarState.voiceState === 'listening' ||
        now - lastUserActivityAtRef.current < 2000
      const agentActive =
        avatarState.voiceState === 'speaking' ||
        now - lastAgentActivityAtRef.current < 2200

      setUserLevel((previous) =>
        userActive
          ? 0.34 + Math.random() * 0.48
          : clamp(previous * 0.76, 0, 1)
      )
      setAgentLevel((previous) =>
        agentActive
          ? 0.38 + Math.random() * 0.5
          : clamp(previous * 0.76, 0, 1)
      )
    }, 120)

    return () => window.clearInterval(tick)
  }, [avatarState.voiceState])

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
  const bridgeTone = bridgeStatus?.connected
    ? 'emerald'
    : bridgeStatus?.running
      ? 'amber'
      : 'slate'
  const voiceTone =
    avatarState.voiceState === 'speaking'
      ? 'violet'
      : avatarState.voiceState === 'listening'
        ? 'emerald'
        : 'slate'

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
          <Separator orientation="vertical" className="hidden h-10 bg-white/8 xl:block" />
          <StatusBadge
            label={
              bridgeStatus?.connected
                ? 'Gateway live'
                : bridgeBooting
                  ? 'Checking local stack'
                  : clusterStatus?.running
                    ? 'Gateway offline'
                    : 'Kind offline'
            }
            tone={bridgeTone}
          />
          <StatusBadge
            label={
              avatarState.connectionState === 'connected'
                ? avatarState.voiceState || 'ready'
                : avatarState.connectionState === 'connecting'
                  ? 'connecting'
                  : 'standby'
            }
            tone={voiceTone}
          />
        </div>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 p-4 lg:grid-cols-[minmax(0,1.2fr)_380px]">
        <section className="relative min-h-0 overflow-hidden rounded-[28px] border border-white/10 bg-black/30 shadow-[0_26px_80px_-42px_rgba(8,145,178,0.7)]">
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,#164e63_0%,rgba(2,6,23,0.38)_24%,rgba(2,6,23,0.9)_100%)]" />
          <div className="absolute left-4 top-4 z-10 flex flex-wrap items-center gap-2">
            <StatusBadge
              label={
                avatarLoaded
                  ? 'Avatar loaded'
                  : bridgeStatus?.connected
                    ? 'Preparing session'
                    : runtimeEnvironment && !runtimeEnvironment.docker_available
                      ? 'Docker required'
                      : clusterStatus?.running
                        ? 'Waiting for ingress'
                        : 'Waiting for local stack'
              }
              tone={avatarLoaded ? 'emerald' : 'slate'}
            />
            {avatarState.audioTrackReady ? (
              <StatusBadge label="Audio active" tone="emerald" />
            ) : null}
            {avatarState.videoTrackReady ? (
              <StatusBadge label="Video active" tone="violet" />
            ) : null}
          </div>
          <div className="relative z-[1] h-full min-h-[760px] w-full">
            <MorganAvatarRoom
              key={avatarSessionKey}
              compact
              autoConnect
              tokenEndpoint={avatarTokenEndpoint}
              onStateChange={handleAvatarStateChange}
            />
          </div>
        </section>

        <aside className="min-h-0 overflow-hidden rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-4">
          <Tabs
            value={panelMode}
            onValueChange={(value) => setPanelMode(value as PanelMode)}
            className="h-full"
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

            <TabsContent value="live" className="mt-0 overflow-auto pr-1">
              <div className="space-y-4">
              <GlassCard
                eyebrow="Voice deck"
                title={
                  bridgeStatus?.connected
                    ? 'Morgan is on the local stack'
                    : runtimeEnvironment && !runtimeEnvironment.docker_available
                      ? 'Docker is required'
                      : clusterStatus && !clusterStatus.exists
                        ? 'Local stack not created'
                        : clusterStatus && !clusterStatus.running
                          ? 'Local cluster offline'
                          : bridgeBooting
                            ? 'Checking Morgan'
                            : 'Morgan ingress offline'
                }
                description={
                  avatarState.connectionState === 'connected'
                    ? `${avatarState.roomName ?? 'morgan'} · ${avatarState.identity ?? 'guest'}`
                    : bridgeStatus?.connected
                      ? 'Avatar surface is ready. Press Talk in the stage to enter the call.'
                    : runtimeEnvironment && !runtimeEnvironment.docker_available
                      ? 'Start Docker so CTO can keep the local stack online.'
                      : clusterStatus && !clusterStatus.exists
                        ? 'Bootstrap the local kind cluster and deploy Morgan.'
                        : clusterStatus && !clusterStatus.running
                          ? 'The cluster exists but is not reachable yet.'
                          : 'The avatar surface will connect as soon as the local ingress is ready.'
                }
              >
                <div className="grid gap-3">
                  <div className="grid grid-cols-2 gap-3">
                    <SignalTile
                      label="Gateway"
                      value={
                        bridgeStatus?.connected
                          ? 'Live'
                          : bridgeBooting
                            ? 'Checking'
                            : 'Offline'
                      }
                      tone={bridgeStatus?.connected ? 'emerald' : 'slate'}
                    />
                    <SignalTile
                      label="Room"
                      value={
                        avatarState.connectionState === 'connected'
                          ? 'Joined'
                          : avatarState.connectionState === 'connecting'
                            ? 'Joining'
                            : 'Standby'
                      }
                      tone={
                        avatarState.connectionState === 'connected'
                          ? 'violet'
                          : 'slate'
                      }
                    />
                    <SignalTile
                      label="Audio"
                      value={avatarState.audioTrackReady ? 'Live' : 'Pending'}
                      tone={avatarState.audioTrackReady ? 'emerald' : 'slate'}
                    />
                    <SignalTile
                      label="Video"
                      value={avatarState.videoTrackReady ? 'Live' : 'Pending'}
                      tone={avatarState.videoTrackReady ? 'violet' : 'slate'}
                    />
                  </div>
                  <div className="rounded-[22px] border border-white/10 bg-black/20 p-3">
                    <div className="flex items-center justify-between gap-3">
                      <div>
                        <p className="text-[11px] uppercase tracking-[0.3em] text-cyan-100/70">
                          Call surface
                        </p>
                        <p className="mt-2 text-sm text-slate-200">
                          {avatarState.voiceState === 'speaking'
                            ? 'Morgan is responding.'
                            : avatarState.voiceState === 'listening'
                              ? 'Morgan is listening.'
                              : avatarState.connectionState === 'connected'
                                ? 'Press Talk on the stage when you want to speak.'
                                : 'Waiting for the room to settle.'}
                        </p>
                      </div>
                      <StatusBadge
                        label={
                          avatarState.voiceState === 'speaking'
                            ? 'speaking'
                            : avatarState.voiceState === 'listening'
                              ? 'listening'
                              : avatarState.connectionState === 'connected'
                                ? 'ready'
                                : 'standby'
                        }
                        tone={
                          avatarState.voiceState === 'speaking'
                            ? 'violet'
                            : avatarState.voiceState === 'listening'
                              ? 'emerald'
                              : 'slate'
                        }
                      />
                    </div>
                    <div className="mt-4 overflow-hidden rounded-[18px] border border-white/8 bg-slate-950/60 px-3 py-2">
                      <LiveWaveform
                        active={false}
                        processing={avatarState.connectionState !== 'idle'}
                        mode="static"
                        height={48}
                        fadeEdges={false}
                        barWidth={4}
                        barGap={2}
                        className="h-12 rounded-[14px] bg-transparent"
                      />
                    </div>
                  </div>
                </div>
              </GlassCard>

              <GlassCard eyebrow="Audio" title="Live voice activity">
                <AudioVisualizerCard
                  icon={<AudioLines className="h-4 w-4 text-emerald-200" />}
                  label="You"
                  detail={
                    avatarState.voiceState === 'listening'
                      ? 'Listening now'
                      : avatarState.latestUserText
                        ? 'Recent input detected'
                        : 'Standing by'
                  }
                  level={userLevel}
                  tone="emerald"
                  state={
                    avatarState.connectionState === 'connecting'
                      ? 'connecting'
                      : avatarState.voiceState === 'listening'
                        ? 'listening'
                        : undefined
                  }
                />
                <AudioVisualizerCard
                  icon={<Link2 className="h-4 w-4 text-fuchsia-200" />}
                  label="Morgan"
                  detail={
                    avatarState.voiceState === 'speaking'
                      ? 'Responding'
                      : avatarState.latestAgentText
                        ? 'Response buffered'
                        : 'Waiting for turn'
                  }
                  level={agentLevel}
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
                      'Your latest utterance will appear here when Morgan hears you.'
                    }
                  />
                  <TranscriptBlock
                    label="Morgan"
                    icon={<Volume2 className="h-3.5 w-3.5 text-fuchsia-200" />}
                    tone="violet"
                    text={
                      avatarState.latestAgentText ||
                      'Morgan’s latest spoken reply will appear here.'
                    }
                  />
                </div>
              </GlassCard>
              </div>
            </TabsContent>

            <TabsContent value="debug" className="mt-0 overflow-auto pr-1">
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

              <GlassCard eyebrow="Debug" title="Avatar telemetry">
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
