import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from './ui/card';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Server, Database, Key, CheckCircle2, ArrowRight } from 'lucide-react';

interface SetupWizardProps {
  onComplete: () => void;
}

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const [step, setStep] = useState(1);
  const [config, setConfig] = useState({
    githubToken: '',
    clusterName: '',
    namespace: 'cto',
  });

  const steps = [
    { id: 1, title: 'GitHub Connection', icon: Key },
    { id: 2, title: 'Cluster Setup', icon: Server },
    { id: 3, title: 'Configuration', icon: Database },
  ];

  const handleNext = () => {
    if (step < 3) {
      setStep(step + 1);
    } else {
      onComplete();
    }
  };

  return (
    <Card className="w-full max-w-lg mx-auto">
      <CardHeader>
        <div className="flex items-center gap-2 mb-2">
          {steps.map((s) => (
            <div
              key={s.id}
              className={`flex items-center gap-2 ${
                s.id === step
                  ? 'text-zinc-100'
                  : s.id < step
                  ? 'text-emerald-500'
                  : 'text-zinc-600'
              }`}
            >
              <s.icon className="w-4 h-4" />
              <span className="text-sm">{s.title}</span>
            </div>
          ))}
        </div>
        <CardTitle>
          {step === 1 && 'Connect to GitHub'}
          {step === 2 && 'Configure Cluster'}
          {step === 3 && 'Final Setup'}
        </CardTitle>
        <CardDescription>
          {step === 1 && 'Enter your GitHub personal access token'}
          {step === 2 && 'Set up your local Kind cluster'}
          {step === 3 && 'Review and confirm your configuration'}
        </CardDescription>
      </CardHeader>
      <CardContent>
        {step === 1 && (
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="github-token">GitHub Personal Access Token</Label>
              <Input
                id="github-token"
                type="password"
                placeholder="ghp_xxxxxxxxxxxx"
                value={config.githubToken}
                onChange={(e) =>
                  setConfig({ ...config, githubToken: e.target.value })
                }
              />
            </div>
            <p className="text-xs text-zinc-500">
              Token needs repo and workflow permissions
            </p>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="cluster-name">Cluster Name</Label>
              <Input
                id="cluster-name"
                placeholder="cto-cluster"
                value={config.clusterName}
                onChange={(e) =>
                  setConfig({ ...config, clusterName: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="namespace">Default Namespace</Label>
              <Input
                id="namespace"
                placeholder="cto"
                value={config.namespace}
                onChange={(e) =>
                  setConfig({ ...config, namespace: e.target.value })
                }
              />
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="space-y-4">
            <div className="rounded-lg bg-zinc-800 p-4 space-y-2">
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                <span className="text-sm">GitHub: Connected</span>
              </div>
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                <span className="text-sm">
                  Cluster: {config.clusterName || 'cto-local'}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 text-emerald-500" />
                <span className="text-sm">Namespace: {config.namespace}</span>
              </div>
            </div>
          </div>
        )}

        <div className="flex justify-between mt-6">
          <Button
            variant="outline"
            onClick={() => setStep(Math.max(1, step - 1))}
            disabled={step === 1}
          >
            Back
          </Button>
          <Button onClick={handleNext}>
            {step === 3 ? 'Complete Setup' : 'Continue'}
            <ArrowRight className="w-4 h-4 ml-2" />
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
