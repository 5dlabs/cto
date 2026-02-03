import { Dashboard } from '../components/Dashboard';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '../components/ui/card';
import { Server, Plus, GitBranch, FileJson, Play } from 'lucide-react';

interface HomeProps {
  onNavigate: (view: 'setup' | 'cluster' | 'settings' | 'home') => void;
}

export function Home({ onNavigate }: HomeProps) {
  const clusterStatus = 'running';

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Dashboard</h1>
          <p className="text-zinc-400 mt-1">
            Welcome to CTO App - Your unified development platform
          </p>
        </div>
        <div className="flex gap-3">
          <Button variant="outline" className="gap-2">
            <Play className="w-4 h-4" />
            Start Cluster
          </Button>
          <Button className="gap-2" onClick={() => onNavigate('setup')}>
            <Plus className="w-4 h-4" />
            New Workflow
          </Button>
        </div>
      </div>

      <Dashboard status={clusterStatus} />

      <div className="grid gap-4 md:grid-cols-3">
        <Card
          className="bg-zinc-900/50 border-zinc-800 cursor-pointer hover:border-zinc-700 transition-colors"
          onClick={() => onNavigate('cluster')}
        >
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Server className="w-5 h-5 text-emerald-500" />
              Clusters
            </CardTitle>
            <CardDescription>
              Manage your Kind clusters and namespaces
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold">1</span>
              <span className="text-zinc-400">active cluster</span>
            </div>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900/50 border-zinc-800 cursor-pointer hover:border-zinc-700 transition-colors">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <GitBranch className="w-5 h-5 text-purple-500" />
              Repositories
            </CardTitle>
            <CardDescription>
              Connected GitHub repositories
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold">12</span>
              <span className="text-zinc-400">repositories</span>
            </div>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900/50 border-zinc-800 cursor-pointer hover:border-zinc-700 transition-colors">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <FileJson className="w-5 h-5 text-cyan-500" />
              Workflows
            </CardTitle>
            <CardDescription>
              PRD and development workflows
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold">5</span>
              <span className="text-zinc-400">active workflows</span>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card className="bg-zinc-900/50 border-zinc-800">
        <CardHeader>
          <CardTitle>Recent Activity</CardTitle>
          <CardDescription>Your latest workflow executions</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[
              { name: 'Build & Deploy API', status: 'completed', time: '2 min ago' },
              { name: 'Run Tests', status: 'completed', time: '15 min ago' },
              { name: 'Lint & Format', status: 'failed', time: '1 hour ago' },
              { name: 'Generate Docs', status: 'running', time: '2 hours ago' },
            ].map((item) => (
              <div
                key={item.name}
                className="flex items-center justify-between py-2 border-b border-zinc-800 last:border-0"
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`w-2 h-2 rounded-full ${
                      item.status === 'completed'
                        ? 'bg-emerald-500'
                        : item.status === 'failed'
                        ? 'bg-red-500'
                        : 'bg-amber-500 animate-pulse'
                    }`}
                  />
                  <span>{item.name}</span>
                </div>
                <span className="text-zinc-500 text-sm">{item.time}</span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
