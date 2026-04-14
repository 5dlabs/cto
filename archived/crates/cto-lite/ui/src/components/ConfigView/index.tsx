import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Bot,
  Wrench,
  Code2,
  Shield,
  TestTube2,
  Rocket,
  Eye,
  Sparkles,
  Loader2,
  Settings,
  Zap,
} from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

// ============================================================================
// Types
// ============================================================================

interface AgentConfig {
  name: string
  displayName: string
  role: string
  icon: LucideIcon
  color: string
  enabled: boolean
  tools: string[]
}

interface PlatformConfig {
  agents: AgentConfig[]
  activeWorkflows: number
  cli: string
  backendStack: string
}

// ============================================================================
// Agent definitions (static display data)
// ============================================================================

const AGENT_DEFINITIONS: Omit<AgentConfig, 'enabled' | 'tools'>[] = [
  { name: 'morgan', displayName: 'Morgan', role: 'PM / Orchestrator', icon: Sparkles, color: 'text-purple-500' },
  { name: 'rex', displayName: 'Rex', role: 'Rust Engineer', icon: Code2, color: 'text-orange-500' },
  { name: 'blaze', displayName: 'Blaze', role: 'Frontend Engineer', icon: Zap, color: 'text-yellow-500' },
  { name: 'grizz', displayName: 'Grizz', role: 'Go Engineer', icon: Code2, color: 'text-cyan-500' },
  { name: 'nova', displayName: 'Nova', role: 'Node.js Engineer', icon: Code2, color: 'text-green-500' },
  { name: 'tess', displayName: 'Tess', role: 'Testing Specialist', icon: TestTube2, color: 'text-blue-500' },
  { name: 'cleo', displayName: 'Cleo', role: 'Code Quality', icon: Eye, color: 'text-indigo-500' },
  { name: 'cipher', displayName: 'Cipher', role: 'Security Analyst', icon: Shield, color: 'text-red-500' },
  { name: 'stitch', displayName: 'Stitch', role: 'Code Reviewer', icon: Eye, color: 'text-pink-500' },
  { name: 'bolt', displayName: 'Bolt', role: 'DevOps Engineer', icon: Rocket, color: 'text-amber-500' },
  { name: 'atlas', displayName: 'Atlas', role: 'Merge Gate', icon: Wrench, color: 'text-emerald-500' },
  { name: 'vex', displayName: 'Vex', role: 'AI/ML Engineer', icon: Bot, color: 'text-violet-500' },
  { name: 'spark', displayName: 'Spark', role: 'Infra Specialist', icon: Zap, color: 'text-rose-500' },
]

// ============================================================================
// Component
// ============================================================================

export function ConfigView() {
  const [config, setConfig] = useState<PlatformConfig | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadConfig()
  }, [])

  async function loadConfig() {
    setLoading(true)
    try {
      // In full implementation, this reads from the canonical cto-config.json
      // via OpenClaw. For now, we display default agent roster.
      const agents: AgentConfig[] = AGENT_DEFINITIONS.map((def) => ({
        ...def,
        enabled: true,
        tools: getDefaultTools(def.name),
      }))

      setConfig({
        agents,
        activeWorkflows: 0,
        cli: 'claude',
        backendStack: 'go',
      })
    } catch (error) {
      console.error('Failed to load config:', error)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!config) return null

  return (
    <ScrollArea className="h-full">
      <div className="max-w-4xl mx-auto p-6 space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Settings className="h-6 w-6" />
            Configuration
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Managed by Morgan. Ask in chat to make changes.
          </p>
        </div>

        {/* Platform summary */}
        <div className="grid grid-cols-3 gap-4">
          <Card>
            <CardContent className="pt-4">
              <div className="text-sm text-muted-foreground">CLI</div>
              <div className="font-semibold capitalize">{config.cli}</div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="text-sm text-muted-foreground">Backend Stack</div>
              <div className="font-semibold capitalize">{config.backendStack}</div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="pt-4">
              <div className="text-sm text-muted-foreground">Active Agents</div>
              <div className="font-semibold">
                {config.agents.filter((a) => a.enabled).length} /{' '}
                {config.agents.length}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Agent roster */}
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Agent Roster</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {config.agents.map((agent) => (
                <AgentCard key={agent.name} agent={agent} />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </ScrollArea>
  )
}

// ============================================================================
// Sub-components
// ============================================================================

function AgentCard({ agent }: { agent: AgentConfig }) {
  const Icon = agent.icon
  return (
    <div
      className={`flex items-start gap-3 p-3 rounded-lg border transition-colors ${
        agent.enabled
          ? 'bg-card hover:bg-accent'
          : 'bg-muted/50 opacity-60'
      }`}
    >
      <div
        className={`w-9 h-9 rounded-lg flex items-center justify-center bg-muted ${agent.color}`}
      >
        <Icon className="h-4 w-4" />
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm">{agent.displayName}</span>
          {agent.enabled ? (
            <Badge
              variant="outline"
              className="text-[10px] text-green-600 border-green-500/30"
            >
              Active
            </Badge>
          ) : (
            <Badge variant="outline" className="text-[10px]">
              Off
            </Badge>
          )}
        </div>
        <div className="text-xs text-muted-foreground">{agent.role}</div>
        {agent.tools.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1.5">
            {agent.tools.slice(0, 3).map((tool) => (
              <Badge key={tool} variant="secondary" className="text-[9px] px-1.5 py-0">
                {tool}
              </Badge>
            ))}
            {agent.tools.length > 3 && (
              <Badge variant="secondary" className="text-[9px] px-1.5 py-0">
                +{agent.tools.length - 3}
              </Badge>
            )}
          </div>
        )}
      </div>
    </div>
  )
}

// ============================================================================
// Helpers
// ============================================================================

function getDefaultTools(agentName: string): string[] {
  const toolMap: Record<string, string[]> = {
    morgan: ['openclaw', 'linear', 'github', 'lobster'],
    rex: ['claude', 'cargo', 'github'],
    blaze: ['claude', 'npm', 'github'],
    grizz: ['claude', 'go', 'github'],
    nova: ['claude', 'bun', 'github'],
    tess: ['claude', 'pytest', 'jest'],
    cleo: ['claude', 'clippy', 'eslint'],
    cipher: ['claude', 'semgrep', 'codeql'],
    stitch: ['claude', 'github'],
    bolt: ['kubectl', 'helm', 'argocd'],
    atlas: ['github', 'claude'],
    vex: ['claude', 'python', 'torch'],
    spark: ['terraform', 'kubectl', 'helm'],
  }
  return toolMap[agentName] ?? []
}
