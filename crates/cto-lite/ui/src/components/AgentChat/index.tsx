import React, { useState, useRef, useEffect, useCallback } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import {
  Send,
  Loader2,
  Bot,
  User,
  CheckCircle2,
  AlertCircle,
  Clock,
  Wrench,
} from 'lucide-react'
import * as tauri from '@/lib/tauri'

// ============================================================================
// Types
// ============================================================================

export interface ChatMessage {
  id: string
  role: 'user' | 'agent' | 'system'
  content: string
  timestamp: Date
  status?: 'sending' | 'delivered' | 'error'
  /** Optional structured action card (e.g., "Click to authorize GitHub") */
  action?: ChatAction
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
  state: 'idle' | 'thinking' | 'working' | 'waiting'
  message: string
}

// ============================================================================
// Component
// ============================================================================

export function AgentChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [inputValue, setInputValue] = useState('')
  const [agentStatus, setAgentStatus] = useState<AgentStatus>({
    state: 'idle',
    message: '',
  })
  const [sessionId] = useState(() => crypto.randomUUID())
  const scrollRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [messages])

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  // Initial greeting from PM agent
  useEffect(() => {
    const greeting: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'agent',
      content:
        "Hello! I'm Morgan, your PM agent. I'll help you set up and manage your development environment. What would you like to do?",
      timestamp: new Date(),
    }
    setMessages([greeting])
  }, [])

  const addMessage = useCallback(
    (role: ChatMessage['role'], content: string, action?: ChatAction) => {
      const msg: ChatMessage = {
        id: crypto.randomUUID(),
        role,
        content,
        timestamp: new Date(),
        action,
      }
      setMessages((prev) => [...prev, msg])
      return msg
    },
    []
  )

  const sendMessage = useCallback(async () => {
    const text = inputValue.trim()
    if (!text) return

    setInputValue('')
    addMessage('user', text)

    setAgentStatus({ state: 'thinking', message: 'Morgan is thinking...' })

    try {
      const response = await tauri.openclawSendMessage(sessionId, text)
      setAgentStatus({ state: 'idle', message: '' })
      addMessage('agent', response.content, response.action)
    } catch (error) {
      setAgentStatus({ state: 'idle', message: '' })
      addMessage(
        'system',
        `Failed to get response: ${String(error)}. The OpenClaw gateway may not be running yet.`
      )
    }
  }, [inputValue, sessionId, addMessage])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
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
    } else if (action.type === 'approve' && action.workflowId) {
      try {
        await tauri.openclawApprove(action.workflowId)
        addMessage('system', 'Approval sent.')
      } catch (error) {
        addMessage('system', `Failed to approve: ${String(error)}`)
      }
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Agent status bar */}
      <AgentStatusBar status={agentStatus} />

      {/* Messages area */}
      <ScrollArea ref={scrollRef} className="flex-1 px-4 py-3">
        <div className="max-w-3xl mx-auto space-y-4">
          {messages.map((msg) => (
            <MessageBubble
              key={msg.id}
              message={msg}
              onActionClick={handleActionClick}
            />
          ))}

          {agentStatus.state === 'thinking' && (
            <div className="flex items-start gap-3">
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                <Bot className="h-4 w-4 text-primary" />
              </div>
              <div className="flex items-center gap-2 py-2 px-3 bg-muted rounded-2xl rounded-tl-sm">
                <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                <span className="text-sm text-muted-foreground">
                  {agentStatus.message}
                </span>
              </div>
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Input area */}
      <div className="border-t bg-background p-4">
        <div className="max-w-3xl mx-auto flex gap-2">
          <Input
            ref={inputRef}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Message Morgan..."
            className="flex-1"
            disabled={agentStatus.state === 'thinking'}
          />
          <Button
            size="icon"
            onClick={sendMessage}
            disabled={!inputValue.trim() || agentStatus.state === 'thinking'}
          >
            <Send className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  )
}

// ============================================================================
// Sub-components
// ============================================================================

function AgentStatusBar({ status }: { status: AgentStatus }) {
  if (status.state === 'idle') return null

  const icons = {
    thinking: <Loader2 className="h-3 w-3 animate-spin" />,
    working: <Wrench className="h-3 w-3 animate-pulse" />,
    waiting: <Clock className="h-3 w-3" />,
    idle: null,
  }

  const colors = {
    thinking: 'bg-blue-500/10 text-blue-600 border-blue-500/20',
    working: 'bg-amber-500/10 text-amber-600 border-amber-500/20',
    waiting: 'bg-purple-500/10 text-purple-600 border-purple-500/20',
    idle: '',
  }

  return (
    <div
      className={`flex items-center gap-2 px-4 py-2 text-xs border-b ${colors[status.state]}`}
    >
      {icons[status.state]}
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
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-muted text-xs text-muted-foreground">
          <AlertCircle className="h-3 w-3" />
          {message.content}
        </div>
      </div>
    )
  }

  return (
    <div className={`flex items-start gap-3 ${isUser ? 'flex-row-reverse' : ''}`}>
      {/* Avatar */}
      <div
        className={`w-8 h-8 rounded-full flex items-center justify-center shrink-0 ${
          isUser
            ? 'bg-primary text-primary-foreground'
            : 'bg-primary/10'
        }`}
      >
        {isUser ? (
          <User className="h-4 w-4" />
        ) : (
          <Bot className="h-4 w-4 text-primary" />
        )}
      </div>

      {/* Content */}
      <div className={`flex flex-col gap-1 max-w-[80%] ${isUser ? 'items-end' : ''}`}>
        <div
          className={`px-4 py-2.5 rounded-2xl text-sm leading-relaxed ${
            isUser
              ? 'bg-primary text-primary-foreground rounded-tr-sm'
              : 'bg-muted rounded-tl-sm'
          }`}
        >
          {message.content}
        </div>

        {/* Action card */}
        {message.action && (
          <ActionCard action={message.action} onClick={onActionClick} />
        )}

        {/* Timestamp */}
        <span className="text-[10px] text-muted-foreground px-1">
          {message.timestamp.toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
          })}
        </span>
      </div>
    </div>
  )
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
      className={`flex items-center gap-3 px-4 py-3 rounded-xl border text-left transition-colors w-full max-w-sm ${
        action.completed
          ? 'bg-green-500/5 border-green-500/20 cursor-default'
          : 'bg-card hover:bg-accent cursor-pointer border-border'
      }`}
    >
      <div className="shrink-0">
        {action.completed ? (
          <CheckCircle2 className="h-5 w-5 text-green-500" />
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
        <div className="font-medium text-sm truncate">{action.label}</div>
        {action.description && (
          <div className="text-xs text-muted-foreground truncate">
            {action.description}
          </div>
        )}
      </div>
    </button>
  )
}
