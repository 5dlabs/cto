import React, { useCallback, useEffect, useRef, useState } from 'react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import {
  AlertCircle,
  AudioLines,
  Bot,
  CheckCircle2,
  Loader2,
  Plus,
  Send,
  User,
} from 'lucide-react'
import * as tauri from '@/lib/tauri'
import { getAgentBranding } from '@/lib/agent-branding'

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

interface AgentChatProps {
  sessionId?: string
  agentId?: string
  agentName?: string
  projectName?: string
  onOpenVoice?: () => void
}

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

function buildGreeting(agentName: string): ChatMessage {
  return {
    id: crypto.randomUUID(),
    role: 'agent',
    content: `${agentName} is ready.`,
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

export function AgentChat({
  sessionId: externalSessionId,
  agentId = 'morgan',
  agentName = 'Morgan',
  projectName,
  onOpenVoice,
}: AgentChatProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [inputValue, setInputValue] = useState('')
  const [agentStatus, setAgentStatus] = useState<AgentStatus>({
    state: 'idle',
    message: '',
  })
  const [historyLoaded, setHistoryLoaded] = useState(false)
  const [sessionId, setSessionId] = useState<string>(() =>
    externalSessionId ?? getPersistedSessionId()
  )
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

  useEffect(() => {
    if (externalSessionId && externalSessionId !== sessionId) {
      setSessionId(externalSessionId)
      setMessages([])
      setInputValue('')
      setAgentStatus({ state: 'idle', message: '' })
    }
  }, [externalSessionId, sessionId])

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
          setMessages([buildGreeting(agentName)])
        }
      } catch (error) {
        if (cancelled) {
          return
        }
        setMessages([
          buildGreeting(agentName),
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
  }, [agentName, sessionId])

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
    if (externalSessionId) {
      return
    }
    const nextSessionId = createSessionId()
    persistSessionId(nextSessionId)
    setSessionId(nextSessionId)
    setMessages([])
    setInputValue('')
    setAgentStatus({ state: 'idle', message: '' })
  }, [externalSessionId])

  const sendMessage = useCallback(async () => {
    const text = inputValue.trim()
    if (!text) return

    setInputValue('')
    addMessage('user', text)
    setAgentStatus({ state: 'thinking', message: `${agentName} is thinking...` })

    try {
      const response = await tauri.openclawSendMessage(sessionId, text, agentId)
      setAgentStatus({ state: 'idle', message: '' })
      addMessage('agent', response.content, response.action, {
        latencyMs: response.latencyMs,
        gatewayUrl: response.gatewayUrl,
      })
    } catch (error) {
      setAgentStatus({ state: 'idle', message: '' })
      addMessage('system', `${agentName} could not reply: ${String(error)}`)
    }
  }, [addMessage, agentId, agentName, inputValue, sessionId])

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

  const activeBranding = getAgentBranding(agentId)

  return (
    <div className="flex h-full flex-col bg-[radial-gradient(circle_at_top,#0f2033_0%,rgba(7,13,24,0.92)_44%,rgba(5,8,16,1)_100%)] text-white">
      <ScrollArea ref={scrollRef} className="flex-1 px-6 py-6">
        <div className="mx-auto flex max-w-3xl flex-col gap-4">
          {!historyLoaded && (
            <div className="flex items-center gap-2 rounded-2xl border border-white/10 bg-white/[0.045] px-4 py-3 text-sm text-slate-300">
              <Loader2 className="h-4 w-4 animate-spin" />
              Restoring thread
            </div>
          )}

          {messages.map((msg) => (
            <MessageBubble
              key={msg.id}
              message={msg}
              onActionClick={handleActionClick}
              avatarSrc={activeBranding.avatar}
              agentName={agentName}
            />
          ))}

          {agentStatus.state === 'thinking' && (
            <div className="flex items-start gap-3">
              <Avatar className="h-9 w-9 border border-sky-400/10 bg-sky-500/10">
                {activeBranding.avatar ? <AvatarImage src={activeBranding.avatar} alt={agentName} /> : null}
                <AvatarFallback className="bg-sky-500/10 text-sky-300">
                  <Bot className="h-4 w-4 text-sky-300" />
                </AvatarFallback>
              </Avatar>
              <div className="flex items-center gap-2 rounded-2xl rounded-tl-sm border border-white/10 bg-white/6 px-4 py-3">
                <Loader2 className="h-4 w-4 animate-spin text-sky-300" />
                <span className="text-sm text-slate-200">
                  {agentStatus.state === 'thinking' ? `${agentName} is thinking…` : agentStatus.message}
                </span>
              </div>
            </div>
          )}
        </div>
      </ScrollArea>

      <div className="border-t border-white/8 bg-black/10 p-5 backdrop-blur-xl">
        <div className="mx-auto max-w-3xl">
          <div className="rounded-[28px] border border-white/10 bg-[linear-gradient(180deg,rgba(8,15,28,0.98)_0%,rgba(5,10,20,0.98)_100%)] p-3 shadow-[0_24px_70px_-42px_rgba(34,211,238,0.45)]">
            <div className="flex items-end gap-3">
              <div className="flex-1 rounded-[22px] border border-white/10 bg-black/35 px-4 py-3 shadow-inner shadow-black/20">
                <textarea
                  ref={inputRef}
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder={`Message ${agentName}...`}
                  rows={1}
                  style={{ WebkitTextFillColor: 'rgb(241 245 249)' }}
                  className="min-h-[32px] w-full resize-none border-0 bg-transparent text-sm leading-7 text-slate-100 outline-none placeholder:text-slate-500 disabled:cursor-not-allowed disabled:opacity-50"
                  disabled={agentStatus.state === 'thinking'}
                />
              </div>

              {onOpenVoice ? (
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={onOpenVoice}
                  className="size-12 rounded-[20px] border border-white/10 bg-white/[0.04] text-slate-200 hover:bg-white/[0.08]"
                  title="Voice"
                >
                  <AudioLines className="h-5 w-5" />
                </Button>
              ) : null}

              {!externalSessionId ? (
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={startNewThread}
                  className="size-12 rounded-[20px] border border-white/10 bg-white/[0.04] text-slate-200 hover:bg-white/[0.08]"
                  title="New thread"
                >
                  <Plus className="h-5 w-5" />
                </Button>
              ) : null}

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

            <div className="mt-2 flex items-center justify-between gap-3 px-1 text-[11px] text-slate-500">
              <span>{projectName ?? agentName}</span>
              <span>Enter to send</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function MessageBubble({
  message,
  onActionClick,
  avatarSrc,
  agentName,
}: {
  message: ChatMessage
  onActionClick: (action: ChatAction) => void
  avatarSrc?: string
  agentName: string
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
        {!isUser && avatarSrc ? <AvatarImage src={avatarSrc} alt={agentName} /> : null}
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
