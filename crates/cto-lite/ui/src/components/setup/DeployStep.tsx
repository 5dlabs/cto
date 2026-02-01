import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { 
  CheckCircle, 
  XCircle, 
  Loader2, 
  Rocket,
  Server,
  Key,
  Settings
} from "lucide-react";
import * as tauri from "@/lib/tauri";

interface DeployStepProps {
  onComplete: () => void;
  onBack: () => void;
}

type DeployStage = 'checking' | 'ready' | 'deploying' | 'complete' | 'failed';

interface PreflightCheck {
  name: string;
  status: 'pending' | 'checking' | 'passed' | 'failed';
  message?: string;
}

export function DeployStep({ onComplete, onBack }: DeployStepProps) {
  const [stage, setStage] = useState<DeployStage>('checking');
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [checks, setChecks] = useState<PreflightCheck[]>([
    { name: 'Docker', status: 'pending' },
    { name: 'Kind Cluster', status: 'pending' },
    { name: 'Helm', status: 'pending' },
    { name: 'API Keys', status: 'pending' },
  ]);
  const [release, setRelease] = useState<tauri.HelmRelease | null>(null);

  // Run preflight checks on mount
  useEffect(() => {
    runPreflightChecks();
  }, []);

  const updateCheck = (name: string, status: PreflightCheck['status'], message?: string) => {
    setChecks(prev => prev.map(c => 
      c.name === name ? { ...c, status, message } : c
    ));
  };

  const runPreflightChecks = async () => {
    setStage('checking');
    
    // Check Docker
    updateCheck('Docker', 'checking');
    try {
      const docker = await tauri.checkDocker();
      if (docker.running) {
        updateCheck('Docker', 'passed', `${docker.runtime} running`);
      } else {
        updateCheck('Docker', 'failed', 'Not running');
      }
    } catch (e) {
      updateCheck('Docker', 'failed', 'Check failed');
    }

    // Check Kind cluster
    updateCheck('Kind Cluster', 'checking');
    try {
      const cluster = await tauri.getClusterStatus();
      if (cluster.running) {
        updateCheck('Kind Cluster', 'passed', `${cluster.nodes.length} nodes`);
      } else if (cluster.exists) {
        updateCheck('Kind Cluster', 'failed', 'Exists but not running');
      } else {
        updateCheck('Kind Cluster', 'failed', 'Not created');
      }
    } catch (e) {
      updateCheck('Kind Cluster', 'failed', 'Check failed');
    }

    // Check Helm
    updateCheck('Helm', 'checking');
    try {
      const helmVersion = await tauri.checkHelm();
      if (helmVersion) {
        updateCheck('Helm', 'passed', helmVersion);
      } else {
        updateCheck('Helm', 'failed', 'Not installed');
      }
    } catch (e) {
      updateCheck('Helm', 'failed', 'Check failed');
    }

    // Check API Keys
    updateCheck('API Keys', 'checking');
    try {
      const hasAnthropic = await tauri.hasApiKey('anthropic');
      const hasOpenai = await tauri.hasApiKey('openai');
      if (hasAnthropic || hasOpenai) {
        const keys = [];
        if (hasAnthropic) keys.push('Anthropic');
        if (hasOpenai) keys.push('OpenAI');
        updateCheck('API Keys', 'passed', keys.join(', '));
      } else {
        updateCheck('API Keys', 'failed', 'No API keys configured');
      }
    } catch (e) {
      updateCheck('API Keys', 'failed', 'Check failed');
    }

    // Check if existing release
    try {
      const existing = await tauri.getReleaseStatus();
      if (existing) {
        setRelease(existing);
      }
    } catch (e) {
      // No release yet
    }

    // Determine if ready
    setTimeout(() => {
      setChecks(prev => {
        const allPassed = prev.every(c => c.status === 'passed');
        setStage(allPassed ? 'ready' : 'failed');
        return prev;
      });
    }, 500);
  };

  const handleDeploy = async () => {
    setStage('deploying');
    setError(null);
    setProgress(10);

    try {
      // Get API keys from keychain
      const anthropicKey = await tauri.getApiKey('anthropic');
      const openaiKey = await tauri.getApiKey('openai');
      const githubToken = await tauri.getApiKey('github');

      setProgress(30);

      // Get setup state for stack selection
      const state = await tauri.getSetupState();
      
      setProgress(50);

      // Deploy chart
      await tauri.deployChart({
        anthropicApiKey: anthropicKey || undefined,
        openaiApiKey: openaiKey || undefined,
        githubToken: githubToken || undefined,
        stack: state.stackSelection || 'grizz',
      });

      setProgress(90);

      // Get release status
      const newRelease = await tauri.getReleaseStatus();
      setRelease(newRelease);

      setProgress(100);
      setStage('complete');
    } catch (err: any) {
      setError(err.message || 'Deployment failed');
      setStage('failed');
    }
  };

  const handleUninstall = async () => {
    try {
      await tauri.uninstallChart();
      setRelease(null);
      runPreflightChecks();
    } catch (err: any) {
      setError(err.message || 'Uninstall failed');
    }
  };

  const getIcon = (status: PreflightCheck['status']) => {
    switch (status) {
      case 'passed': return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'failed': return <XCircle className="h-4 w-4 text-red-500" />;
      case 'checking': return <Loader2 className="h-4 w-4 animate-spin text-blue-500" />;
      default: return <div className="h-4 w-4 rounded-full bg-muted" />;
    }
  };

  const getCheckIcon = (name: string) => {
    switch (name) {
      case 'Docker': return <Server className="h-4 w-4" />;
      case 'Kind Cluster': return <Server className="h-4 w-4" />;
      case 'Helm': return <Settings className="h-4 w-4" />;
      case 'API Keys': return <Key className="h-4 w-4" />;
      default: return <CheckCircle className="h-4 w-4" />;
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Deploy CTO Lite</h2>
        <p className="text-muted-foreground mt-2">
          Deploy the CTO Lite platform to your local Kubernetes cluster.
        </p>
      </div>

      {/* Preflight Checks */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Preflight Checks</CardTitle>
          <CardDescription>
            Verifying all requirements are met
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {checks.map((check) => (
              <div 
                key={check.name}
                className="flex items-center justify-between p-2 rounded bg-muted/50"
              >
                <div className="flex items-center gap-3">
                  {getCheckIcon(check.name)}
                  <span className="font-medium">{check.name}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-sm text-muted-foreground">
                    {check.message}
                  </span>
                  {getIcon(check.status)}
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Deployment Status */}
      {(stage === 'deploying' || stage === 'complete') && (
        <Card className={stage === 'complete' 
          ? "border-green-200 bg-green-50/50 dark:border-green-800 dark:bg-green-950/20"
          : ""
        }>
          <CardHeader className="pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              {stage === 'complete' ? (
                <CheckCircle className="h-5 w-5 text-green-500" />
              ) : (
                <Loader2 className="h-5 w-5 animate-spin" />
              )}
              Deployment
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {stage === 'deploying' && (
              <div className="space-y-2">
                <Progress value={progress} className="h-2" />
                <p className="text-sm text-muted-foreground text-center">
                  Deploying CTO Lite to your cluster...
                </p>
              </div>
            )}
            
            {stage === 'complete' && release && (
              <div className="space-y-2">
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div className="text-muted-foreground">Release:</div>
                  <div className="font-medium">{release.name}</div>
                  <div className="text-muted-foreground">Status:</div>
                  <div className="font-medium text-green-600">{release.status}</div>
                  <div className="text-muted-foreground">Version:</div>
                  <div className="font-medium">{release.appVersion}</div>
                  <div className="text-muted-foreground">Namespace:</div>
                  <div className="font-medium">{release.namespace}</div>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Existing Release */}
      {release && stage !== 'deploying' && stage !== 'complete' && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Existing Installation</CardTitle>
            <CardDescription>
              CTO Lite is already deployed
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div className="text-muted-foreground">Status:</div>
                <div className="font-medium">{release.status}</div>
                <div className="text-muted-foreground">Version:</div>
                <div className="font-medium">{release.appVersion}</div>
              </div>
              <Button variant="outline" size="sm" onClick={handleUninstall}>
                Uninstall
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Error */}
      {error && (
        <div className="flex items-center gap-2 p-3 rounded bg-red-500/10 border border-red-500/20">
          <XCircle className="h-5 w-5 text-red-500" />
          <span className="text-red-600 dark:text-red-400">{error}</span>
        </div>
      )}

      {/* Actions */}
      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={onBack}>
          Back
        </Button>
        <div className="flex gap-2">
          {stage === 'complete' ? (
            <Button onClick={onComplete}>
              Finish Setup
            </Button>
          ) : stage === 'ready' ? (
            <Button onClick={handleDeploy}>
              <Rocket className="h-4 w-4 mr-2" />
              Deploy
            </Button>
          ) : release ? (
            <Button onClick={onComplete}>
              Continue
            </Button>
          ) : (
            <Button 
              onClick={runPreflightChecks} 
              variant="outline"
              disabled={stage === 'checking'}
            >
              {stage === 'checking' ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Checking...
                </>
              ) : (
                'Retry Checks'
              )}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
