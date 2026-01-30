import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Loader2, CheckCircle, XCircle, AlertTriangle, Play } from "lucide-react";

interface RuntimeStatus {
  runtime: string;
  installed: boolean;
  running: boolean;
  version: string | null;
  path: string | null;
  docker_compatible: boolean;
  kubernetes_included: boolean;
}

interface RuntimeEnvironment {
  runtimes: RuntimeStatus[];
  docker_available: boolean;
  kubernetes_available: boolean;
  recommended: string | null;
  macos_version: string | null;
  can_use_apple_virtualization: boolean;
}

interface RuntimeStepProps {
  onComplete: (runtime: string) => void;
}

const runtimeLabels: Record<string, { name: string; description: string }> = {
  docker: { 
    name: "Docker Desktop", 
    description: "Most popular, full-featured container platform" 
  },
  orbstack: { 
    name: "OrbStack", 
    description: "Fast, lightweight Docker alternative for macOS" 
  },
  colima: { 
    name: "Colima", 
    description: "Lightweight Docker runtime using Lima VMs" 
  },
  podman: { 
    name: "Podman", 
    description: "Rootless container engine, Docker-compatible" 
  },
  lima: { 
    name: "Lima", 
    description: "Linux virtual machines for macOS" 
  },
  rancherdesktop: { 
    name: "Rancher Desktop", 
    description: "Kubernetes and container management" 
  },
};

export function RuntimeStep({ onComplete }: RuntimeStepProps) {
  const [scanning, setScanning] = useState(true);
  const [environment, setEnvironment] = useState<RuntimeEnvironment | null>(null);
  const [starting, setStarting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    scanEnvironment();
  }, []);

  const scanEnvironment = async () => {
    setScanning(true);
    setError(null);
    try {
      const result = await invoke<RuntimeEnvironment>("scan_runtime_environment");
      setEnvironment(result);
      
      // If we have a running Docker-compatible runtime, auto-advance
      if (result.docker_available && result.recommended) {
        onComplete(result.recommended);
      }
    } catch (err: any) {
      setError(err.message || "Failed to scan environment");
    } finally {
      setScanning(false);
    }
  };

  const startRuntime = async (runtime: string) => {
    setStarting(runtime);
    setError(null);
    try {
      await invoke("start_container_runtime", { runtime });
      // Wait a bit for it to start
      await new Promise(resolve => setTimeout(resolve, 3000));
      // Rescan
      await scanEnvironment();
    } catch (err: any) {
      setError(err.message || `Failed to start ${runtime}`);
    } finally {
      setStarting(null);
    }
  };

  const installedRuntimes = environment?.runtimes.filter(r => r.installed) || [];
  const runningRuntimes = environment?.runtimes.filter(r => r.running) || [];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Container Runtime</h2>
        <p className="text-muted-foreground mt-2">
          CTO Lite needs a container runtime to run Kubernetes locally.
        </p>
      </div>

      {scanning ? (
        <Card>
          <CardContent className="py-12 text-center">
            <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-muted-foreground" />
            <p className="text-muted-foreground">Scanning for container runtimes...</p>
          </CardContent>
        </Card>
      ) : environment?.docker_available ? (
        <Card className="border-green-200 bg-green-50/50 dark:border-green-800 dark:bg-green-950/20">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-green-700 dark:text-green-400">
              <CheckCircle className="h-5 w-5" />
              Container Runtime Ready
            </CardTitle>
            <CardDescription>
              {environment.recommended && runtimeLabels[environment.recommended]?.name} is running
              and Docker-compatible.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={() => onComplete(environment.recommended!)}>
              Continue
            </Button>
          </CardContent>
        </Card>
      ) : (
        <>
          {/* macOS version info */}
          {environment?.macos_version && (
            <div className="text-sm text-muted-foreground">
              macOS {environment.macos_version}
              {environment.can_use_apple_virtualization && (
                <Badge variant="outline" className="ml-2">
                  Apple Virtualization supported
                </Badge>
              )}
            </div>
          )}

          {installedRuntimes.length === 0 ? (
            <Card className="border-yellow-200 bg-yellow-50/50 dark:border-yellow-800 dark:bg-yellow-950/20">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-yellow-700 dark:text-yellow-400">
                  <AlertTriangle className="h-5 w-5" />
                  No Container Runtime Found
                </CardTitle>
                <CardDescription>
                  Please install one of the following container runtimes:
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="grid gap-3">
                  <a 
                    href="https://orbstack.dev" 
                    target="_blank" 
                    className="block p-3 border rounded-lg hover:bg-muted/50 transition-colors"
                  >
                    <div className="font-medium">OrbStack</div>
                    <div className="text-sm text-muted-foreground">
                      Recommended for macOS - fast and lightweight
                    </div>
                  </a>
                  <a 
                    href="https://docker.com/products/docker-desktop" 
                    target="_blank"
                    className="block p-3 border rounded-lg hover:bg-muted/50 transition-colors"
                  >
                    <div className="font-medium">Docker Desktop</div>
                    <div className="text-sm text-muted-foreground">
                      Most popular - works on macOS, Windows, Linux
                    </div>
                  </a>
                </div>
                <Button variant="outline" onClick={scanEnvironment} className="mt-4">
                  <Loader2 className="h-4 w-4 mr-2" />
                  Scan Again
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-3">
              <p className="text-sm text-muted-foreground">
                Found {installedRuntimes.length} runtime(s) installed. 
                {runningRuntimes.length === 0 && " Please start one:"}
              </p>
              
              {installedRuntimes.map((runtime) => {
                const info = runtimeLabels[runtime.runtime] || { 
                  name: runtime.runtime, 
                  description: "" 
                };
                
                return (
                  <Card 
                    key={runtime.runtime}
                    className={runtime.running ? "border-green-200" : ""}
                  >
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between">
                        <div>
                          <CardTitle className="text-base flex items-center gap-2">
                            {info.name}
                            {runtime.running ? (
                              <Badge className="bg-green-500">Running</Badge>
                            ) : (
                              <Badge variant="secondary">Stopped</Badge>
                            )}
                            {runtime.docker_compatible && runtime.running && (
                              <Badge variant="outline">Docker Compatible</Badge>
                            )}
                            {runtime.kubernetes_included && (
                              <Badge variant="outline">K8s Included</Badge>
                            )}
                          </CardTitle>
                          <CardDescription className="mt-1">
                            {info.description}
                            {runtime.version && (
                              <span className="ml-2 text-xs">({runtime.version})</span>
                            )}
                          </CardDescription>
                        </div>
                        <div>
                          {runtime.running ? (
                            <Button onClick={() => onComplete(runtime.runtime)}>
                              Use This
                            </Button>
                          ) : (
                            <Button 
                              variant="outline"
                              onClick={() => startRuntime(runtime.runtime)}
                              disabled={starting !== null}
                            >
                              {starting === runtime.runtime ? (
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                              ) : (
                                <Play className="h-4 w-4 mr-2" />
                              )}
                              Start
                            </Button>
                          )}
                        </div>
                      </div>
                    </CardHeader>
                  </Card>
                );
              })}
              
              <Button variant="ghost" onClick={scanEnvironment} className="mt-2">
                <Loader2 className="h-4 w-4 mr-2" />
                Refresh
              </Button>
            </div>
          )}
        </>
      )}

      {error && (
        <Card className="border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20">
          <CardContent className="pt-4">
            <p className="text-sm text-red-600 dark:text-red-400 flex items-center gap-2">
              <XCircle className="h-4 w-4" />
              {error}
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
