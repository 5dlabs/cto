import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useToast } from '@/hooks/use-toast'
import {
  Activity,
  CheckCircle2,
  Circle,
  Clock,
  GitBranch,
  Play,
  RefreshCw,
  Server,
  Settings,
  XCircle,
  Loader2,
} from 'lucide-react'

interface WorkflowInfo {
  name: string
  namespace: string
  phase: string
  started_at: string | null
  finished_at: string | null
  message: string | null
}

interface ClusterStatus {
  name: string
  exists: boolean
  running: boolean
  nodes: Array<{ name: string; role: string; status: string }>
  kubeconfig_path: string | null
}

interface TunnelStatus {
  exists: boolean
  running: boolean
  tunnel_id: string | null
  url: string | null
}

export function Dashboard() {
  const [workflows, setWorkflows] = useState<WorkflowInfo[]>([])
  const [clusterStatus, setClusterStatus] = useState<ClusterStatus | null>(null)
  const [tunnelStatus, setTunnelStatus] = useState<TunnelStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const { toast } = useToast()

  useEffect(() => {
    loadStatus()
    const interval = setInterval(loadStatus, 30000) // Refresh every 30s
    return () => clearInterval(interval)
  }, [])

  async function loadStatus() {
    try {
      const [cluster, tunnel, workflowList] = await Promise.all([
        invoke<ClusterStatus>('get_cluster_status'),
        invoke<TunnelStatus>('get_tunnel_status'),
        invoke<WorkflowInfo[]>('list_workflows').catch(() => []),
      ])
      setClusterStatus(cluster)
      setTunnelStatus(tunnel)
      setWorkflows(workflowList)
    } catch (error) {
      console.error('Failed to load status:', error)
    } finally {
      setLoading(false)
    }
  }

  async function handleRefresh() {
    setRefreshing(true)
    await loadStatus()
    setRefreshing(false)
    toast({ title: 'Status Refreshed' })
  }

  async function handleStartTunnel() {
    try {
      await invoke('start_tunnel')
      await loadStatus()
      toast({ title: 'Tunnel Started' })
    } catch (error) {
      toast({ title: 'Failed to start tunnel', description: String(error), variant: 'destructive' })
    }
  }

  async function handleStopTunnel() {
    try {
      await invoke('stop_tunnel')
      await loadStatus()
      toast({ title: 'Tunnel Stopped' })
    } catch (error) {
      toast({ title: 'Failed to stop tunnel', description: String(error), variant: 'destructive' })
    }
  }

  function getPhaseIcon(phase: string) {
    switch (phase) {
      case 'Succeeded':
        return <CheckCircle2 className="h-4 w-4 text-green-500" />
      case 'Failed':
      case 'Error':
        return <XCircle className="h-4 w-4 text-red-500" />
      case 'Running':
        return <Loader2 className="h-4 w-4 text-blue-500 animate-spin" />
      case 'Pending':
        return <Clock className="h-4 w-4 text-yellow-500" />
      default:
        return <Circle className="h-4 w-4 text-muted-foreground" />
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
          <p className="text-sm text-muted-foreground">Loading dashboard...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen p-8">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold">CTO Lite Dashboard</h1>
          <p className="text-muted-foreground">Monitor your AI development workflows</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleRefresh} disabled={refreshing}>
            <RefreshCw className={`mr-2 h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button variant="outline" size="sm">
            <Settings className="mr-2 h-4 w-4" />
            Settings
          </Button>
        </div>
      </div>

      {/* Status Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        {/* Cluster Status */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Server className="h-4 w-4" />
              Cluster
            </CardTitle>
          </CardHeader>
          <CardContent>
            {clusterStatus?.running ? (
              <div className="flex items-center gap-2">
                <div className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                <span className="text-green-500 font-medium">Running</span>
              </div>
            ) : clusterStatus?.exists ? (
              <div className="flex items-center gap-2">
                <div className="h-2 w-2 rounded-full bg-yellow-500" />
                <span className="text-yellow-500 font-medium">Stopped</span>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <div className="h-2 w-2 rounded-full bg-red-500" />
                <span className="text-red-500 font-medium">Not Created</span>
              </div>
            )}
            {clusterStatus?.nodes && clusterStatus.nodes.length > 0 && (
              <p className="text-xs text-muted-foreground mt-1">
                {clusterStatus.nodes.length} node(s)
              </p>
            )}
          </CardContent>
        </Card>

        {/* Tunnel Status */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Activity className="h-4 w-4" />
              Tunnel
            </CardTitle>
          </CardHeader>
          <CardContent>
            {tunnelStatus?.running ? (
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <div className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                  <span className="text-green-500 font-medium">Connected</span>
                </div>
                {tunnelStatus.url && (
                  <p className="text-xs text-muted-foreground truncate">
                    {tunnelStatus.url}
                  </p>
                )}
                <Button variant="outline" size="sm" onClick={handleStopTunnel}>
                  Stop Tunnel
                </Button>
              </div>
            ) : tunnelStatus?.exists ? (
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <div className="h-2 w-2 rounded-full bg-yellow-500" />
                  <span className="text-yellow-500 font-medium">Disconnected</span>
                </div>
                <Button variant="outline" size="sm" onClick={handleStartTunnel}>
                  Start Tunnel
                </Button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <div className="h-2 w-2 rounded-full bg-muted-foreground" />
                <span className="text-muted-foreground">Not Configured</span>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Workflows Count */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <GitBranch className="h-4 w-4" />
              Workflows
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{workflows.length}</div>
            <p className="text-xs text-muted-foreground">
              {workflows.filter(w => w.phase === 'Running').length} running
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Workflows List */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Recent Workflows
          </CardTitle>
          <CardDescription>
            Your development workflows and their status
          </CardDescription>
        </CardHeader>
        <CardContent>
          {workflows.length === 0 ? (
            <div className="text-center py-12">
              <Play className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No workflows yet</h3>
              <p className="text-sm text-muted-foreground mb-4">
                Start a workflow from your IDE using MCP tools
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {workflows.map((workflow) => (
                <div
                  key={workflow.name}
                  className="flex items-center justify-between p-4 rounded-lg border"
                >
                  <div className="flex items-center gap-4">
                    {getPhaseIcon(workflow.phase)}
                    <div>
                      <div className="font-medium">{workflow.name}</div>
                      <div className="text-sm text-muted-foreground">
                        {workflow.started_at
                          ? new Date(workflow.started_at).toLocaleString()
                          : 'Not started'}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span
                      className={`px-2 py-1 rounded-full text-xs font-medium ${
                        workflow.phase === 'Succeeded'
                          ? 'bg-green-500/10 text-green-500'
                          : workflow.phase === 'Failed' || workflow.phase === 'Error'
                          ? 'bg-red-500/10 text-red-500'
                          : workflow.phase === 'Running'
                          ? 'bg-blue-500/10 text-blue-500'
                          : 'bg-muted text-muted-foreground'
                      }`}
                    >
                      {workflow.phase}
                    </span>
                    <Button variant="ghost" size="sm">
                      View Logs
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
