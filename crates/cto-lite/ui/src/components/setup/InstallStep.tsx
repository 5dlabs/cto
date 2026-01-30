import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { 
  CheckCircle, 
  XCircle, 
  Loader2, 
  AlertTriangle,
  Terminal,
  Download,
  Server,
  Settings
} from "lucide-react";

interface BinaryCheck {
  name: string;
  found: boolean;
  path: string | null;
  version: string | null;
}

interface InstallStatus {
  step: string;
  message: string;
  progress: number;
  error: string | null;
}

interface InstallStepProps {
  onComplete: () => void;
  onBack: () => void;
}

export function InstallStep({ onComplete, onBack }: InstallStepProps) {
  const [checking, setChecking] = useState(true);
  const [prerequisites, setPrerequisites] = useState<BinaryCheck[]>([]);
  const [installing, setInstalling] = useState(false);
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkPrerequisites();
    
    // Listen for installation progress
    const unlisten = listen<InstallStatus>("install-progress", (event) => {
      setStatus(event.payload);
      if (event.payload.step === "Complete") {
        setTimeout(() => onComplete(), 1000);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const checkPrerequisites = async () => {
    setChecking(true);
    setError(null);
    try {
      const result = await invoke<BinaryCheck[]>("check_prerequisites");
      setPrerequisites(result);
    } catch (err: any) {
      setError(err.message || "Failed to check prerequisites");
    } finally {
      setChecking(false);
    }
  };

  const startInstallation = async () => {
    setInstalling(true);
    setError(null);
    setStatus({
      step: "CheckingPrerequisites",
      message: "Starting installation...",
      progress: 0,
      error: null,
    });
    
    try {
      await invoke("run_installation");
    } catch (err: any) {
      setError(err.message || "Installation failed");
      setStatus(prev => prev ? { ...prev, step: "Failed", error: err.message } : null);
    } finally {
      setInstalling(false);
    }
  };

  const resetInstallation = async () => {
    try {
      await invoke("reset_installation");
      setStatus(null);
      await checkPrerequisites();
    } catch (err: any) {
      setError(err.message || "Failed to reset");
    }
  };

  const allPrereqsMet = prerequisites.every(p => p.found);
  const missingPrereqs = prerequisites.filter(p => !p.found);

  const stepIcons: Record<string, React.ReactNode> = {
    CheckingPrerequisites: <Settings className="h-4 w-4" />,
    InstallingBinaries: <Download className="h-4 w-4" />,
    CreatingCluster: <Server className="h-4 w-4" />,
    PullingImages: <Download className="h-4 w-4" />,
    DeployingServices: <Server className="h-4 w-4" />,
    ConfiguringIngress: <Settings className="h-4 w-4" />,
    Complete: <CheckCircle className="h-4 w-4 text-green-500" />,
    Failed: <XCircle className="h-4 w-4 text-red-500" />,
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Install CTO Lite</h2>
        <p className="text-muted-foreground mt-2">
          Set up the local Kubernetes cluster and deploy CTO Lite services.
        </p>
      </div>

      {/* Prerequisites Check */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base flex items-center gap-2">
            <Terminal className="h-4 w-4" />
            Prerequisites
          </CardTitle>
          <CardDescription>
            Required tools for CTO Lite
          </CardDescription>
        </CardHeader>
        <CardContent>
          {checking ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Checking prerequisites...
            </div>
          ) : (
            <div className="space-y-2">
              {prerequisites.map((prereq) => (
                <div 
                  key={prereq.name}
                  className="flex items-center justify-between p-2 rounded bg-muted/50"
                >
                  <div className="flex items-center gap-2">
                    {prereq.found ? (
                      <CheckCircle className="h-4 w-4 text-green-500" />
                    ) : (
                      <XCircle className="h-4 w-4 text-red-500" />
                    )}
                    <span className="font-medium">{prereq.name}</span>
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {prereq.found ? (
                      prereq.version || prereq.path || "Found"
                    ) : (
                      <Badge variant="destructive">Not found</Badge>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {!checking && missingPrereqs.length > 0 && (
            <div className="mt-4 p-3 rounded bg-yellow-500/10 border border-yellow-500/20">
              <div className="flex items-start gap-2">
                <AlertTriangle className="h-4 w-4 text-yellow-500 mt-0.5" />
                <div className="text-sm">
                  <p className="font-medium text-yellow-600 dark:text-yellow-400">
                    Missing: {missingPrereqs.map(p => p.name).join(", ")}
                  </p>
                  <p className="text-muted-foreground mt-1">
                    Install these tools before continuing. 
                    {missingPrereqs.some(p => p.name === "kind") && (
                      <> Try: <code className="text-xs bg-muted px-1 rounded">brew install kind</code></>
                    )}
                  </p>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Installation Progress */}
      {(installing || status) && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              {status?.step && stepIcons[status.step]}
              Installation Progress
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <Progress value={status?.progress || 0} className="h-2" />
            
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">
                {status?.message || "Preparing..."}
              </span>
              <span className="font-medium">{status?.progress || 0}%</span>
            </div>

            {status?.step === "Complete" && (
              <div className="flex items-center gap-2 p-3 rounded bg-green-500/10 border border-green-500/20">
                <CheckCircle className="h-5 w-5 text-green-500" />
                <span className="text-green-600 dark:text-green-400 font-medium">
                  Installation complete!
                </span>
              </div>
            )}

            {status?.step === "Failed" && (
              <div className="flex items-center gap-2 p-3 rounded bg-red-500/10 border border-red-500/20">
                <XCircle className="h-5 w-5 text-red-500" />
                <span className="text-red-600 dark:text-red-400">
                  {status.error || "Installation failed"}
                </span>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {error && !status?.error && (
        <Card className="border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20">
          <CardContent className="pt-4">
            <p className="text-sm text-red-600 dark:text-red-400 flex items-center gap-2">
              <XCircle className="h-4 w-4" />
              {error}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Actions */}
      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={onBack} disabled={installing}>
          Back
        </Button>
        <div className="flex gap-2">
          {status?.step === "Complete" ? (
            <Button onClick={onComplete}>
              Continue
            </Button>
          ) : status?.step === "Failed" ? (
            <>
              <Button variant="outline" onClick={resetInstallation}>
                Reset
              </Button>
              <Button onClick={startInstallation}>
                Retry
              </Button>
            </>
          ) : (
            <Button 
              onClick={startInstallation} 
              disabled={!allPrereqsMet || installing || checking}
            >
              {installing ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Installing...
                </>
              ) : (
                "Install"
              )}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
