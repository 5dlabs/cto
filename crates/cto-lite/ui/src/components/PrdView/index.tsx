import { useState, useCallback } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  FileText,
  Upload,
  Link,
  Hash,
  Plus,
  Loader2,
} from 'lucide-react'
import { LaunchButton } from '@/components/LaunchButton'
import { LifecycleProgress, type LifecycleStage } from '@/components/LifecycleProgress'
import * as tauri from '@/lib/tauri'

// ============================================================================
// Types
// ============================================================================

interface PrdSource {
  type: 'paste' | 'linear' | 'file' | 'url'
  label: string
  icon: typeof FileText
}

interface PrdDocument {
  title: string
  content: string
  source: PrdSource['type']
  sourceRef?: string // Linear issue ID, file path, or URL
}

// ============================================================================
// Component
// ============================================================================

const PRD_SOURCES: PrdSource[] = [
  { type: 'paste', label: 'Paste', icon: FileText },
  { type: 'linear', label: 'Linear', icon: Hash },
  { type: 'file', label: 'File', icon: Upload },
  { type: 'url', label: 'URL', icon: Link },
]

export function PrdView() {
  const [activeSource, setActiveSource] = useState<PrdSource['type']>('paste')
  const [prd, setPrd] = useState<PrdDocument>({
    title: '',
    content: '',
    source: 'paste',
  })
  const [linearIssueId, setLinearIssueId] = useState('')
  const [url, setUrl] = useState('')
  const [fetchingPrd, setFetchingPrd] = useState(false)

  // Lifecycle tracking
  const [activeWorkflowId, setActiveWorkflowId] = useState<string | null>(null)
  const [lifecycleStages] = useState<LifecycleStage[]>([
    { id: 'intake', name: 'Intake', status: 'pending' },
    { id: 'implementation', name: 'Implementation', status: 'pending' },
    { id: 'testing', name: 'Testing', status: 'pending' },
    { id: 'deploy', name: 'Deploy', status: 'pending' },
  ])

  const fetchFromLinear = useCallback(async () => {
    if (!linearIssueId.trim()) return
    setFetchingPrd(true)
    try {
      const result = await tauri.openclawSendMessage(
        'prd-fetch',
        `Fetch PRD from Linear issue ${linearIssueId}`
      )
      setPrd({
        title: `Linear: ${linearIssueId}`,
        content: result.content,
        source: 'linear',
        sourceRef: linearIssueId,
      })
    } catch (error) {
      console.error('Failed to fetch from Linear:', error)
    } finally {
      setFetchingPrd(false)
    }
  }, [linearIssueId])

  const fetchFromUrl = useCallback(async () => {
    if (!url.trim()) return
    setFetchingPrd(true)
    try {
      const result = await tauri.openclawSendMessage(
        'prd-fetch',
        `Fetch PRD from URL: ${url}`
      )
      setPrd({
        title: `URL Import`,
        content: result.content,
        source: 'url',
        sourceRef: url,
      })
    } catch (error) {
      console.error('Failed to fetch from URL:', error)
    } finally {
      setFetchingPrd(false)
    }
  }, [url])

  const handleFileSelect = useCallback(async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const filePath = await open({
        filters: [
          { name: 'Documents', extensions: ['md', 'txt', 'markdown'] },
        ],
      })
      if (filePath) {
        const result = await tauri.openclawSendMessage(
          'prd-fetch',
          `Read PRD file at: ${String(filePath)}`
        )
        setPrd({
          title: String(filePath).split('/').pop() ?? 'Imported PRD',
          content: result.content,
          source: 'file',
          sourceRef: String(filePath),
        })
      }
    } catch (error) {
      console.error('Failed to read file:', error)
    }
  }, [])

  const handleLaunch = (workflowId: string) => {
    setActiveWorkflowId(workflowId)
  }

  const hasPrdContent = prd.content.trim().length > 0

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-hidden">
        <ScrollArea className="h-full">
          <div className="max-w-4xl mx-auto p-6 space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold">Product Requirements</h1>
                <p className="text-muted-foreground text-sm mt-1">
                  Load a PRD and launch the development lifecycle
                </p>
              </div>
              {hasPrdContent && (
                <LaunchButton
                  prdContent={prd.content}
                  prdTitle={prd.title}
                  onLaunch={handleLaunch}
                  onError={(err) => console.error('Launch failed:', err)}
                />
              )}
            </div>

            {/* Source tabs */}
            <div className="flex gap-2">
              {PRD_SOURCES.map((source) => {
                const Icon = source.icon
                const isActive = activeSource === source.type
                return (
                  <Button
                    key={source.type}
                    variant={isActive ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => setActiveSource(source.type)}
                    className="gap-1.5"
                  >
                    <Icon className="h-3.5 w-3.5" />
                    {source.label}
                  </Button>
                )
              })}
            </div>

            {/* Source-specific input */}
            <Card>
              <CardContent className="pt-6">
                {activeSource === 'paste' && (
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="prd-title">Title</Label>
                      <Input
                        id="prd-title"
                        placeholder="PRD title..."
                        value={prd.title}
                        onChange={(e) =>
                          setPrd((prev) => ({ ...prev, title: e.target.value }))
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="prd-content">Content</Label>
                      <Textarea
                        id="prd-content"
                        placeholder="Paste your PRD content here (Markdown supported)..."
                        rows={16}
                        className="font-mono text-sm"
                        value={prd.content}
                        onChange={(e) =>
                          setPrd((prev) => ({
                            ...prev,
                            content: e.target.value,
                            source: 'paste',
                          }))
                        }
                      />
                    </div>
                  </div>
                )}

                {activeSource === 'linear' && (
                  <div className="space-y-4">
                    <p className="text-sm text-muted-foreground">
                      Enter a Linear issue identifier to fetch the PRD content.
                    </p>
                    <div className="flex gap-2">
                      <Input
                        placeholder="e.g. ENG-123 or issue URL"
                        value={linearIssueId}
                        onChange={(e) => setLinearIssueId(e.target.value)}
                      />
                      <Button
                        onClick={fetchFromLinear}
                        disabled={fetchingPrd || !linearIssueId.trim()}
                      >
                        {fetchingPrd ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          'Fetch'
                        )}
                      </Button>
                    </div>
                  </div>
                )}

                {activeSource === 'file' && (
                  <div className="space-y-4">
                    <p className="text-sm text-muted-foreground">
                      Select a Markdown file from your filesystem.
                    </p>
                    <Button onClick={handleFileSelect} variant="outline" className="gap-2">
                      <Upload className="h-4 w-4" />
                      Choose File
                    </Button>
                  </div>
                )}

                {activeSource === 'url' && (
                  <div className="space-y-4">
                    <p className="text-sm text-muted-foreground">
                      Provide a URL to a PRD document.
                    </p>
                    <div className="flex gap-2">
                      <Input
                        placeholder="https://..."
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                      />
                      <Button
                        onClick={fetchFromUrl}
                        disabled={fetchingPrd || !url.trim()}
                      >
                        {fetchingPrd ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          'Fetch'
                        )}
                      </Button>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* PRD Preview (if loaded from non-paste source) */}
            {hasPrdContent && activeSource !== 'paste' && (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <FileText className="h-4 w-4" />
                    {prd.title || 'Loaded PRD'}
                    <Badge variant="outline" className="text-xs ml-auto">
                      {prd.source}
                    </Badge>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <pre className="text-sm whitespace-pre-wrap font-mono bg-muted p-4 rounded-lg max-h-[400px] overflow-auto">
                    {prd.content}
                  </pre>
                </CardContent>
              </Card>
            )}

            {/* Lifecycle Progress (shown after launch) */}
            {activeWorkflowId && (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Plus className="h-4 w-4" />
                    Lifecycle Progress
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <LifecycleProgress
                    stages={lifecycleStages}
                    workflowId={activeWorkflowId}
                  />
                </CardContent>
              </Card>
            )}
          </div>
        </ScrollArea>
      </div>
    </div>
  )
}
