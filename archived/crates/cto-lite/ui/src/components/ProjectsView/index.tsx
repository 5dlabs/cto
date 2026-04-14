import { FolderKanban, GitBranch, Settings2, Sparkles } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { PrdMarkdownEditor } from '@/components/PrdMarkdownEditor'
import type { ProjectRecord, StudioState } from '@/lib/tauri'

type ProjectsViewProps = {
  state: StudioState
  onStateChange: (next: StudioState) => void
}

export function ProjectsView({ state, onStateChange }: ProjectsViewProps) {
  const selectedProject =
    state.projects.find((project) => project.id === state.selectedProjectId) ?? state.projects[0]

  if (!selectedProject) {
    return null
  }

  const updateProject = (patch: Partial<ProjectRecord>) => {
    onStateChange({
      ...state,
      projects: state.projects.map((project) =>
        project.id === selectedProject.id ? { ...project, ...patch } : project
      ),
    })
  }

  return (
    <div className="flex h-full flex-col overflow-hidden bg-[#090f1a] text-slate-100">
      <div className="border-b border-white/10 bg-[radial-gradient(circle_at_top,#12324f_0%,rgba(7,13,24,0.96)_52%,rgba(7,13,24,1)_100%)] px-6 py-4">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="rounded-full">
                Projects
              </Badge>
              <Badge variant="outline" className="rounded-full border-white/12 bg-white/[0.04]">
                Hub
              </Badge>
            </div>
            <h2 className="mt-3 text-xl font-semibold tracking-tight text-white">Project hub</h2>
            <p className="mt-1 text-sm text-slate-300">PRD, workflow, and config in one place.</p>
          </div>
          <div className="min-w-[240px]">
            <p className="mb-2 text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">
              Active project
            </p>
            <Select
              value={state.selectedProjectId}
              onValueChange={(value) => onStateChange({ ...state, selectedProjectId: value })}
            >
              <SelectTrigger className="border-white/10 bg-black/20 text-slate-100">
                <SelectValue placeholder="Choose a project" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  {state.projects.map((project) => (
                    <SelectItem key={project.id} value={project.id}>
                      {project.name}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      <ScrollArea className="h-full">
        <div className="grid min-w-0 gap-4 p-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(0,1fr)]">
          <Card className="rounded-[28px] border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322]">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-white">
                <FolderKanban className="h-4 w-4 text-cyan-200" />
                Overview
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <Field label="Project name" value={selectedProject.name} onChange={(value) => updateProject({ name: value })} />
              <Field label="Summary" value={selectedProject.summary} onChange={(value) => updateProject({ summary: value })} />
              <Field
                label="Repository"
                value={selectedProject.repository ?? ''}
                onChange={(value) => updateProject({ repository: value || null })}
              />
              <Field
                label="Config notes"
                value={selectedProject.configNotes}
                onChange={(value) => updateProject({ configNotes: value })}
                multiline
              />
            </CardContent>
          </Card>

          <Card className="rounded-[28px] border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322]">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-white">
                <GitBranch className="h-4 w-4 text-cyan-200" />
                Workflow
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <Field
                label="Workflow summary"
                value={selectedProject.workflowSummary}
                onChange={(value) => updateProject({ workflowSummary: value })}
              />
              <Field
                label="Workflow notes"
                value={selectedProject.workflowNotes}
                onChange={(value) => updateProject({ workflowNotes: value })}
                multiline
              />
            </CardContent>
          </Card>

          <Card className="rounded-[28px] border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] lg:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-white">
                <Sparkles className="h-4 w-4 text-cyan-200" />
                PRD
              </CardTitle>
            </CardHeader>
            <CardContent className="flex min-w-0 flex-col gap-6">
              <Field
                label="PRD title"
                value={selectedProject.prdTitle}
                onChange={(value) => updateProject({ prdTitle: value })}
              />
              <div>
                <p className="mb-3 text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">PRD content</p>
                <PrdMarkdownEditor
                  value={selectedProject.prdContent}
                  onChange={(value) => updateProject({ prdContent: value })}
                  minHeightPx={380}
                />
              </div>
            </CardContent>
          </Card>

          <Card className="rounded-[28px] border-white/10 bg-gradient-to-b from-[#111c2f] to-[#0b1322] lg:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-white">
                <Settings2 className="h-4 w-4 text-cyan-200" />
                Runtime view
              </CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3 md:grid-cols-3">
              <SummaryTile label="Session key" value={`morgan-${selectedProject.id}`} />
              <SummaryTile label="Voice room" value={`morgan-${selectedProject.id}`} />
              <SummaryTile label="Scope" value="Shared across chat, voice, video" />
            </CardContent>
          </Card>
        </div>
      </ScrollArea>
    </div>
  )
}

function Field({
  label,
  value,
  onChange,
  multiline = false,
  rows = 6,
}: {
  label: string
  value: string
  onChange: (value: string) => void
  multiline?: boolean
  rows?: number
}) {
  return (
    <div>
      <p className="mb-2 text-[11px] uppercase tracking-[0.28em] text-cyan-100/70">{label}</p>
      {multiline ? (
        <Textarea
          rows={rows}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          className="border-white/10 bg-black/20 text-slate-100"
        />
      ) : (
        <Input
          value={value}
          onChange={(event) => onChange(event.target.value)}
          className="border-white/10 bg-black/20 text-slate-100"
        />
      )}
    </div>
  )
}

function SummaryTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[22px] border border-white/10 bg-black/20 p-4">
      <p className="text-[11px] uppercase tracking-[0.28em] text-slate-500">{label}</p>
      <p className="mt-2 text-sm leading-6 text-slate-100">{value}</p>
    </div>
  )
}
