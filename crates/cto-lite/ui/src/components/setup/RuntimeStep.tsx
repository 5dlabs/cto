import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Loader2, CheckCircle, XCircle, AlertTriangle, RefreshCw, ExternalLink } from "lucide-react";
import { useDockerStatus, useKindStatus } from "@/hooks/use-tauri";

interface RuntimeStepProps {
  onComplete: () => void;
}

const runtimeLabels: Record<string, { name: string; description: string; url: string }> = {
  docker: { 
    name: "Docker Desktop", 
    description: "Most popular, full-featured container platform",
    url: "https://docker.com/products/docker-desktop"
  },
  orbstack: { 
    name: "OrbStack", 
    description: "Fast, lightweight Docker alternative for macOS",
    url: "https://orbstack.dev"
  },
  colima: { 
    name: "Colima", 
    description: "Lightweight Docker runtime using Lima VMs",
    url: "https://github.com/abiosoft/colima"
  },
  podman: { 
    name: "Podman", 
    description: "Rootless container engine, Docker-compatible",
    url: "https://podman.io"
  },
  rancherDesktop: { 
    name: "Rancher Desktop", 
    description: "Kubernetes and container management",
    url: "https://rancherdesktop.io"
  },
};

export function RuntimeStep({ onComplete }: RuntimeStepProps) {
  const docker = useDockerStatus();
  const kind = useKindStatus();
  const [autoAdvanced, setAutoAdvanced] = useState(false);

  // Auto-advance if everything is ready
  useEffect(() => {
    if (!autoAdvanced && docker.data?.installed && docker.data?.running && kind.data?.installed) {
      setAutoAdvanced(true);
      // Small delay for UX
      setTimeout(() => onComplete(), 500);
    }
  }, [docker.data, kind.data, autoAdvanced, onComplete]);

  const loading = docker.loading || kind.loading;
  const runtimeInfo = docker.data?.runtime ? runtimeLabels[docker.data.runtime] : null;

  const refresh = () => {
    docker.refetch();
    kind.refetch();
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-2xl font-bold">Container Runtime</h2>
          <p className="text-muted-foreground mt-2">
            CTO Lite needs Docker and Kind to run Kubernetes locally.
          </p>
        </div>
        <Card>
          <CardContent className="py-12 text-center">
            <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-muted-foreground" />
            <p className="text-muted-foreground">Scanning for container runtimes...</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  const dockerReady = docker.data?.installed && docker.data?.running;
  const kindReady = kind.data?.installed;
  const allReady = dockerReady && kindReady;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Container Runtime</h2>
        <p className="text-muted-foreground mt-2">
          CTO Lite needs Docker and Kind to run Kubernetes locally.
        </p>
      </div>

      {/* Docker Status */}
      <Card className={dockerReady 
        ? "border-green-200 bg-green-50/50 dark:border-green-800 dark:bg-green-950/20"
        : docker.data?.installed 
          ? "border-yellow-200 bg-yellow-50/50 dark:border-yellow-800 dark:bg-yellow-950/20"
          : "border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20"
      }>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            {dockerReady ? (
              <CheckCircle className="h-5 w-5 text-green-600" />
            ) : docker.data?.installed ? (
              <AlertTriangle className="h-5 w-5 text-yellow-600" />
            ) : (
              <XCircle className="h-5 w-5 text-red-600" />
            )}
            Docker
            {docker.data?.installed && (
              <Badge variant={dockerReady ? "default" : "secondary"}>
                {dockerReady ? "Running" : "Stopped"}
              </Badge>
            )}
          </CardTitle>
          <CardDescription>
            {dockerReady && runtimeInfo ? (
              <>Using {runtimeInfo.name} {docker.data?.version && `(${docker.data.version})`}</>
            ) : docker.data?.installed ? (
              <>Docker is installed but not running. Please start it.</>
            ) : (
              <>No Docker-compatible runtime found.</>
            )}
          </CardDescription>
        </CardHeader>
        {!docker.data?.installed && (
          <CardContent className="pt-0 space-y-3">
            <p className="text-sm text-muted-foreground">
              Install one of these container runtimes:
            </p>
            <div className="grid gap-2">
              <a 
                href="https://orbstack.dev" 
                target="_blank" 
                rel="noopener"
                className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                <div>
                  <div className="font-medium">OrbStack</div>
                  <div className="text-sm text-muted-foreground">
                    Recommended for macOS - fast and lightweight
                  </div>
                </div>
                <ExternalLink className="h-4 w-4 text-muted-foreground" />
              </a>
              <a 
                href="https://docker.com/products/docker-desktop" 
                target="_blank"
                rel="noopener"
                className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                <div>
                  <div className="font-medium">Docker Desktop</div>
                  <div className="text-sm text-muted-foreground">
                    Works on macOS, Windows, Linux
                  </div>
                </div>
                <ExternalLink className="h-4 w-4 text-muted-foreground" />
              </a>
            </div>
          </CardContent>
        )}
      </Card>

      {/* Kind Status */}
      <Card className={kindReady 
        ? "border-green-200 bg-green-50/50 dark:border-green-800 dark:bg-green-950/20"
        : "border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20"
      }>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            {kindReady ? (
              <CheckCircle className="h-5 w-5 text-green-600" />
            ) : (
              <XCircle className="h-5 w-5 text-red-600" />
            )}
            Kind (Kubernetes in Docker)
            {kindReady && kind.data?.version && (
              <Badge variant="outline">{kind.data.version}</Badge>
            )}
          </CardTitle>
          <CardDescription>
            {kindReady ? (
              <>Kind is installed and ready to create clusters.</>
            ) : (
              <>Kind is required to run local Kubernetes clusters.</>
            )}
          </CardDescription>
        </CardHeader>
        {!kindReady && (
          <CardContent className="pt-0">
            <p className="text-sm text-muted-foreground mb-3">
              Install Kind using one of these methods:
            </p>
            <div className="space-y-2 font-mono text-sm bg-muted p-3 rounded-lg">
              <div># Using Homebrew (macOS/Linux)</div>
              <div className="text-primary">brew install kind</div>
              <div className="mt-2"># Or download binary</div>
              <a 
                href="https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
                target="_blank"
                rel="noopener"
                className="text-primary hover:underline flex items-center gap-1"
              >
                kind.sigs.k8s.io <ExternalLink className="h-3 w-3" />
              </a>
            </div>
          </CardContent>
        )}
      </Card>

      {/* Actions */}
      <div className="flex justify-between items-center">
        <Button variant="outline" onClick={refresh}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Refresh
        </Button>
        
        {allReady ? (
          <Button onClick={onComplete}>
            Continue
          </Button>
        ) : (
          <p className="text-sm text-muted-foreground">
            Install the requirements above to continue
          </p>
        )}
      </div>

      {(docker.error || kind.error) && (
        <Card className="border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20">
          <CardContent className="pt-4">
            <p className="text-sm text-red-600 dark:text-red-400 flex items-center gap-2">
              <XCircle className="h-4 w-4" />
              {docker.error?.message || kind.error?.message}
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
