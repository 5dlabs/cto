import { useState } from 'react';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from './ui/card';
import { Badge } from './ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from './ui/table';
import { Button } from './ui/button';
import {
  Server,
  GitBranch,
  Activity,
  Terminal,
  Cpu,
  Zap,
  Shield,
  Users,
  Brain,
  Flame,
  Key,
  Eye,
  Zap as ZapIcon,
  RefreshCw,
} from 'lucide-react';

interface DashboardProps {
  status: 'running' | 'stopped' | 'starting';
}

// Mock data for agent pods
const agentPods = [
  { name: 'Morgan', role: 'Lead Agent', status: 'running', icon: Brain, tier: 'pro' },
  { name: 'Grizz/Nova', role: 'Security/Research', status: 'running', icon: Shield, tier: 'pro' },
  { name: 'Blaze', role: 'DevOps Engineer', status: 'running', icon: Flame, tier: 'pro' },
  { name: 'Cleo', role: 'Product Manager', status: 'stopped', icon: Users, tier: 'enterprise' },
  { name: 'Cipher', role: 'Security Analyst', status: 'running', icon: Key, tier: 'enterprise' },
  { name: 'Tess', role: 'QA Engineer', status: 'running', icon: Eye, tier: 'pro' },
  { name: 'Bolt', role: 'Infrastructure', status: 'stopped', icon: ZapIcon, tier: 'enterprise' },
];

// Mock data for recent workflow runs
const workflowRuns = [
  { id: 'wf-001', name: 'Build & Deploy API', agent: 'Blaze', status: 'completed', duration: '2m 34s', time: '2 min ago' },
  { id: 'wf-002', name: 'Security Audit', agent: 'Cipher', status: 'completed', duration: '5m 12s', time: '15 min ago' },
  { id: 'wf-003', name: 'PRD Generation', agent: 'Morgan', status: 'running', duration: '-', time: '1 hour ago' },
  { id: 'wf-004', name: 'Test Suite', agent: 'Tess', status: 'failed', duration: '0m 45s', time: '2 hours ago' },
  { id: 'wf-005', name: 'Code Review', agent: 'Grizz', status: 'completed', duration: '3m 21s', time: '3 hours ago' },
];

// Mock logs for the log viewer
const mockLogs = [
  { timestamp: '14:32:15', level: 'INFO', message: 'Cluster cto-local is ready' },
  { timestamp: '14:32:14', level: 'INFO', message: 'Pod morgan-7b5d9f8c5-xk2mz ready' },
  { timestamp: '14:32:12', level: 'INFO', message: 'Pod blaze-7b5d9f8c5-ml4np ready' },
  { timestamp: '14:32:10', level: 'INFO', message: 'Starting agent pods...' },
  { timestamp: '14:32:08', level: 'INFO', message: 'Kind cluster cto-local started' },
];

export function Dashboard({ status }: DashboardProps) {
  const [activeTab, setActiveTab] = useState('agents');
  const [isStreaming, setIsStreaming] = useState(false);

  const stats: Array<{
    title: string;
    value: string | number;
    icon: React.ComponentType<{ className?: string }>;
    color: string;
    badgeVariant?: 'default' | 'secondary' | 'destructive' | 'outline' | 'running' | 'stopped';
  }> = [
    {
      title: 'Cluster Status',
      value: status.charAt(0).toUpperCase() + status.slice(1),
      icon: Server,
      color: status === 'running' ? 'text-emerald-500' : 'text-amber-500',
      badgeVariant: status === 'running' ? 'default' : 'stopped',
    },
    {
      title: 'Active Agents',
      value: agentPods.filter((p) => p.status === 'running').length,
      icon: Activity,
      color: 'text-blue-500',
    },
    {
      title: 'Running Workflows',
      value: workflowRuns.filter((w) => w.status === 'running').length,
      icon: Zap,
      color: 'text-amber-500',
    },
    {
      title: 'Completed Today',
      value: workflowRuns.filter((w) => w.status === 'completed').length,
      icon: GitBranch,
      color: 'text-emerald-500',
    },
  ];

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'running':
        return <Badge variant="running">Running</Badge>;
      case 'completed':
        return <Badge variant="default">Completed</Badge>;
      case 'failed':
        return <Badge variant="destructive">Failed</Badge>;
      case 'stopped':
        return <Badge variant="stopped">Stopped</Badge>;
      default:
        return <Badge variant="secondary">{status}</Badge>;
    }
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'INFO':
        return 'text-blue-400';
      case 'WARN':
        return 'text-amber-400';
      case 'ERROR':
        return 'text-red-400';
      case 'DEBUG':
        return 'text-zinc-400';
      default:
        return 'text-zinc-300';
    }
  };

  return (
    <div className="space-y-6">
      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.title} className="bg-zinc-900/50 border-zinc-800">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-zinc-400">
                {stat.title}
              </CardTitle>
              <stat.icon className={`h-4 w-4 ${stat.color}`} />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold flex items-center gap-2">
                {stat.value}
                {stat.badgeVariant && (
                  <Badge variant={stat.badgeVariant} className="text-xs">
                    {stat.value === 'Running' ? '●' : '○'}
                  </Badge>
                )}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-3 bg-zinc-800">
          <TabsTrigger value="agents" className="gap-2">
            <Cpu className="w-4 h-4" />
            Agent Pods
          </TabsTrigger>
          <TabsTrigger value="workflows" className="gap-2">
            <GitBranch className="w-4 h-4" />
            Workflows
          </TabsTrigger>
          <TabsTrigger value="logs" className="gap-2">
            <Terminal className="w-4 h-4" />
            Log Viewer
          </TabsTrigger>
        </TabsList>

        {/* Agent Pods Tab */}
        <TabsContent value="agents" className="mt-4">
          <Card className="bg-zinc-900/50 border-zinc-800">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Cpu className="w-5 h-5 text-emerald-500" />
                Agent Pods Status
              </CardTitle>
              <CardDescription>
                Monitor the status of all CTO agent pods in your cluster
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow className="border-zinc-800 hover:bg-zinc-800/50">
                    <TableHead className="text-zinc-400">Agent</TableHead>
                    <TableHead className="text-zinc-400">Role</TableHead>
                    <TableHead className="text-zinc-400">Tier</TableHead>
                    <TableHead className="text-zinc-400">Status</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {agentPods.map((pod) => (
                    <TableRow key={pod.name} className="border-zinc-800 hover:bg-zinc-800/50">
                      <TableCell className="flex items-center gap-3">
                        <pod.icon className="w-4 h-4 text-zinc-400" />
                        <span className="font-medium">{pod.name}</span>
                      </TableCell>
                      <TableCell className="text-zinc-300">{pod.role}</TableCell>
                      <TableCell>
                        <Badge variant={pod.tier === 'enterprise' ? 'default' : 'secondary'} className="capitalize">
                          {pod.tier}
                        </Badge>
                      </TableCell>
                      <TableCell>{getStatusBadge(pod.status)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Workflows Tab */}
        <TabsContent value="workflows" className="mt-4">
          <Card className="bg-zinc-900/50 border-zinc-800">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <GitBranch className="w-5 h-5 text-purple-500" />
                    Recent Workflow Runs
                  </CardTitle>
                  <CardDescription>
                    Track your recent workflow executions and their status
                  </CardDescription>
                </div>
                <Button variant="outline" size="sm" className="gap-2">
                  <RefreshCw className="w-4 h-4" />
                  Refresh
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow className="border-zinc-800 hover:bg-zinc-800/50">
                    <TableHead className="text-zinc-400">Workflow</TableHead>
                    <TableHead className="text-zinc-400">Agent</TableHead>
                    <TableHead className="text-zinc-400">Duration</TableHead>
                    <TableHead className="text-zinc-400">Status</TableHead>
                    <TableHead className="text-zinc-400">Time</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {workflowRuns.map((run) => (
                    <TableRow key={run.id} className="border-zinc-800 hover:bg-zinc-800/50">
                      <TableCell className="font-medium">{run.name}</TableCell>
                      <TableCell className="text-zinc-300">{run.agent}</TableCell>
                      <TableCell className="text-zinc-400">{run.duration}</TableCell>
                      <TableCell>{getStatusBadge(run.status)}</TableCell>
                      <TableCell className="text-zinc-500">{run.time}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Log Viewer Tab */}
        <TabsContent value="logs" className="mt-4">
          <Card className="bg-zinc-900/50 border-zinc-800">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <Terminal className="w-5 h-5 text-cyan-500" />
                    Cluster Logs
                  </CardTitle>
                  <CardDescription>
                    View real-time logs from your Kind cluster (streaming disabled)
                  </CardDescription>
                </div>
                <Button
                  variant={isStreaming ? 'destructive' : 'outline'}
                  size="sm"
                  className="gap-2"
                  onClick={() => setIsStreaming(!isStreaming)}
                >
                  <Activity className="w-4 h-4" />
                  {isStreaming ? 'Stop Stream' : 'Start Stream'}
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="bg-zinc-950 rounded-lg p-4 font-mono text-sm h-80 overflow-y-auto border border-zinc-800">
                <div className="space-y-1">
                  {mockLogs.map((log, index) => (
                    <div key={index} className="flex gap-3">
                      <span className="text-zinc-500 shrink-0">{log.timestamp}</span>
                      <span className={getLogLevelColor(log.level)}>[{log.level}]</span>
                      <span className="text-zinc-300">{log.message}</span>
                    </div>
                  ))}
                  {isStreaming && (
                    <div className="flex items-center gap-2 text-amber-400 mt-4">
                      <span className="animate-pulse">●</span>
                      <span>Streaming logs from cluster...</span>
                    </div>
                  )}
                  {!isStreaming && (
                    <div className="flex items-center justify-center h-32 text-zinc-500">
                      <div className="text-center">
                        <Terminal className="w-8 h-8 mx-auto mb-2 opacity-50" />
                        <p>Log streaming is disabled</p>
                        <p className="text-sm">Click "Start Stream" to begin receiving logs</p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
