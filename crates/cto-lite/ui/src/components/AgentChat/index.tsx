import React, { useCallback, useEffect, useRef, useState } from 'react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { LiveWaveform } from '@/components/ui/live-waveform'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Separator } from '@/components/ui/separator'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  AlertCircle,
  Bot,
  CheckCircle2,
  Loader2,
  Plus,
  Send,
  User,
  Wifi,
  WifiOff,
  Wrench,
} from 'lucide-react'
import * as tauri from '@/lib/tauri'

const CHAT_SESSION_STORAGE_KEY = 'cto.morganChatSessionId'

export interface ChatMessage {
  id: string
  role: 'user' | 'agent' | 'system'
  content: string
  timestamp: Date
  status?: 'sending' | 'delivered' | 'error'
  action?: ChatAction
  meta?: ChatMessageMeta
}

export interface ChatMessageMeta {
  latencyMs?: number
  gatewayUrl?: string
}

export interface ChatAction {
  type: 'oauth' | 'approve' | 'link' | 'confirm'
  label: string
  description?: string
  url?: string
  workflowId?: string
  completed?: boolean
}

interface AgentStatus {
  state: 'idle' | 'thinking' | 'working'
  message: string
}

type GatewayState = 'connecting' | 'online' | 'offline'

function createSessionId(): string {
  return crypto.randomUUID()
}

function getPersistedSessionId(): string {
  if (typeof window === 'undefined') {
    return createSessionId()
  }

  const existing = window.localStorage.getItem(CHAT_SESSION_STORAGE_KEY)
  if (existing) {
    return existing
  }

  const next = createSessionId()
  window.localStorage.setItem(CHAT_SESSION_STORAGE_KEY, next)
  return next
}

function persistSessionId(sessionId: string) {
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(CHAT_SESSION_STORAGE_KEY, sessionId)
  }
}

function buildGreeting(): ChatMessage {
  return {
    id: crypto.randomUUID(),
    role: 'agent',
    content: 'Morgan is ready on your local cluster. Ask for anything.',
    timestamp: new Date(),
  }
}

function mapHistoryMessage(
  message: tauri.OpenClawMessage,
  index: number,
  total: number
): ChatMessage {
  const secondsAgo = Math.max(total - index, 1)
  return {
    id: crypto.randomUUID(),
    role: message.role === 'user' ? 'user' : 'agent',
    content: message.content,
    timestamp: new Date(Date.now() - secondsAgo * 1000),
  }
}

export function AgentChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [inputValue, setInputValue] = useState('')
  const [agentStatus, setAgentStatus] = useState<AgentStatus>({
    state: 'idle',
    message: '',
  })
  const [gatewayState, setGatewayState] = useState<GatewayState>('connecting')
  const [gatewayVersion, setGatewayVersion] = useState<string | null>(null)
  const [historyLoaded, setHistoryLoaded] = useState(false)
  const [sessionId, setSessionId] = useState<string>(() => getPersistedSessionId())
  const scrollRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [messages, agentStatus.state])

  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  const syncGatewayStatus = useCallback(async () => {
    try {
      const status = await tauri.openclawGetStatus()
      setGatewayState(status.connected ? 'online' : 'offline')
      setGatewayVersion(status.version ?? null)
    } catch {
      setGatewayState('offline')
      setGatewayVersion(null)
    }
  }, [])

  useEffect(() => {
    let cancelled = false

    const loadHistory = async () => {
      setHistoryLoaded(false)
      try {
        const history = await tauri.openclawGetMessages(sessionId)
        if (cancelled) {
          return
        }

        if (history.length > 0) {
          setMessages(history.map((message, index) => mapHistoryMessage(message, index, history.length)))
        } else {
          setMessages([buildGreeting()])
        }
      } catch (error) {
        if (cancelled) {
          return
        }
        setMessages([
          buildGreeting(),
          {
            id: crypto.randomUUID(),
            role: 'system',
            content: `Unable to restore prior messages: ${String(error)}`,
            timestamp: new Date(),
          },
        ])
      } finally {
        if (!cancelled) {
          setHistoryLoaded(true)
        }
      }
    }

    void loadHistory()

    return () => {
      cancelled = true
    }
  }, [sessionId])

  useEffect(() => {
    let cancelled = false

    const refresh = async () => {
      if (cancelled) {
        return
      }
      await syncGatewayStatus()
    }

    void refresh()
    const intervalId = window.setInterval(() => {
      void refresh()
    }, 5000)

    return () => {
      cancelled = true
      window.clearInterval(intervalId)
    }
  }, [syncGatewayStatus])

  const addMessage = useCallback(
    (
      role: ChatMessage['role'],
      content: string,
      action?: ChatAction,
      meta?: ChatMessageMeta
    ) => {
      const msg: ChatMessage = {
        id: crypto.randomUUID(),
        role,
        content,
        timestamp: new Date(),
        action,
        meta,
      }
      setMessages((prev) => [...prev, msg])
      return msg
    },
    []
  )

  const startNewThread = useCallback(() => {
    const nextSessionId = createSessionId()
    persistSessionId(nextSessionId)
    setSessionId(nextSessionId)
    setMessages([])
    setInputValue('')
    setAgentStatus({ state: 'idle', message: '' })
  }, [])

  const sendMessage = useCallback(async () => {
    const text = inputValue.trim()
    if (!text) return

    setInputValue('')
    addMessage('user', text)
    setAgentStatus({ state: 'thinking', message: 'Morgan is thinking...' })

    try {
      const response = await tauri.openclawSendMessage(sessionId, text)
      setGatewayState('online')
      setAgentStatus({ state: 'idle', message: '' })
      addMessage('agent', response.content, response.action, {
        latencyMs: response.latencyMs,
        gatewayUrl: response.gatewayUrl,
      })
    } catch (error) {
      setAgentStatus({ state: 'idle', message: '' })
      setGatewayState('offline')
      addMessage('system', `Morgan could not reply: ${String(error)}`)
      void syncGatewayStatus()
    }
  }, [addMessage, inputValue, sessionId, syncGatewayStatus])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      void sendMessage()
    }
  }

  const handleActionClick = async (action: ChatAction) => {
    if (action.type === 'oauth' && action.url) {
      try {
        const { openUrl } = await import('@tauri-apps/plugin-opener')
        await openUrl(action.url)
        addMessage('system', `Opening ${action.label} in your browser...`)
      } catch (error) {
        addMessage('system', `Failed to open URL: ${String(error)}`)
      }
      return
    }

    if (action.type === 'approve' && action.workflowId) {
      try {
        await tauri.openclawApprove(action.workflowId)
        addMessage('system', 'Approval sent.')
      } catch (error) {
        addMessage('system', `Failed to approve: ${String(error)}`)
      }
    }
  }

  const gatewayTone =
    gatewayState === 'online'
      ? 'border-emerald-400/20 bg-emerald-500/10 text-emerald-100'
      : gatewayState === 'offline'
        ? 'border-rose-400/20 bg-rose-500/10 text-rose-100'
        : 'border-sky-400/20 bg-sky-500/10 text-sky-100'

  return (
    <div className="flex h-full flex-col bg-[radial-gradient(circle_at_top,#0f2033_0%,rgba(7,13,24,0.92)_44%,rgba(5,8,16,1)_100%)] text-white">
      <div className="border-b border-white/8 bg-black/10 px-6 py-5 backdrop-blur-xl">
        <div className="mx-auto grid max-w-5xl gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
          <Card className="overflow-hidden rounded-[28px] border-white/10 bg-white/[0.045] shadow-[0_28px_80px_-48px_rgba(34,211,238,0.45)]">
            <CardHeader className="gap-4 p-5">
              <div className="flex items-start gap-4">
                <Avatar
                  size="lg"
                  className="ring-1 ring-cyan-300/20 shadow-[0_18px_44px_-28px_rgba(34,211,238,0.65)]"
                >
                  <AvatarFallback className="bg-cyan-400/15 text-cyan-50">MO</AvatarFallback>
                </Avatar>
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge className={`rounded-full border ${gatewayTone}`}>
                      {gatewayState === 'online' ? (
                        <Wifi className="mr-1 h-3.5 w-3.5" />
                      ) : gatewayState === 'offline' ? (
                        <WifiOff className="mr-1 h-3.5 w-3.5" />
                      ) : (
                        <Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" />
                      )}
                      {gatewayState === 'online'
                        ? 'Local Morgan Connected'
                        : gatewayState === 'offline'
                          ? 'Gateway Offline'
                          : 'Checking Local Morgan'}
                    </Badge>
                    <Badge
                      variant="outline"
                      className="rounded-full border-white/12 bg-white/[0.04] text-slate-200"
                    >
                      Shared session
                    </Badge>
                  </div>
                  <CardTitle className="mt-4 text-[1.6rem] text-white">
                    Morgan chat stays on the local stack.
                  </CardTitle>
                  <CardDescription className="mt-2 max-w-2xl leading-6 text-slate-300">
                    Ask through text first, validate latency and turn quality, then compare that
                    exact flow with the avatar path. {gatewayVersion ? `Gateway ${gatewayVersion}.` : ''}
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
          </Card>

          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-1">
            <InsightTile
              label="Runtime"
              value={gatewayState === 'online' ? 'Healthy' : gatewayState === 'offline' ? 'Offline' : 'Checking'}
              detail="Morgan lives behind the local ingress in kind."
            />
            <InsightTile
              label="Thread"
              value={historyLoaded ? 'Restored' : 'Loading'}
              detail="The chat and avatar surfaces share the same backend persona."
              action={
                <Button variant="secondary" size="sm" onClick={startNewThread}>
                  <Plus data-icon="inline-start" />
                  New thread
                </Button>
              }
            />
          </div>
        </div>
      </div>

      <AgentStatusBar status={agentStatus} />

      <ScrollArea ref={scrollRef} className="flex-1 px-6 py-6">
        <div className="mx-auto flex max-w-5xl flex-col gap-4">
          {!historyLoaded && (
            <div className="flex items-center gap-2 rounded-2xl border border-white/10 bg-white/[0.045] px-4 py-3 text-sm text-slate-300">
              <Loader2 className="h-4 w-4 animate-spin" />
              Restoring the current Morgan thread.
            </div>
          )}

          {messages.map((msg) => (
            <MessageBubble
              key={msg.id}
              message={msg}
              onActionClick={handleActionClick}
            />
          ))}

          {agentStatus.state === 'thinking' && (
            <div className="flex items-start gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-sky-500/10">
                <Bot className="h-4 w-4 text-sky-300" />
              </div>
              <div className="flex items-center gap-2 rounded-2xl rounded-tl-sm border border-white/10 bg-white/6 px-4 py-3">
                <Loader2 className="h-4 w-4 animate-spin text-sky-300" />
                <span className="text-sm text-slate-200">{agentStatus.message}</span>
              </div>
            </div>
          )}
        </div>
      </ScrollArea>

      <div className="border-t border-white/8 bg-black/10 p-5 backdrop-blur-xl">
        <div className="mx-auto max-w-5xl">
          <div className="rounded-[30px] border border-white/10 bg-[linear-gradient(180deg,rgba(8,15,28,0.98)_0%,rgba(5,10,20,0.98)_100%)] p-4 shadow-[0_24px_70px_-42px_rgba(34,211,238,0.45)]">
            <div className="flex flex-wrap items-center gap-4">
              <Avatar className="ring-1 ring-white/10">
                <AvatarFallback className="bg-white/10 text-slate-100">Y</AvatarFallback>
              </Avatar>
              <div className="min-w-0 flex-1">
                <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">
                  Composer
                </p>
                <p className="mt-1 text-sm text-slate-300">
                  Ask through text, validate the reply, then compare it against the avatar turn.
                </p>
              </div>
              <Badge variant="secondary" className="rounded-full">
                Text path
              </Badge>
            </div>

            <Separator className="my-4 bg-white/8" />

            <div className="overflow-hidden rounded-[22px] border border-white/8 bg-slate-950/70 px-3 py-2">
              <LiveWaveform
                active={false}
                processing={agentStatus.state === 'thinking'}
                mode="static"
                height={40}
                fadeEdges={false}
                barWidth={3}
                barGap={2}
                className="h-10 rounded-[16px] bg-transparent"
              />
            </div>

            <div className="mt-4 flex items-end gap-3">
              <div className="flex-1 rounded-[24px] border border-white/10 bg-black/35 px-4 py-3 shadow-inner shadow-black/20">
                <textarea
                  ref={inputRef}
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Message Morgan..."
                  rows={1}
                  style={{ WebkitTextFillColor: 'rgb(241 245 249)' }}
                  className="min-h-[32px] w-full resize-none border-0 bg-transparent text-sm leading-7 text-slate-100 outline-none placeholder:text-slate-500 disabled:cursor-not-allowed disabled:opacity-50"
                  disabled={agentStatus.state === 'thinking'}
                />
              </div>

              <Button
                className="size-14 rounded-[22px] shadow-[0_18px_40px_-26px_rgba(34,211,238,0.85)]"
                size="icon"
                onClick={() => {
                  void sendMessage()
                }}
                disabled={!inputValue.trim() || agentStatus.state === 'thinking'}
              >
                <Send />
              </Button>
            </div>

            <div className="mt-4 flex flex-wrap items-center justify-between gap-3 px-1 text-[11px] uppercase tracking-[0.22em] text-slate-500">
              <span>Enter to send</span>
              <span>Shift+Enter for newline</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function AgentStatusBar({ status }: { status: AgentStatus }) {
  if (status.state === 'idle') return null

  const icon =
    status.state === 'thinking' ? (
      <Loader2 className="h-3.5 w-3.5 animate-spin" />
    ) : (
      <Wrench className="h-3.5 w-3.5 animate-pulse" />
    )

  return (
    <div className="flex items-center gap-2 border-b border-sky-500/15 bg-sky-500/10 px-6 py-2 text-xs text-sky-100">
      {icon}
      <span>{status.message}</span>
    </div>
  )
}

function MessageBubble({
  message,
  onActionClick,
}: {
  message: ChatMessage
  onActionClick: (action: ChatAction) => void
}) {
  const isUser = message.role === 'user'
  const isSystem = message.role === 'system'

  if (isSystem) {
    return (
      <div className="flex justify-center">
        <div className="flex items-center gap-2 rounded-full border border-amber-400/20 bg-amber-500/10 px-3 py-1.5 text-xs text-amber-100">
          <AlertCircle className="h-3.5 w-3.5" />
          {message.content}
        </div>
      </div>
    )
  }

  return (
    <div className={`flex items-start gap-3 ${isUser ? 'flex-row-reverse' : ''}`}>
      <Avatar
        className={
          isUser
            ? 'bg-white text-slate-950 ring-1 ring-white/20'
            : 'bg-sky-500/10 ring-1 ring-sky-400/10'
        }
      >
        <AvatarFallback className={isUser ? 'bg-white text-slate-950' : 'bg-sky-500/10 text-sky-300'}>
        {isUser ? (
          <User className="h-4 w-4" />
        ) : (
          <Bot className="h-4 w-4 text-sky-300" />
        )}
        </AvatarFallback>
      </Avatar>

      <div className={`flex max-w-[82%] flex-col gap-1 ${isUser ? 'items-end' : ''}`}>
        <div
          className={`rounded-2xl px-4 py-3 text-sm leading-relaxed ${
            isUser
              ? 'rounded-tr-sm bg-white text-slate-950 shadow-[0_18px_44px_-32px_rgba(255,255,255,0.45)]'
              : 'rounded-tl-sm border border-white/10 bg-white/[0.055] text-slate-100'
          }`}
        >
          {message.content}
        </div>

        {message.meta?.latencyMs ? (
          <div className="flex flex-wrap items-center gap-2 px-1 text-[10px] uppercase tracking-[0.22em] text-slate-500">
            <span>{formatLatency(message.meta.latencyMs)}</span>
            {message.meta.gatewayUrl ? <span>{formatGateway(message.meta.gatewayUrl)}</span> : null}
          </div>
        ) : null}

        {message.action && (
          <ActionCard action={message.action} onClick={onActionClick} />
        )}

        <span className="px-1 text-[10px] text-slate-500">
          {message.timestamp.toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
          })}
        </span>
      </div>
    </div>
  )
}

function formatLatency(latencyMs: number): string {
  if (latencyMs >= 1000) {
    return `${(latencyMs / 1000).toFixed(1)}s`
  }

  return `${latencyMs}ms`
}

function formatGateway(gatewayUrl: string): string {
  try {
    return new URL(gatewayUrl).host
  } catch {
    return gatewayUrl
  }
}

function ActionCard({
  action,
  onClick,
}: {
  action: ChatAction
  onClick: (action: ChatAction) => void
}) {
  return (
    <button
      onClick={() => onClick(action)}
      disabled={action.completed}
      className={`flex w-full max-w-sm items-center gap-3 rounded-xl border px-4 py-3 text-left transition-colors ${
        action.completed
          ? 'cursor-default border-emerald-400/20 bg-emerald-500/5'
          : 'border-white/10 bg-white/6 hover:bg-white/10'
      }`}
    >
      <div className="shrink-0">
        {action.completed ? (
          <CheckCircle2 className="h-5 w-5 text-emerald-300" />
        ) : action.type === 'approve' ? (
          <Badge variant="outline" className="text-xs">
            Approve
          </Badge>
        ) : (
          <Badge variant="secondary" className="text-xs">
            Action
          </Badge>
        )}
      </div>
      <div className="min-w-0">
        <div className="truncate text-sm font-medium">{action.label}</div>
        {action.description && (
          <div className="truncate text-xs text-slate-400">{action.description}</div>
        )}
      </div>
    </button>
  )
}

function InsightTile({
  label,
  value,
  detail,
  action,
}: {
  label: string
  value: string
  detail: string
  action?: React.ReactNode
}) {
  return (
    <Card className="rounded-[24px] border-white/10 bg-white/[0.045]">
      <CardContent className="flex h-full flex-col gap-3 p-4">
        <div>
          <p className="text-[11px] uppercase tracking-[0.26em] text-slate-500">{label}</p>
          <p className="mt-2 text-lg font-semibold text-white">{value}</p>
          <p className="mt-1 text-sm leading-6 text-slate-300">{detail}</p>
        </div>
        {action ? <div className="pt-1">{action}</div> : null}
      </CardContent>
    </Card>
  )
}
