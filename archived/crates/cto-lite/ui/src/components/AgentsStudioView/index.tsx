import { useMemo, useState } from 'react'
import { Bot, Check, FileJson, Settings2 } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import type { AgentUiConfig, ProjectRecord, StudioState } from '@/lib/tauri'
import * as tauri from '@/lib/tauri'
import { getAgentBranding } from '@/lib/agent-branding'

type AgentsStudioViewProps = {
  state: tauri.StudioState
  selectedProject: ProjectRecord
  onStateChange: (next: StudioState) => void
}

export function AgentsStudioView({
  state,
  selectedProject,
  onStateChange,
}: AgentsStudioViewProps) {
  const [selectedAgentId, setSelectedAgentId] = useState(state.agents[0]?.id ?? 'morgan')
  const [renderedConfig, setRenderedConfig] = useState<tauri.RenderedAgentConfig | null>(null)
  const [status, setStatus] = useState<string | null>(null)
  const [busyAction, setBusyAction] = useState<'render' | 'apply' | 'export' | null>(null)

  const selectedAgent = useMemo(
    () => state.agents.find((agent) => agent.id === selectedAgentId) ?? state.agents[0],
    [selectedAgentId, state.agents]
  )

  if (!selectedAgent) {
    return null
  }

  const selectedBranding = getAgentBranding(selectedAgent.id)

  const updateAgent = (patch: Partial<AgentUiConfig>) => {
    onStateChange({
      ...state,
      agents: state.agents.map((agent) =>
        agent.id === selectedAgent.id ? { ...agent, ...patch } : agent
      ),
    })
  }

  const runAction = async (action: 'render' | 'apply' | 'export') => {
    setBusyAction(action)
    setStatus(null)
    try {
      if (action === 'render') {
        const rendered = await tauri.studioRenderAgentConfig(selectedAgent.id, selectedProject.id)
        setRenderedConfig(rendered)
        setStatus('Rendered local runtime config.')
      } else if (action === 'export') {
        const rendered = await tauri.studioExportAgentConfig(selectedAgent.id, selectedProject.id)
        setRenderedConfig(rendered)
        setStatus(`Prepared export payload for ${selectedAgent.displayName}.`)
      } else {
        const result = await tauri.studioApplyAgentConfig(selectedAgent.id, selectedProject.id)
        const rendered = await tauri.studioRenderAgentConfig(selectedAgent.id, selectedProject.id)
        setRenderedConfig(rendered)
        setStatus(result.message)
      }
    } catch (error) {
      setStatus(`Agent config action failed: ${String(error)}`)
    } finally {
      setBusyAction(null)
    }
  }

  return (
    <div className="flex h-full flex-col overflow-hidden bg-[#090f1a] text-slate-100">
      <div className="border-b border-white/10 bg-[radial-gradient(circle_at_top,#12324f_0%,rgba(7,13,24,0.96)_52%,rgba(7,13,24,1)_100%)] px-6 py-4">
        <div className="flex items-center gap-3">
          <Badge variant="secondary" className="rounded-full">
            Agents
          </Badge>
          <Badge variant="outline" className="rounded-full border-white/12 bg-white/[0.04]">
            {selectedProject.name}
          </Badge>
        </div>
        <h2 className="mt-3 text-xl font-semibold tracking-tight text-white">Agent studio</h2>
        <p className="mt-1 text-sm text-slate-300">Configure runtime behavior and export deterministic config.</p>
      </div>

      <div className="grid min-h-0 min-w-0 flex-1 grid-cols-1 gap-4 p-4 xl:grid-cols-[280px_minmax(0,1fr)_minmax(320px,30vw)]">
        <ScrollArea className="rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-3">
          <div className="space-y-3">
            {state.agents.map((agent) => (
              <button
                key={agent.id}
                onClick={() => setSelectedAgentId(agent.id)}
                className={`w-full rounded-[22px] border p-4 text-left transition ${
                  selectedAgentId === agent.id
                    ? 'border-cyan-300/25 bg-cyan-400/10'
                    : 'border-white/10 bg-black/15 hover:bg-white/[0.04]'
                }`}
              >
                {(() => {
                  const branding = getAgentBranding(agent.id)
                  return (
                <div className="flex items-start gap-3">
                  <Avatar className="border border-white/10">
                    {branding.avatar ? <AvatarImage src={branding.avatar} alt={agent.displayName} /> : null}
                    <AvatarFallback className="bg-white/10 text-slate-100">
                      {agent.avatarLabel}
                    </AvatarFallback>
                  </Avatar>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <p className="font-medium text-white">{agent.displayName}</p>
                      <Badge variant="outline" className="rounded-full border-white/10">
                        {agent.enabled ? 'On' : 'Off'}
                      </Badge>
                    </div>
                    <p className="mt-1 text-xs text-slate-400">{agent.role}</p>
                    <p className="mt-2 text-sm leading-6 text-slate-300">{agent.summary}</p>
                  </div>
                </div>
                  )
                })()}
              </button>
            ))}
          </div>
        </ScrollArea>

        <ScrollArea className="rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-4">
          <div className="space-y-4">
            <Card className="rounded-[24px] border-white/10 bg-black/15">
              <CardHeader>
                <CardTitle className="text-white">Profile</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="rounded-[22px] border border-white/10 bg-white/[0.04] p-4">
                  <div className="flex items-center gap-4">
                    <Avatar className="h-16 w-16 border border-white/10 shadow-[0_18px_44px_-30px_rgba(34,211,238,0.65)]">
                      {selectedBranding.avatar ? (
                        <AvatarImage src={selectedBranding.avatar} alt={selectedAgent.displayName} />
                      ) : null}
                      <AvatarFallback className="bg-white/10 text-slate-100">
                        {selectedAgent.avatarLabel}
                      </AvatarFallback>
                    </Avatar>
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <p className="text-lg font-semibold text-white">{selectedAgent.displayName}</p>
                        <Badge variant="outline" className="rounded-full border-white/10">
                          {selectedAgent.enabled ? 'On' : 'Off'}
                        </Badge>
                      </div>
                      <p className="text-sm text-slate-400">{selectedAgent.role}</p>
                      <p className="mt-1 text-sm leading-6 text-slate-300">{selectedAgent.summary}</p>
                    </div>
                  </div>
                </div>

                <div className="grid gap-4 md:grid-cols-2">
                  <Field
                    label="Display name"
                    value={selectedAgent.displayName}
                    onChange={(value) => updateAgent({ displayName: value })}
                  />
                  <Field
                    label="Role"
                    value={selectedAgent.role}
                    onChange={(value) => updateAgent({ role: value })}
                  />
                  <Field
                    label="Heartbeat"
                    value={selectedAgent.heartbeatEvery}
                    onChange={(value) => updateAgent({ heartbeatEvery: value })}
                  />
                  <Field
                    label="Model"
                    value={selectedAgent.model}
                    onChange={(value) => updateAgent({ model: value })}
                  />
                </div>

                <Field
                  label="Summary"
                  value={selectedAgent.summary}
                  onChange={(value) => updateAgent({ summary: value })}
                />

                <TokenField
                  label="Skills"
                  values={selectedAgent.skills}
                  onChange={(values) => updateAgent({ skills: values })}
                />
                <TokenField
                  label="Capabilities"
                  values={selectedAgent.capabilities}
                  onChange={(values) => updateAgent({ capabilities: values })}
                />
                <TokenField
                  label="Tools"
                  values={selectedAgent.tools}
                  onChange={(values) => updateAgent({ tools: values })}
                />

                <div>
                  <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">
                    System prompt
                  </p>
                  <Textarea
                    value={selectedAgent.systemPrompt}
                    onChange={(event) => updateAgent({ systemPrompt: event.target.value })}
                    className="mt-2 min-h-[220px] border-white/10 bg-black/20 text-slate-100 placeholder:text-slate-500"
                  />
                </div>
              </CardContent>
            </Card>
          </div>
        </ScrollArea>

        <div className="min-h-0 overflow-hidden rounded-[28px] border border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] p-4">
          <Tabs defaultValue="runtime" className="flex h-full min-h-0 flex-col">
            <TabsList className="rounded-full border border-white/10 bg-white/[0.05]">
              <TabsTrigger value="runtime" className="rounded-full px-3 text-xs">
                Runtime
              </TabsTrigger>
              <TabsTrigger value="preview" className="rounded-full px-3 text-xs">
                Preview
              </TabsTrigger>
            </TabsList>

            <TabsContent value="runtime" className="mt-4 min-h-0 flex-1 overflow-auto">
                  <Card className="rounded-[24px] border-white/10 bg-black/15">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-white">
                    <Settings2 className="h-4 w-4 text-cyan-200" />
                    Runtime
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <p className="text-sm text-slate-300">Render, apply, or export the generated config.</p>
                  <div className="grid gap-3">
                    <Button onClick={() => void runAction('render')} disabled={busyAction !== null}>
                      <FileJson data-icon="inline-start" />
                      {busyAction === 'render' ? 'Rendering…' : 'Render config'}
                    </Button>
                    <Button
                      variant="secondary"
                      onClick={() => void runAction('apply')}
                      disabled={busyAction !== null}
                    >
                      <Check data-icon="inline-start" />
                      {busyAction === 'apply' ? 'Applying…' : 'Apply locally'}
                    </Button>
                    <Button
                      variant="outline"
                      onClick={() => void runAction('export')}
                      disabled={busyAction !== null}
                    >
                      <Bot data-icon="inline-start" />
                      {busyAction === 'export' ? 'Preparing…' : 'Export config'}
                    </Button>
                  </div>
                  {status ? (
                    <div className="rounded-[18px] border border-white/10 bg-black/20 px-3 py-2 text-sm text-slate-200">
                      {status}
                    </div>
                  ) : null}
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="preview" className="mt-4 min-h-0 flex-1 overflow-auto">
              <Card className="rounded-[24px] border-white/10 bg-black/15">
                <CardHeader>
                  <CardTitle className="text-white">Rendered preview</CardTitle>
                </CardHeader>
                <CardContent>
                  <pre className="overflow-auto rounded-[20px] border border-white/10 bg-black/20 p-4 text-xs leading-6 text-slate-200">
                    {renderedConfig?.content ??
                      'Render or apply this agent to inspect the generated runtime payload.'}
                  </pre>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  )
}

function Field({
  label,
  value,
  onChange,
}: {
  label: string
  value: string
  onChange: (value: string) => void
}) {
  return (
    <div>
      <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">{label}</p>
      <Input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="mt-2 border-white/10 bg-black/20 text-slate-100"
      />
    </div>
  )
}

function TokenField({
  label,
  values,
  onChange,
}: {
  label: string
  values: string[]
  onChange: (values: string[]) => void
}) {
  return (
    <div>
      <p className="text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">{label}</p>
      <Input
        value={values.join(', ')}
        onChange={(event) =>
          onChange(
            event.target.value
              .split(',')
              .map((value) => value.trim())
              .filter(Boolean)
          )
        }
        className="mt-2 border-white/10 bg-black/20 text-slate-100"
      />
    </div>
  )
}
