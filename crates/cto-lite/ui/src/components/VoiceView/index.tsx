import { type ReactNode, useCallback, useEffect, useState } from 'react'
import { AudioLines, CircleDot, Loader2, Volume2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import { BarVisualizer } from '@/components/ui/bar-visualizer'
import {
  MorganAvatarRoom,
  type MorganAvatarState,
} from '@/components/MorganAvatarRoom'
import * as tauri from '@/lib/tauri'

type VoiceViewProps = {
  agentId: string
  agentName: string
  projectName: string
  roomName: string
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

export function VoiceView({ agentId, agentName, projectName, roomName }: VoiceViewProps) {
  const avatarTokenEndpoint =
    ((import.meta as { env?: Record<string, string> }).env?.VITE_MORGAN_AVATAR_TOKEN_ENDPOINT as string) ??
    '/avatar-api/token'
  const [avatarState, setAvatarState] = useState<MorganAvatarState>(DEFAULT_AVATAR_STATE)
  const [panelMode, setPanelMode] = useState<'live' | 'debug'>('live')
  const [sharedContextValue, setSharedContextValue] = useState('')
  const [sharedContextStatus, setSharedContextStatus] = useState<string | null>(null)
  const [sharedContextSending, setSharedContextSending] = useState(false)
  const [diagnostics, setDiagnostics] = useState<tauri.MorganDiagnostics | null>(null)

  useEffect(() => {
    let cancelled = false

    const syncDiagnostics = async () => {
      try {
        const next = await tauri.openclawGetMorganDiagnostics(agentId)
        if (!cancelled) {
          setDiagnostics(next)
        }
      } catch {
        if (!cancelled) {
          setDiagnostics(null)
        }
      }
    }

    void syncDiagnostics()
    const timer = window.setInterval(() => {
      void syncDiagnostics()
    }, 5000)

    return () => {
      cancelled = true
      window.clearInterval(timer)
    }
  }, [agentId])

  const sendSharedContext = useCallback(async () => {
    const content = sharedContextValue.trim()
    if (!content) {
      setSharedContextStatus('Paste a URL or brief before sending.')
      return
    }

    setSharedContextSending(true)
    setSharedContextStatus(null)

    try {
      const response = await tauri.openclawSendAvatarContext(roomName, content, agentId)
      setSharedContextStatus(
        response.content === 'CONTEXT_STORED'
          ? `Shared with ${agentName} for this live call.`
          : response.content
      )
      setSharedContextValue('')
    } catch (error) {
      setSharedContextStatus(`${agentName} did not accept the shared context: ${String(error)}`)
    } finally {
      setSharedContextSending(false)
    }
  }, [agentId, agentName, roomName, sharedContextValue])

  const listening = avatarState.voiceState === 'listening' || avatarState.microphoneEnabled
  const speaking = avatarState.voiceState === 'speaking'
  const thinking = avatarState.voiceState === 'thinking'

  return (
    <div className="flex h-full flex-col overflow-hidden bg-[#090f1a] text-slate-100">
      <div className="border-b border-white/10 bg-[radial-gradient(circle_at_top,#12324f_0%,rgba(7,13,24,0.96)_52%,rgba(7,13,24,1)_100%)] px-6 py-3">
        <div className="flex items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline" className="rounded-full border-white/12 bg-white/[0.04]">
              {projectName}
            </Badge>
            <div className="flex h-9 items-center gap-2 rounded-full border border-white/10 bg-white/[0.05] px-3 text-slate-200">
              <Avatar className="h-5 w-5 border border-white/10">
                <AvatarFallback className="bg-cyan-400/15 text-[9px] text-cyan-50">MO</AvatarFallback>
              </Avatar>
              <span className="text-sm font-medium text-slate-100">{agentName}</span>
            </div>
            <Badge variant="secondary" className="rounded-full">
              Call
            </Badge>
          </div>
          <Tabs value={panelMode} onValueChange={(value) => setPanelMode(value as 'live' | 'debug')}>
            <TabsList className="rounded-full border border-white/10 bg-white/[0.05]">
              <TabsTrigger value="live" className="rounded-full px-3 text-xs">
                Live
              </TabsTrigger>
              <TabsTrigger value="debug" className="rounded-full px-3 text-xs">
                Debug
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>

      <div
        className={`grid min-h-0 min-w-0 flex-1 gap-4 p-4 ${
          panelMode === 'debug'
            ? 'grid-cols-1 xl:grid-cols-[minmax(0,1fr)_minmax(300px,32vw)]'
            : 'grid-cols-1'
        }`}
      >
        <section className="min-h-0 min-w-0">
          <div className="h-full min-h-[300px] w-full sm:min-h-[340px] xl:min-h-[420px]">
            <MorganAvatarRoom
              compact
              autoConnect={false}
              tokenEndpoint={avatarTokenEndpoint}
              roomName={roomName}
              mediaMode="voice"
              onStateChange={setAvatarState}
            />
          </div>
        </section>

        {panelMode === 'debug' ? (
        <aside className="min-h-0 min-w-0 overflow-hidden rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-4">
          <Tabs value={panelMode} className="flex h-full min-h-0 flex-col">
            <TabsContent value="live" className="mt-0 min-h-0 flex-1 overflow-auto pr-1">
              <div className="space-y-4">
                <Card className="rounded-[24px] border-white/10 bg-black/15">
                  <CardHeader>
                    <CardTitle className="text-base text-white">Call state</CardTitle>
                  </CardHeader>
                  <CardContent className="grid gap-3">
                    <SignalRow
                      icon={<AudioLines className="h-4 w-4 text-cyan-200" />}
                      label="You"
                      detail={
                        avatarState.latestUserText
                          ? 'Transcript captured'
                          : listening
                            ? 'Push-to-talk live'
                            : 'Waiting for speech'
                      }
                      active={listening}
                      tone="emerald"
                    />
                    <SignalRow
                      icon={<Volume2 className="h-4 w-4 text-fuchsia-200" />}
                      label="Morgan"
                      detail={
                        speaking
                          ? 'Speaking now'
                          : thinking
                            ? 'Thinking'
                            : avatarState.latestAgentText
                              ? 'Reply ready'
                              : 'Waiting for reply'
                      }
                      active={speaking}
                      tone="violet"
                    />
                  </CardContent>
                </Card>

                <Card className="rounded-[24px] border-white/10 bg-black/15">
                  <CardHeader>
                    <CardTitle className="text-base text-white">Latest exchange</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <TranscriptPanel
                      label="You"
                      icon={<CircleDot className="h-3.5 w-3.5 text-emerald-200" />}
                      text={
                        avatarState.latestUserText ||
                        `Your latest utterance will appear here once ${agentName} hears you.`
                      }
                    />
                    <TranscriptPanel
                      label="Morgan"
                      icon={<Volume2 className="h-3.5 w-3.5 text-fuchsia-200" />}
                      text={
                        avatarState.latestAgentText ||
                        `${agentName}'s spoken reply will appear here as soon as a response is ready.`
                      }
                    />
                  </CardContent>
                </Card>

                <Card className="rounded-[24px] border-white/10 bg-black/15">
                  <CardHeader>
                    <CardTitle className="text-base text-white">Shared context</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <p className="text-sm leading-6 text-slate-300">
                      Paste a link or brief, then say the task naturally.
                    </p>
                    <Textarea
                      value={sharedContextValue}
                      onChange={(event) => setSharedContextValue(event.target.value)}
                      placeholder="Paste a URL, PRD excerpt, or structured notes for this call."
                      className="min-h-[112px] border-white/10 bg-black/20 text-slate-100 placeholder:text-slate-500"
                    />
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-xs text-slate-400">Targets room {roomName}.</p>
                      <Button
                        size="sm"
                        onClick={() => void sendSharedContext()}
                        disabled={sharedContextSending}
                      >
                        {sharedContextSending ? (
                          <>
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            Sending
                          </>
                        ) : (
                          `Send to ${agentName}`
                        )}
                      </Button>
                    </div>
                    {sharedContextStatus ? (
                      <div className="rounded-[18px] border border-white/10 bg-black/20 px-3 py-2 text-sm text-slate-200">
                        {sharedContextStatus}
                      </div>
                    ) : null}
                  </CardContent>
                </Card>
              </div>
            </TabsContent>

            <TabsContent value="debug" className="mt-0 min-h-0 flex-1 overflow-auto pr-1">
              <div className="space-y-4">
                <DebugCard
                  title="Session"
                  rows={[
                    ['Connection', avatarState.connectionState],
                    ['Voice state', avatarState.voiceState],
                    ['Room', avatarState.roomName ?? roomName],
                    ['Identity', avatarState.identity ?? 'guest'],
                    ['Audio', avatarState.audioTrackReady ? 'ready' : 'pending'],
                  ]}
                />
                <DebugCard
                  title="Backend"
                  rows={[
                    ['Healthy', diagnostics?.healthy ? 'yes' : 'unknown'],
                    ['Primary model', diagnostics?.modelPrimary ?? 'unresolved'],
                    ['Fallbacks', diagnostics?.modelFallbacks.join(', ') || 'none'],
                  ]}
                />
                {diagnostics?.recentErrors?.length ? (
                  <Card className="rounded-[24px] border-white/10 bg-black/15">
                    <CardHeader>
                      <CardTitle className="text-base text-white">Recent backend errors</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-2">
                      {diagnostics.recentErrors.map((entry) => (
                        <div
                          key={entry}
                          className="rounded-[18px] border border-white/8 bg-black/20 px-3 py-2 text-xs leading-6 text-slate-300"
                        >
                          {entry}
                        </div>
                      ))}
                    </CardContent>
                  </Card>
                ) : null}
              </div>
            </TabsContent>
          </Tabs>
        </aside>
        ) : null}
      </div>
    </div>
  )
}

function SignalRow({
  icon,
  label,
  detail,
  active,
  tone,
}: {
  icon: ReactNode
  label: string
  detail: string
  active: boolean
  tone: 'emerald' | 'violet'
}) {
  return (
    <div className="rounded-[20px] border border-white/10 bg-black/20 p-4">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="flex size-10 items-center justify-center rounded-full border border-white/10 bg-white/[0.05]">
            {icon}
          </div>
          <div>
            <p className="text-sm font-medium text-white">{label}</p>
            <p className="text-xs text-slate-400">{detail}</p>
          </div>
        </div>
        <div className="w-[108px]">
          <BarVisualizer state={active ? (tone === 'emerald' ? 'listening' : 'speaking') : undefined} className="h-10" />
        </div>
      </div>
    </div>
  )
}

function TranscriptPanel({
  label,
  icon,
  text,
}: {
  label: string
  icon: ReactNode
  text: string
}) {
  return (
    <div className="rounded-[20px] border border-white/10 bg-black/20 p-4">
      <div className="flex items-center gap-2 text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">
        {icon}
        {label}
      </div>
      <Separator className="my-3 bg-white/8" />
      <p className="text-sm leading-6 text-slate-200">{text}</p>
    </div>
  )
}

function DebugCard({
  title,
  rows,
}: {
  title: string
  rows: Array<[string, string]>
}) {
  return (
    <Card className="rounded-[24px] border-white/10 bg-black/15">
      <CardHeader>
        <CardTitle className="text-base text-white">{title}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {rows.map(([label, value]) => (
          <div key={label} className="flex items-start justify-between gap-3 text-sm">
            <span className="text-slate-400">{label}</span>
            <span className="max-w-[14rem] text-right text-slate-100">{value}</span>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
