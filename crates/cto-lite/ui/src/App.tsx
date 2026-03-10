import { useEffect, useState } from 'react'
import { AgentChat } from './components/AgentChat'
import { AgentsView } from './components/AgentsView'
import { AppBootstrap } from './components/AppBootstrap'
import { PrdView } from './components/PrdView'
import { ConfigView } from './components/ConfigView'
import { Dashboard } from './components/Dashboard'
import { Toaster } from './components/ui/toaster'
import { Badge } from './components/ui/badge'
import { Avatar, AvatarFallback } from './components/ui/avatar'
import { Separator } from './components/ui/separator'
import {
  MessageSquare,
  AudioLines,
  FileText,
  Settings,
  LayoutDashboard,
  Sparkles,
  CircleDot,
} from 'lucide-react'

// ============================================================================
// Navigation types
// ============================================================================

type NavView = 'chat' | 'avatar' | 'prds' | 'config' | 'workflows'

interface NavItem {
  id: NavView
  label: string
  icon: typeof MessageSquare
}

const NAV_ITEMS: NavItem[] = [
  { id: 'chat', label: 'Chat', icon: MessageSquare },
  { id: 'avatar', label: 'Avatar', icon: AudioLines },
  { id: 'prds', label: 'PRDs', icon: FileText },
  { id: 'workflows', label: 'Workflows', icon: LayoutDashboard },
  { id: 'config', label: 'Config', icon: Settings },
]

const VIEW_META: Record<
  NavView,
  {
    title: string
    description: string
    badge: string
  }
> = {
  chat: {
    title: 'Morgan Chat',
    description: 'Fast local chat against the same Morgan runtime running in the private cluster.',
    badge: 'Text',
  },
  avatar: {
    title: 'Morgan Avatar',
    description: 'A call-style live surface for Morgan with voice, video, transcript, and runtime status.',
    badge: 'Live',
  },
  prds: {
    title: 'PRD Workspace',
    description: 'Keep product docs, decomposition, and implementation handoff material in one place.',
    badge: 'Docs',
  },
  workflows: {
    title: 'Workflow Control',
    description: 'Track local automations, background operations, and the orchestration path behind CTO.',
    badge: 'Ops',
  },
  config: {
    title: 'Configuration',
    description: 'Operational settings for the desktop app, runtime, and future secrets management.',
    badge: 'Setup',
  },
}

// ============================================================================
// App
// ============================================================================

function App() {
  const [activeView, setActiveView] = useState<NavView>(() => {
    if (typeof window === 'undefined') {
      return 'avatar'
    }

    const hash = window.location.hash.replace(/^#/, '')
    if (hash && NAV_ITEMS.some((item) => item.id === hash)) {
      return hash as NavView
    }

    const savedView = window.localStorage.getItem('cto.activeView')
    if (savedView && NAV_ITEMS.some((item) => item.id === savedView)) {
      return savedView as NavView
    }

    return 'avatar'
  })

  useEffect(() => {
    window.localStorage.setItem('cto.activeView', activeView)
    window.location.hash = activeView
  }, [activeView])

  const activeMeta = VIEW_META[activeView]

  return (
    <div className="theme dark relative flex h-screen overflow-hidden bg-[radial-gradient(circle_at_top,#123456_0%,#08111d_26%,#04070d_100%)] text-foreground">
      <div className="absolute inset-0 bg-[linear-gradient(135deg,rgba(14,165,233,0.14)_0%,transparent_28%,transparent_72%,rgba(99,102,241,0.12)_100%)]" />

      <nav className="relative z-[1] flex w-[112px] flex-col gap-4 border-r border-white/8 bg-[linear-gradient(180deg,rgba(7,15,28,0.98)_0%,rgba(4,8,14,0.98)_100%)] px-3 py-4">
        <div className="rounded-[28px] border border-white/10 bg-white/[0.045] p-3 shadow-[0_24px_60px_-36px_rgba(34,211,238,0.6)]">
          <div className="flex flex-col items-center gap-3">
            <Avatar
              size="lg"
              className="ring-1 ring-cyan-300/20 shadow-[0_18px_44px_-28px_rgba(34,211,238,0.75)]"
            >
              <AvatarFallback className="bg-cyan-400/15 text-cyan-50">
                <Sparkles className="size-4" />
              </AvatarFallback>
            </Avatar>
            <div className="text-center">
              <p className="text-[11px] font-semibold uppercase tracking-[0.28em] text-cyan-100/70">
                CTO
              </p>
              <p className="mt-1 text-[10px] text-slate-400">Desktop</p>
            </div>
          </div>
        </div>

        {NAV_ITEMS.map((item) => {
          const Icon = item.icon
          const isActive = activeView === item.id
          return (
            <button
              key={item.id}
              onClick={() => setActiveView(item.id)}
              className={`group relative flex h-16 w-full flex-col items-center justify-center gap-1.5 rounded-[22px] border transition ${
                isActive
                  ? 'border-cyan-300/20 bg-cyan-400/10 text-cyan-50 shadow-[0_18px_40px_-28px_rgba(34,211,238,0.5)]'
                  : 'border-transparent text-slate-500 hover:border-white/8 hover:bg-white/[0.05] hover:text-slate-100'
              }`}
              title={item.label}
            >
              {isActive ? (
                <span className="absolute left-0 top-1/2 h-8 w-1 -translate-y-1/2 rounded-full bg-cyan-300/80" />
              ) : null}
              <Icon
                className={`h-5 w-5 transition ${isActive ? 'scale-105' : 'group-hover:scale-105'}`}
              />
              <span className="text-[9px] font-medium uppercase tracking-[0.24em] leading-none">
                {item.label}
              </span>
            </button>
          )
        })}

        <div className="mt-auto rounded-[24px] border border-white/10 bg-white/[0.045] p-3">
          <div className="flex flex-col gap-3">
            <Badge variant="secondary" className="justify-center rounded-full">
              Local Stack
            </Badge>
            <div className="text-center">
              <p className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                Runtime
              </p>
              <p className="mt-1 text-xs text-slate-200">Kind + Morgan</p>
            </div>
          </div>
        </div>
      </nav>

      <div className="relative z-[1] flex min-w-0 flex-1 flex-col">
        <header className="border-b border-white/8 bg-black/10 px-6 py-5 backdrop-blur-xl">
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex min-w-0 items-center gap-4">
              <Avatar
                size="lg"
                className="ring-1 ring-white/10 shadow-[0_18px_40px_-28px_rgba(15,23,42,0.9)]"
              >
                <AvatarFallback className="bg-white/10 text-slate-100">MO</AvatarFallback>
              </Avatar>
              <div className="min-w-0">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant="secondary" className="rounded-full">
                    {activeMeta.badge}
                  </Badge>
                  <Badge variant="outline" className="rounded-full border-white/12 bg-white/[0.04]">
                    Morgan-first
                  </Badge>
                </div>
                <h1 className="mt-3 truncate text-2xl font-semibold tracking-tight text-white">
                  {activeMeta.title}
                </h1>
                <p className="mt-1 max-w-2xl text-sm leading-6 text-slate-300">
                  {activeMeta.description}
                </p>
              </div>
            </div>

            <Separator orientation="vertical" className="hidden h-14 bg-white/8 xl:block" />

            <div className="ml-auto flex flex-wrap items-center gap-3">
              <div className="rounded-[22px] border border-white/10 bg-white/[0.05] px-4 py-3">
                <p className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                  Workspace
                </p>
                <p className="mt-1 text-sm text-slate-100">/Users/jonathon/5dlabs/cto</p>
              </div>
              <div className="rounded-[22px] border border-white/10 bg-white/[0.05] px-4 py-3">
                <p className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                  Surface
                </p>
                <div className="mt-1 flex items-center gap-2 text-sm text-slate-100">
                  <CircleDot className="size-3 text-cyan-300" />
                  Desktop MVP
                </div>
              </div>
            </div>
          </div>
        </header>

        <main className="min-h-0 flex-1 p-4 lg:p-5">
          <div className="h-full overflow-hidden rounded-[34px] border border-white/10 bg-[linear-gradient(180deg,rgba(9,16,28,0.96)_0%,rgba(5,8,16,0.98)_100%)] shadow-[0_32px_120px_-56px_rgba(8,145,178,0.55)]">
            {activeView === 'chat' && <AgentChat />}
            {activeView === 'avatar' && <AgentsView />}
            {activeView === 'prds' && <PrdView />}
            {activeView === 'config' && <ConfigView />}
            {activeView === 'workflows' && <Dashboard />}
          </div>
        </main>
      </div>

      <Toaster />
      <AppBootstrap />
    </div>
  )
}

export default App
