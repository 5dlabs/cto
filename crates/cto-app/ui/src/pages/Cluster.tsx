import { useState } from 'react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Server, Play, Square, RefreshCw, Plus, Terminal } from 'lucide-react';

interface ClusterProps {
  onBack: () => void;
}

export function Cluster({ onBack }: ClusterProps) {
  const [clusters, setClusters] = useState([
    { name: 'cto-local', status: 'running', nodes: 1, kubeconfig: '~/.kube/config' },
  ]);

  const [showCreateModal, setShowCreateModal] = useState(false);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" onClick={onBack}>
            Back
          </Button>
          <div>
            <h1 className="text-2xl font-semibold">Clusters</h1>
            <p className="text-zinc-400 mt-1">
              Manage your local Kind clusters
            </p>
          </div>
        </div>
        <Button className="gap-2" onClick={() => setShowCreateModal(true)}>
          <Plus className="w-4 h-4" />
          Create Cluster
        </Button>
      </div>

      <div className="grid gap-4">
        {clusters.map((cluster) => (
          <Card key={cluster.name} className="bg-zinc-900/50 border-zinc-800">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2">
                  <Server className="w-5 h-5 text-emerald-500" />
                  {cluster.name}
                </CardTitle>
                <div className="flex items-center gap-2">
                  <span
                    className={`px-2 py-1 rounded-full text-xs font-medium ${
                      cluster.status === 'running'
                        ? 'bg-emerald-500/20 text-emerald-500'
                        : 'bg-zinc-700 text-zinc-300'
                    }`}
                  >
                    {cluster.status}
                  </span>
                </div>
              </div>
              <CardDescription>
                {cluster.nodes} node(s) • {cluster.kubeconfig}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex items-center gap-3">
                {cluster.status === 'running' ? (
                  <Button variant="outline" size="sm" className="gap-2">
                    <Square className="w-4 h-4" />
                    Stop
                  </Button>
                ) : (
                  <Button variant="outline" size="sm" className="gap-2">
                    <Play className="w-4 h-4" />
                    Start
                  </Button>
                )}
                <Button variant="outline" size="sm" className="gap-2">
                  <RefreshCw className="w-4 h-4" />
                  Restart
                </Button>
                <Button variant="outline" size="sm" className="gap-2">
                  <Terminal className="w-4 h-4" />
                  Shell
                </Button>
                <Button variant="destructive" size="sm" className="ml-auto">
                  Delete
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {showCreateModal && (
        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader>
            <CardTitle>Create New Cluster</CardTitle>
            <CardDescription>
              Configure your new Kind cluster
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="cluster-name">Cluster Name</Label>
              <Input id="cluster-name" placeholder="my-cluster" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="node-count">Node Count</Label>
              <Input id="node-count" type="number" min="1" max="10" defaultValue="1" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="k8s-version">Kubernetes Version</Label>
              <Input id="k8s-version" placeholder="1.28.0" defaultValue="1.28.0" />
            </div>
            <div className="flex justify-end gap-3 pt-4">
              <Button variant="outline" onClick={() => setShowCreateModal(false)}>
                Cancel
              </Button>
              <Button onClick={() => setShowCreateModal(false)}>
                Create Cluster
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
