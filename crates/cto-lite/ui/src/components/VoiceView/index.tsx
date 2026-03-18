import { useCallback, useEffect, useState } from 'react'
import { Bug, BugOff } from 'lucide-react'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { MorganDebugPanel } from '@/components/MorganDebugPanel'
import {
  MorganAvatarRoom,
  type MorganAvatarState,
} from '@/components/MorganAvatarRoom'
import { SharedContextComposer } from '@/components/SharedContextComposer'
import type { MorganSessionState } from '@/lib/morgan-session'
import * as tauri from '@/lib/tauri'

type VoiceViewProps = {
  agentId: string
  agentName: string
  projectName: string
  roomName: string
  sharedSession?: MorganSessionState | null
  onSessionStateChange?: (patch: Partial<MorganSessionState>) => void
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

export function VoiceView({
  agentId,
  agentName,
  projectName,
  roomName,
  sharedSession,
  onSessionStateChange,
}: VoiceViewProps) {
  const avatarTokenEndpoint =
    ((import.meta as { env?: Record<string, string> }).env?.VITE_MORGAN_AVATAR_TOKEN_ENDPOINT as string) ??
    '/avatar-api/token'
  const [avatarState, setAvatarState] = useState<MorganAvatarState>(DEFAULT_AVATAR_STATE)
  const [showDebug, setShowDebug] = useState(false)
  const [sharedContextValue, setSharedContextValue] = useState('')
  const [sharedContextStatus, setSharedContextStatus] = useState<string | null>(null)
  const [sharedContextSending, setSharedContextSending] = useState(false)
  const [diagnostics, setDiagnostics] = useState<tauri.MorganDiagnostics | null>(null)
  const [localHealth, setLocalHealth] = useState<tauri.LocalMorganHealth | null>(null)

  useEffect(() => {
    let cancelled = false

    const syncDiagnostics = async () => {
      const [health, nextDiagnostics] = await Promise.all([
        tauri.openclawGetLocalHealth().catch(() => null),
        tauri.openclawGetMorganDiagnostics(agentId).catch(() => null),
      ])

      if (!cancelled) {
        setLocalHealth(health)
        setDiagnostics(nextDiagnostics)
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
    const activeRoomName = (avatarState.roomName ?? roomName)?.trim()
    const content = sharedContextValue.trim()

    if (!activeRoomName) {
      setSharedContextStatus('Start a call first so CTO can target the active room.')
      return
    }

    if (!content) {
      setSharedContextStatus('Paste a URL or brief before sending context.')
      return
    }

    setSharedContextSending(true)
    setSharedContextStatus(null)

    try {
      const response = await tauri.openclawSendAvatarContext(activeRoomName, content, agentId)
      setSharedContextStatus(
        response.content === 'CONTEXT_STORED'
          ? `Shared with ${agentName}.`
          : response.content
      )
      setSharedContextValue('')
      onSessionStateChange?.({
        gatewaySessionKey: response.gatewaySessionKey ?? sharedSession?.gatewaySessionKey ?? null,
        latestTransport: 'call',
      })
    } catch (error) {
      setSharedContextStatus(
        `${agentName} did not accept the shared context: ${tauri.getErrorMessage(error)}`
      )
    } finally {
      setSharedContextSending(false)
    }
  }, [
    agentId,
    agentName,
    avatarState.roomName,
    onSessionStateChange,
    roomName,
    sharedContextValue,
    sharedSession?.gatewaySessionKey,
  ])

  const handleAvatarStateChange = useCallback(
    (nextState: MorganAvatarState) => {
      setAvatarState(nextState)
      onSessionStateChange?.({
        roomName: nextState.roomName ?? roomName,
        latestUserText: nextState.latestUserText,
        latestAgentText: nextState.latestAgentText,
        connectionState: nextState.connectionState,
        voiceState: nextState.voiceState,
        latestTransport: 'call',
      })
    },
    [onSessionStateChange, roomName]
  )

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

          <Button
            variant={showDebug ? 'secondary' : 'outline'}
            size="sm"
            onClick={() => setShowDebug((current) => !current)}
          >
            {showDebug ? (
              <>
                <BugOff className="mr-2 h-4 w-4" />
                Hide debug
              </>
            ) : (
              <>
                <Bug className="mr-2 h-4 w-4" />
                Debug
              </>
            )}
          </Button>
        </div>
      </div>

      <div
        className={`grid min-h-0 min-w-0 flex-1 gap-4 p-4 ${
          showDebug
            ? 'grid-cols-1 xl:grid-cols-[minmax(0,1fr)_minmax(320px,32vw)]'
            : 'grid-cols-1'
        }`}
      >
        <section className="flex min-h-0 min-w-0 flex-col gap-4">
          <div className="min-h-[300px] flex-1 sm:min-h-[340px] xl:min-h-[420px]">
            <MorganAvatarRoom
              compact
              autoConnect={false}
              tokenEndpoint={avatarTokenEndpoint}
              roomName={roomName}
              mediaMode="voice"
              onStateChange={handleAvatarStateChange}
            />
          </div>

          <SharedContextComposer
            agentName={agentName}
            roomName={avatarState.roomName ?? roomName ?? null}
            value={sharedContextValue}
            status={sharedContextStatus}
            sending={sharedContextSending}
            onValueChange={setSharedContextValue}
            onSend={() => void sendSharedContext()}
          />
        </section>

        {showDebug ? (
          <aside className="min-h-0 min-w-0 overflow-hidden rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-4">
            <MorganDebugPanel
              health={localHealth}
              diagnostics={diagnostics}
              avatarState={avatarState}
              roomName={roomName}
            />
          </aside>
        ) : null}
      </div>
    </div>
  )
}
