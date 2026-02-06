import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { 
  CheckCircle, 
  XCircle, 
  Loader2, 
  Server,
  Play,
  Trash2
} from "lucide-react";
import { useCluster, useSystemCheck } from "@/hooks/use-tauri";

interface InstallStepProps {
  onComplete: () => void;
  onBack: () => void;
}

type InstallStage = 'idle' | 'creating' | 'complete' | 'failed';

export function InstallStep({ onComplete, onBack }: InstallStepProps) {
  const cluster = useCluster();
  const system = useSystemCheck();
  const [stage, setStage] = useState<InstallStage>('idle');
  const [error, setError] = useState<string | null>(null);

  // Check if cluster already exists and is running
  useEffect(() => {
    if (cluster.data?.exists && cluster.data?.running) {
      setStage('complete');
    }
  }, [cluster.data]);

  const handleCreate = async () => {
    setStage('creating');
    setError(null);
    try {
      await cluster.create();
      setStage('complete');
    } catch (err: any) {
      setError(err.message || 'Failed to create cluster');
      setStage('failed');
    }
  };

  const handleDelete = async () => {
    setError(null);
    try {
      await cluster.remove();
      setStage('idle');
    } catch (err: any) {
      setError(err.message || 'Failed to delete cluster');
    }
  };

  const getProgress = () => {
    switch (stage) {
      case 'idle': return 0;
      case 'creating': return 50;
      case 'complete': return 100;
      case 'failed': return 0;
    }
  };

  const getMessage = () => {
    if (cluster.creating) return 'Creating Kind cluster (this may take a few minutes)...';
    if (cluster.deleting) return 'Deleting cluster...';
    switch (stage) {
      case 'idle': return 'Ready to create local Kubernetes cluster';
      case 'creating': return 'Setting up Kind cluster...';
      case 'complete': return 'Cluster is running and ready!';
      case 'failed': return error || 'Failed to create cluster';
    }
  };

  const isLoading = cluster.loading || cluster.creating || cluster.deleting;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Create Local Cluster</h2>
        <p className="text-muted-foreground mt-2">
          CTO App will create a local Kubernetes cluster using Kind.
        </p>
      </div>

      {/* System Check */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base flex items-center gap-2">
            <Server className="h-4 w-4" />
            System Status
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            <div className="flex items-center justify-between p-2 rounded bg-muted/50">
              <div className="flex items-center gap-2">
                {system.docker?.running ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-red-500" />
                )}
                <span>Docker</span>
              </div>
              <span className="text-sm text-muted-foreground">
                {system.docker?.running ? `Running (${system.docker.runtime})` : 'Not running'}
              </span>
            </div>
            <div className="flex items-center justify-between p-2 rounded bg-muted/50">
              <div className="flex items-center gap-2">
                {system.kind?.installed ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-red-500" />
                )}
                <span>Kind</span>
              </div>
              <span className="text-sm text-muted-foreground">
                {system.kind?.installed ? system.kind.version || 'Installed' : 'Not installed'}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Cluster Status */}
      <Card className={stage === 'complete' 
        ? "border-green-200 bg-green-50/50 dark:border-green-800 dark:bg-green-950/20"
        : stage === 'failed'
          ? "border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20"
          : ""
      }>
        <CardHeader className="pb-3">
          <CardTitle className="text-base flex items-center gap-2">
            {stage === 'complete' ? (
              <CheckCircle className="h-4 w-4 text-green-500" />
            ) : stage === 'failed' ? (
              <XCircle className="h-4 w-4 text-red-500" />
            ) : isLoading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Server className="h-4 w-4" />
            )}
            Kind Cluster
          </CardTitle>
          <CardDescription>
            {cluster.data?.exists 
              ? `Cluster "${cluster.data.name}" ${cluster.data.running ? 'is running' : 'exists but not running'}`
              : 'No cluster created yet'
            }
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Progress bar during creation */}
          {(cluster.creating || stage === 'creating') && (
            <div className="space-y-2">
              <Progress value={getProgress()} className="h-2" />
              <p className="text-sm text-muted-foreground text-center">
                {getMessage()}
              </p>
            </div>
          )}

          {/* Cluster nodes */}
          {cluster.data?.nodes && cluster.data.nodes.length > 0 && (
            <div className="space-y-2">
              <p className="text-sm font-medium">Nodes:</p>
              {cluster.data.nodes.map((node) => (
                <div 
                  key={node.name}
                  className="flex items-center justify-between p-2 rounded bg-muted/50 text-sm"
                >
                  <span>{node.name}</span>
                  <div className="flex items-center gap-2">
                    <span className="text-muted-foreground">{node.role}</span>
                    <span className={node.status === 'Ready' ? 'text-green-500' : 'text-yellow-500'}>
                      {node.status}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Success message */}
          {stage === 'complete' && !cluster.creating && (
            <div className="flex items-center gap-2 p-3 rounded bg-green-500/10 border border-green-500/20">
              <CheckCircle className="h-5 w-5 text-green-500" />
              <span className="text-green-600 dark:text-green-400 font-medium">
                Cluster is ready!
              </span>
            </div>
          )}

          {/* Error message */}
          {stage === 'failed' && (
            <div className="flex items-center gap-2 p-3 rounded bg-red-500/10 border border-red-500/20">
              <XCircle className="h-5 w-5 text-red-500" />
              <span className="text-red-600 dark:text-red-400">
                {error}
              </span>
            </div>
          )}

          {/* Actions for existing cluster */}
          {cluster.data?.exists && !cluster.creating && (
            <div className="flex gap-2">
              <Button 
                variant="outline" 
                size="sm"
                onClick={handleDelete}
                disabled={isLoading}
              >
                {cluster.deleting ? (
                  <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                ) : (
                  <Trash2 className="h-4 w-4 mr-1" />
                )}
                Delete Cluster
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Actions */}
      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={onBack} disabled={isLoading}>
          Back
        </Button>
        <div className="flex gap-2">
          {stage === 'complete' ? (
            <Button onClick={onComplete}>
              Complete Setup
            </Button>
          ) : stage === 'failed' ? (
            <Button onClick={handleCreate}>
              Retry
            </Button>
          ) : cluster.data?.exists ? (
            <Button onClick={onComplete}>
              Continue
            </Button>
          ) : (
            <Button 
              onClick={handleCreate} 
              disabled={!system.allReady || isLoading}
            >
              {cluster.creating ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Creating...
                </>
              ) : (
                <>
                  <Play className="h-4 w-4 mr-2" />
                  Create Cluster
                </>
              )}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
