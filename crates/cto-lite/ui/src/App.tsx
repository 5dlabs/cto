import { useState } from 'react'
import { AgentChat } from './components/AgentChat'
import { AgentsView } from './components/AgentsView'
import { PrdView } from './components/PrdView'
import { ConfigView } from './components/ConfigView'
import { Dashboard } from './components/Dashboard'
import { Toaster } from './components/ui/toaster'
import {
  MessageSquare,
  AudioLines,
  FileText,
  Settings,
  LayoutDashboard,
  Sparkles,
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

// ============================================================================
// App
// ============================================================================

function App() {
  const [activeView, setActiveView] = useState<NavView>('chat')

  return (
    <div className="flex h-screen bg-background">
      {/* Left sidebar navigation */}
      <nav className="w-16 border-r flex flex-col items-center py-4 gap-1 bg-muted/30">
        {/* Logo */}
        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-blue-600 flex items-center justify-center mb-4">
          <Sparkles className="h-5 w-5 text-white" />
        </div>

        {/* Nav items */}
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon
          const isActive = activeView === item.id
          return (
            <button
              key={item.id}
              onClick={() => setActiveView(item.id)}
              className={`w-12 h-12 rounded-xl flex flex-col items-center justify-center gap-0.5 transition-colors ${
                isActive
                  ? 'bg-primary/10 text-primary'
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted'
              }`}
              title={item.label}
            >
              <Icon className="h-5 w-5" />
              <span className="text-[9px] font-medium leading-none">
                {item.label}
              </span>
            </button>
          )
        })}
      </nav>

      {/* Main content area */}
      <main className="flex-1 overflow-hidden">
        {activeView === 'chat' && <AgentChat />}
        {activeView === 'avatar' && <AgentsView />}
        {activeView === 'prds' && <PrdView />}
        {activeView === 'config' && <ConfigView />}
        {activeView === 'workflows' && <Dashboard />}
      </main>

      <Toaster />
    </div>
  )
}

export default App
