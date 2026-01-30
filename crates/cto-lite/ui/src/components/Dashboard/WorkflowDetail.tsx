import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useToast } from '@/hooks/use-toast'
import {
  ArrowLeft,
  CheckCircle2,
  Circle,
  Clock,
  Loader2,
  RefreshCw,
  XCircle,
  Terminal,
  StopCircle,
} from 'lucide-react'

interface WorkflowInfo {
  name: string
  namespace: string
  phase: string
  started_at: string | null
  finished_at: string | null
  message: string | null
}

interface WorkflowDetailProps {
  workflowName: string
  onBack: () => void
}

export function WorkflowDetail({ workflowName, onBack }: WorkflowDetailProps) {
  const [workflow, setWorkflow] = useState<WorkflowInfo | null>(null)
  const [logs, setLogs] = useState<string>('')
  const [loading, setLoading] = useState(true)
  const [autoRefresh, setAutoRefresh] = useState(true)
  const logsEndRef = useRef<HTMLDivElement>(null)
  const { toast } = useToast()

  useEffect(() => {
    loadWorkflow()
    loadLogs()

    if (autoRefresh) {
      const interval = setInterval(() => {
        loadWorkflow()
        loadLogs()
      }, 5000) // Refresh every 5s
      return () => clearInterval(interval)
    }
  }, [workflowName, autoRefresh])

  useEffect(() => {
    // Auto-scroll to bottom when logs update
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  async function loadWorkflow() {
    try {
      const status = await invoke<WorkflowInfo>('get_workflow_status', { name: workflowName })
      setWorkflow(status)
      
      // Stop auto-refresh if workflow is complete
      if (status.phase === 'Succeeded' || status.phase === 'Failed' || status.phase === 'Error') {
        setAutoRefresh(false)
      }
    } catch (error) {
      console.error('Failed to load workflow:', error)
    } finally {
      setLoading(false)
    }
  }

  async function loadLogs() {
    try {
      const logText = await invoke<string>('get_workflow_logs', { 
        name: workflowName,
        tail: 500 
      })
      setLogs(logText)
    } catch (error) {
      console.error('Failed to load logs:', error)
    }
  }

  async function handleCancel() {
    try {
      await invoke('cancel_workflow', { name: workflowName })
      toast({ title: 'Workflow cancelled' })
      await loadWorkflow()
    } catch (error) {
      toast({ title: 'Failed to cancel workflow', description: String(error), variant: 'destructive' })
    }
  }

  function getPhaseIcon(phase: string) {
    switch (phase) {
      case 'Succeeded':
        return <CheckCircle2 className="h-5 w-5 text-green-500" />
      case 'Failed':
      case 'Error':
        return <XCircle className="h-5 w-5 text-red-500" />
      case 'Running':
        return <Loader2 className="h-5 w-5 text-blue-500 animate-spin" />
      case 'Pending':
        return <Clock className="h-5 w-5 text-yellow-500" />
      default:
        return <Circle className="h-5 w-5 text-muted-foreground" />
    }
  }

  function getPhaseColor(phase: string) {
    switch (phase) {
      case 'Succeeded':
        return 'bg-green-500/10 text-green-500 border-green-500/20'
      case 'Failed':
      case 'Error':
        return 'bg-red-500/10 text-red-500 border-red-500/20'
      case 'Running':
        return 'bg-blue-500/10 text-blue-500 border-blue-500/20'
      case 'Pending':
        return 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20'
      default:
        return 'bg-muted text-muted-foreground'
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="sm" onClick={onBack}>
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back
          </Button>
          <div>
            <h2 className="text-xl font-semibold flex items-center gap-2">
              {workflow && getPhaseIcon(workflow.phase)}
              {workflowName}
            </h2>
            {workflow?.started_at && (
              <p className="text-sm text-muted-foreground">
                Started {new Date(workflow.started_at).toLocaleString()}
              </p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          {workflow?.phase === 'Running' && (
            <Button variant="destructive" size="sm" onClick={handleCancel}>
              <StopCircle className="mr-2 h-4 w-4" />
              Cancel
            </Button>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={() => { loadWorkflow(); loadLogs() }}
          >
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      {/* Status Card */}
      {workflow && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">Status</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-4">
              <span className={`px-3 py-1 rounded-full text-sm font-medium border ${getPhaseColor(workflow.phase)}`}>
                {workflow.phase}
              </span>
              {workflow.finished_at && (
                <span className="text-sm text-muted-foreground">
                  Finished {new Date(workflow.finished_at).toLocaleString()}
                </span>
              )}
            </div>
            {workflow.message && (
              <p className="mt-2 text-sm text-muted-foreground">
                {workflow.message}
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {/* Logs Card */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Terminal className="h-4 w-4" />
              Logs
            </CardTitle>
            <div className="flex items-center gap-2">
              {autoRefresh && (
                <span className="text-xs text-muted-foreground flex items-center gap-1">
                  <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                  Auto-refreshing
                </span>
              )}
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="relative">
            <pre className="bg-muted/50 rounded-lg p-4 text-xs font-mono overflow-auto max-h-[500px] whitespace-pre-wrap">
              {logs || 'No logs available yet...'}
              <div ref={logsEndRef} />
            </pre>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
