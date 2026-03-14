import { useEffect, useMemo, useState } from 'react'
import { AgentChat } from './components/AgentChat'
import { AgentsView } from './components/AgentsView'
import { AgentsStudioView } from './components/AgentsStudioView'
import { AppBootstrap } from './components/AppBootstrap'
import { ProjectsView } from './components/ProjectsView'
import { VoiceView } from './components/VoiceView'
import { Toaster } from './components/ui/toaster'
import { Badge } from './components/ui/badge'
import { Avatar, AvatarFallback, AvatarImage } from './components/ui/avatar'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './components/ui/select'
import {
  AudioLines,
  Blocks,
  Bot,
  MessageSquare,
  Video,
} from 'lucide-react'
import * as tauri from './lib/tauri'
import { getAgentBranding } from './lib/agent-branding'

type NavView = 'chat' | 'voice' | 'video' | 'agents' | 'projects'

interface NavItem {
  id: NavView
  label: string
  icon: typeof MessageSquare
}

const NAV_ITEMS: NavItem[] = [
  { id: 'chat', label: 'Chat', icon: MessageSquare },
  { id: 'voice', label: 'Call', icon: AudioLines },
  { id: 'video', label: 'Video', icon: Video },
  { id: 'agents', label: 'Agents', icon: Bot },
  { id: 'projects', label: 'Projects', icon: Blocks },
]

const VIEW_META: Record<
  NavView,
  {
    title: string
    badge: string
  }
> = {
  chat: {
    title: 'Morgan chat',
    badge: 'Text',
  },
  voice: {
    title: 'Morgan call',
    badge: 'Call',
  },
  video: {
    title: 'Morgan video',
    badge: 'Live',
  },
  agents: {
    title: 'Agent studio',
    badge: 'Studio',
  },
  projects: {
    title: 'Project hub',
    badge: 'Projects',
  },
}

function isConversationView(view: NavView): boolean {
  return view === 'chat' || view === 'voice' || view === 'video'
}

function createSharedSessionId(agentId: string, projectId: string): string {
  return `${agentId}-${projectId}`
}

function initialsForName(name: string): string {
  const compact = name.trim()
  if (!compact) return 'AG'
  const parts = compact.split(/\s+/).filter(Boolean)
  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase()
  }
  return `${parts[0][0] ?? ''}${parts[1][0] ?? ''}`.toUpperCase()
}

function App() {
  const [activeView, setActiveView] = useState<NavView>(() => {
    if (typeof window === 'undefined') {
      return 'video'
    }

    const hash = window.location.hash.replace(/^#/, '')
    if (hash && NAV_ITEMS.some((item) => item.id === hash)) {
      return hash as NavView
    }

    const savedView = window.localStorage.getItem('cto.activeView')
    if (savedView && NAV_ITEMS.some((item) => item.id === savedView)) {
      return savedView as NavView
    }

    return 'video'
  })
  const [studioState, setStudioState] = useState<tauri.StudioState | null>(null)
  const [studioError, setStudioError] = useState<string | null>(null)
  const [selectedConversationAgentId, setSelectedConversationAgentId] = useState('morgan')

  useEffect(() => {
    window.localStorage.setItem('cto.activeView', activeView)
    window.location.hash = activeView
  }, [activeView])

  useEffect(() => {
    let cancelled = false

    const loadStudioState = async () => {
      try {
        const next = await tauri.studioGetState()
        if (!cancelled) {
          setStudioState(next)
          setStudioError(null)
        }
      } catch (error) {
        if (!cancelled) {
          setStudioError(String(error))
        }
      }
    }

    void loadStudioState()

    return () => {
      cancelled = true
    }
  }, [])

  const handleStudioStateChange = (next: tauri.StudioState) => {
    setStudioState(next)
    setStudioError(null)
    void tauri.studioSaveState(next).catch((error) => {
      setStudioError(`Unable to save studio state: ${String(error)}`)
    })
  }

  const activeMeta = VIEW_META[activeView]
  useEffect(() => {
    if (!studioState?.agents?.length) return

    const enabledAgents = studioState.agents.filter((agent) => agent.enabled)
    if (enabledAgents.length === 0) return

    if (!enabledAgents.some((agent) => agent.id === selectedConversationAgentId)) {
      setSelectedConversationAgentId(enabledAgents[0].id)
    }
  }, [selectedConversationAgentId, studioState])

  const selectedProject =
    studioState?.projects.find((project) => project.id === studioState.selectedProjectId) ??
    studioState?.projects[0] ??
    null
  const selectedConversationAgent =
    studioState?.agents.find((agent) => agent.id === selectedConversationAgentId) ?? null
  const selectedConversationAgentName =
    selectedConversationAgent?.displayName ?? selectedConversationAgentId
  const selectedConversationAgentInitials = initialsForName(selectedConversationAgentName)
  const sharedSessionId = selectedProject
    ? createSharedSessionId(selectedConversationAgentId, selectedProject.id)
    : `${selectedConversationAgentId}-default`
  const sharedRoomName = sharedSessionId
  const selectedConversationBranding = getAgentBranding(selectedConversationAgentId)
  const activeConversationTitle =
    activeView === 'chat'
      ? `${selectedConversationAgentName} chat`
      : activeView === 'voice'
        ? `${selectedConversationAgentName} call`
        : activeView === 'video'
          ? `${selectedConversationAgentName} video`
          : activeMeta.title

  const shellBody = useMemo(() => {
    if (!studioState || !selectedProject) {
      return (
        <div className="flex h-full items-center justify-center">
          <div className="rounded-[28px] border border-white/10 bg-white/[0.04] px-6 py-5 text-sm text-slate-300">
            Loading CTO studio state…
          </div>
        </div>
      )
    }

    switch (activeView) {
      case 'chat':
        return (
          <AgentChat
            key={sharedSessionId}
            sessionId={sharedSessionId}
            agentId={selectedConversationAgentId}
            agentName={selectedConversationAgentName}
            projectName={selectedProject.name}
            onOpenVoice={() => setActiveView('voice')}
          />
        )
      case 'voice':
        return (
          <VoiceView
            key={sharedRoomName}
            agentId={selectedConversationAgentId}
            agentName={selectedConversationAgentName}
            projectName={selectedProject.name}
            roomName={sharedRoomName}
          />
        )
      case 'video':
        return (
          <AgentsView
            key={sharedRoomName}
            agentId={selectedConversationAgentId}
            agentName={selectedConversationAgentName}
            projectName={selectedProject.name}
            roomName={sharedRoomName}
          />
        )
      case 'agents':
        return (
          <AgentsStudioView
            state={studioState}
            selectedProject={selectedProject}
            onStateChange={handleStudioStateChange}
          />
        )
      case 'projects':
        return <ProjectsView state={studioState} onStateChange={handleStudioStateChange} />
    }
  }, [activeView, selectedProject, sharedRoomName, sharedSessionId, studioState])

  return (
    <div className="theme dark relative flex h-screen min-w-0 overflow-hidden bg-[radial-gradient(circle_at_top,#123456_0%,#08111d_26%,#04070d_100%)] text-foreground">
      <div className="absolute inset-0 bg-[linear-gradient(135deg,rgba(14,165,233,0.14)_0%,transparent_28%,transparent_72%,rgba(99,102,241,0.12)_100%)]" />

      <nav className="relative z-[1] flex w-[72px] shrink-0 flex-col items-center gap-3 border-r border-white/8 bg-[linear-gradient(180deg,rgba(7,15,28,0.98)_0%,rgba(4,8,14,0.98)_100%)] px-2 py-4">
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon
          const isActive = activeView === item.id
          return (
            <button
              key={item.id}
              onClick={() => setActiveView(item.id)}
              className={`group relative flex h-12 w-12 items-center justify-center rounded-2xl border transition ${
                isActive
                  ? 'border-cyan-300/20 bg-cyan-400/10 text-cyan-50 shadow-[0_18px_40px_-28px_rgba(34,211,238,0.5)]'
                  : 'border-transparent text-slate-500 hover:border-white/8 hover:bg-white/[0.05] hover:text-slate-100'
              }`}
              title={item.label}
            >
              {isActive ? (
                <span className="absolute left-0 top-1/2 h-8 w-1 -translate-y-1/2 rounded-full bg-cyan-300/80" />
              ) : null}
              <Icon className={`h-5 w-5 transition ${isActive ? 'scale-105' : 'group-hover:scale-105'}`} />
              <span className="sr-only">{item.label}</span>
            </button>
          )
        })}

        <div className="mt-auto h-2 w-2 rounded-full bg-cyan-300/70 shadow-[0_0_18px_rgba(34,211,238,0.65)]" />
      </nav>

      <div className="relative z-[1] flex min-w-0 flex-1 flex-col">
        <header className="border-b border-white/8 bg-black/10 px-6 py-3 backdrop-blur-xl">
          <div className="flex flex-wrap items-center gap-2.5">
            {!isConversationView(activeView) ? (
              <div className="flex min-w-0 items-center gap-2.5">
                <Badge variant="secondary" className="rounded-full">
                  {activeMeta.badge}
                </Badge>
                <h1 className="truncate text-base font-semibold tracking-tight text-white">
                  {activeConversationTitle}
                </h1>
              </div>
            ) : null}

            <div className={`flex flex-wrap items-center gap-2.5 ${isConversationView(activeView) ? '' : 'ml-auto'}`}>
              {isConversationView(activeView) && studioState?.projects?.length ? (
                <div className="min-w-[230px] rounded-[20px] border border-white/10 bg-white/[0.05] px-3 py-2.5">
                  <p className="text-[10px] uppercase tracking-[0.24em] text-slate-500">Project</p>
                  <Select
                    value={studioState.selectedProjectId}
                    onValueChange={(value) =>
                      handleStudioStateChange({ ...studioState, selectedProjectId: value })
                    }
                  >
                    <SelectTrigger className="mt-1.5 h-9 border-white/10 bg-black/20 text-slate-100">
                      <SelectValue placeholder="Choose a project" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectGroup>
                        {studioState.projects.map((project) => (
                          <SelectItem key={project.id} value={project.id}>
                            {project.name}
                          </SelectItem>
                        ))}
                      </SelectGroup>
                    </SelectContent>
                  </Select>
                </div>
              ) : null}

              {selectedProject && isConversationView(activeView) ? (
                <div className="flex h-10 items-center gap-2 rounded-full border border-white/10 bg-white/[0.05] px-3.5 text-slate-200">
                  {studioState?.agents?.length ? (
                    <Select
                      value={selectedConversationAgentId}
                      onValueChange={setSelectedConversationAgentId}
                    >
                      <SelectTrigger className="h-8 min-w-[180px] border-white/10 bg-black/20 text-slate-100">
                        <SelectValue placeholder="Choose an agent" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectGroup>
                          {studioState.agents
                            .filter((agent) => agent.enabled)
                            .map((agent) => (
                              <SelectItem key={agent.id} value={agent.id}>
                                {agent.displayName}
                              </SelectItem>
                            ))}
                        </SelectGroup>
                      </SelectContent>
                    </Select>
                  ) : null}
                  <Avatar className="h-6 w-6 border border-white/10">
                    {selectedConversationBranding.avatar ? (
                      <AvatarImage src={selectedConversationBranding.avatar} alt={selectedConversationAgentName} />
                    ) : null}
                    <AvatarFallback className="bg-cyan-400/15 text-[10px] text-cyan-50">
                      {selectedConversationAgentInitials}
                    </AvatarFallback>
                  </Avatar>
                  <span className="text-sm font-medium text-slate-100">{selectedConversationAgentName}</span>
                </div>
              ) : null}
            </div>
          </div>
          {studioError ? (
            <div className="mt-3 rounded-[18px] border border-amber-400/20 bg-amber-500/10 px-4 py-3 text-sm text-amber-100">
              {studioError}
            </div>
          ) : null}
        </header>

        <main className="min-h-0 min-w-0 flex-1 p-3 lg:p-5">
          <div className="h-full overflow-hidden rounded-[34px] border border-white/10 bg-[linear-gradient(180deg,rgba(9,16,28,0.96)_0%,rgba(5,8,16,0.98)_100%)] shadow-[0_32px_120px_-56px_rgba(8,145,178,0.55)]">
            {shellBody}
          </div>
        </main>
      </div>

      <Toaster />
      <AppBootstrap />
    </div>
  )
}

export default App
