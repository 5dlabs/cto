import { type ReactNode, useEffect, useRef, useState } from 'react'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  AudioLines,
  ExternalLink,
  Link2,
  Mic,
  MicOff,
  RefreshCw,
  ShieldAlert,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import * as tauri from '@/lib/tauri'

type PanelMode = 'live' | 'debug'

type AvatarEmbedState = {
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

const DEFAULT_AVATAR_STATE: AvatarEmbedState = {
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

export function AgentsView() {
  const [avatarUrl, setAvatarUrl] = useState(
    ((import.meta as { env?: Record<string, string> }).env?.VITE_MORGAN_AVATAR_URL as string) ??
      'http://localhost:3000/embed'
  )
  const [avatarLoaded, setAvatarLoaded] = useState(false)
  const [iframeKey, setIframeKey] = useState(0)
  const [panelMode, setPanelMode] = useState<PanelMode>('live')
  const [avatarState, setAvatarState] = useState<AvatarEmbedState>(DEFAULT_AVATAR_STATE)
  const [bridgeStatus, setBridgeStatus] = useState<tauri.OpenClawBridgeStatus | null>(null)
  const [bridgeBooting, setBridgeBooting] = useState(true)
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

    const syncBridgeStatus = async (startIfNeeded: boolean) => {
      try {
        let status = await tauri.openclawGetLocalBridgeStatus()
        if (startIfNeeded && !status.connected && !status.running) {
          status = await tauri.openclawStartLocalBridge()
        }

        if (!cancelled) {
          setBridgeStatus(status)
          setLastError(null)
        }
      } catch (error) {
        if (!cancelled) {
          setLastError(String(error))
        }
      } finally {
        if (!cancelled) {
          setBridgeBooting(false)
        }
      }
    }

    void syncBridgeStatus(true)
    const poll = window.setInterval(() => {
      void syncBridgeStatus(false)
    }, 4000)

    return () => {
      cancelled = true
      window.clearInterval(poll)
    }
  }, [])

  useEffect(() => {
    const allowedOrigin = (() => {
      try {
        return new URL(avatarUrl).origin
      } catch {
        return null
      }
    })()

    const handleMessage = (event: MessageEvent) => {
      if (allowedOrigin && event.origin !== allowedOrigin) {
        return
      }

      const payload = event.data as
        | { type?: string; payload?: AvatarEmbedState }
        | undefined
      if (payload?.type !== 'cto-avatar-state' || !payload.payload) {
        return
      }

      setAvatarState(payload.payload)
      if (payload.payload.error) {
        setLastError(payload.payload.error)
      }
    }

    window.addEventListener('message', handleMessage)
    return () => window.removeEventListener('message', handleMessage)
  }, [avatarUrl])

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
    setIframeKey((value) => value + 1)
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

  const openAvatarInBrowser = async () => {
    await openUrl(avatarUrl)
  }

  const bridgeTone = bridgeStatus?.connected
    ? 'emerald'
    : bridgeStatus?.running || bridgeBooting
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
          <StatusBadge label="Avatar" tone="cyan" />
          <h1 className="text-lg font-semibold tracking-tight">Morgan</h1>
          <StatusBadge
            label={
              bridgeStatus?.connected
                ? 'Bridge live'
                : bridgeBooting
                  ? 'Linking bridge'
                  : 'Bridge offline'
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
          <div className="ml-auto inline-flex rounded-full border border-white/10 bg-white/5 p-1">
            <button
              type="button"
              onClick={() => setPanelMode('live')}
              className={`rounded-full px-3 py-1.5 text-xs font-medium transition ${
                panelMode === 'live'
                  ? 'bg-cyan-400/15 text-cyan-100'
                  : 'text-slate-400 hover:text-slate-100'
              }`}
            >
              Live
            </button>
            <button
              type="button"
              onClick={() => setPanelMode('debug')}
              className={`rounded-full px-3 py-1.5 text-xs font-medium transition ${
                panelMode === 'debug'
                  ? 'bg-cyan-400/15 text-cyan-100'
                  : 'text-slate-400 hover:text-slate-100'
              }`}
            >
              Debug
            </button>
          </div>
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
                  : bridgeBooting
                    ? 'Preparing session'
                    : 'Waiting for avatar'
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
          <iframe
            key={iframeKey}
            src={avatarUrl}
            title="Morgan Avatar"
            className="relative z-[1] h-full min-h-[760px] w-full bg-transparent"
            onLoad={() => setAvatarLoaded(true)}
          />
        </section>

        <aside className="min-h-0 overflow-auto rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-4">
          {panelMode === 'live' ? (
            <div className="space-y-4">
              <GlassCard
                eyebrow="Session"
                title={
                  bridgeStatus?.connected
                    ? 'Morgan is linked to CTO'
                    : bridgeBooting
                      ? 'Linking Morgan'
                      : 'Morgan is waiting for the bridge'
                }
                description={
                  avatarState.connectionState === 'connected'
                    ? `Room ${avatarState.roomName ?? 'live'}${
                        avatarState.identity ? ` · ${avatarState.identity}` : ''
                      }`
                    : 'The avatar surface will auto-connect as soon as the local bridge is ready.'
                }
              />

              <GlassCard eyebrow="Audio" title="Live voice activity">
                <LevelRow
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
                  colorClass="from-emerald-300 via-cyan-300 to-sky-300"
                />
                <LevelRow
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
                  colorClass="from-fuchsia-300 via-violet-300 to-indigo-300"
                />
              </GlassCard>

              <GlassCard eyebrow="Transcript" title="You said">
                <TranscriptBlock
                  tone="emerald"
                  text={
                    avatarState.latestUserText ||
                    'Your latest utterance will appear here as soon as Morgan hears you.'
                  }
                />
              </GlassCard>

              <GlassCard eyebrow="Transcript" title="Morgan said">
                <TranscriptBlock
                  tone="violet"
                  text={
                    avatarState.latestAgentText ||
                    'Morgan’s latest spoken reply will appear here.'
                  }
                />
              </GlassCard>
            </div>
          ) : (
            <div className="space-y-4">
              <GlassCard eyebrow="Debug" title="Avatar source">
                <div className="space-y-3">
                  <Input
                    value={avatarUrl}
                    onChange={(event) => setAvatarUrl(event.target.value)}
                    placeholder="Avatar URL"
                    className="h-10 border-white/10 bg-white/5"
                  />
                  <div className="flex flex-wrap gap-2">
                    <Button variant="secondary" size="sm" onClick={reloadAvatar}>
                      <RefreshCw className="mr-2 h-4 w-4" />
                      Reload
                    </Button>
                    <Button variant="outline" size="sm" onClick={openAvatarInBrowser}>
                      <ExternalLink className="mr-2 h-4 w-4" />
                      Open
                    </Button>
                  </div>
                </div>
              </GlassCard>

              <GlassCard eyebrow="Debug" title="OpenClaw bridge">
                <div className="space-y-3 text-sm text-slate-200">
                  <DebugRow
                    label="Status"
                    value={
                      bridgeStatus?.connected
                        ? 'Connected'
                        : bridgeStatus?.running
                          ? 'Forwarding'
                          : 'Stopped'
                    }
                  />
                  <DebugRow
                    label="Service"
                    value={
                      bridgeStatus?.service && bridgeStatus?.namespace
                        ? `${bridgeStatus.namespace}/${bridgeStatus.service}`
                        : 'Unresolved'
                    }
                  />
                  <DebugRow
                    label="Local URL"
                    value={bridgeStatus?.localUrl ?? 'http://localhost:18789'}
                  />
                  <div className="flex flex-wrap gap-2">
                    <Button size="sm" onClick={() => void startBridge()}>
                      Start bridge
                    </Button>
                    <Button variant="secondary" size="sm" onClick={() => void stopBridge()}>
                      Stop bridge
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
          )}
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
  text,
  tone,
}: {
  text: string
  tone: 'emerald' | 'violet'
}) {
  const toneClass =
    tone === 'emerald'
      ? 'border-emerald-400/20 bg-emerald-950/35'
      : 'border-fuchsia-400/20 bg-fuchsia-950/25'

  return (
    <div className={`rounded-[20px] border p-4 text-sm leading-7 text-slate-100 ${toneClass}`}>
      {text}
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
