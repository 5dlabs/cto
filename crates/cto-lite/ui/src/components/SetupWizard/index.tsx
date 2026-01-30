import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Progress } from '@/components/ui/progress'
import { useToast } from '@/hooks/use-toast'
import { 
  CheckCircle2, 
  Circle, 
  Container, 
  Github, 
  Cloud, 
  Key, 
  Settings2,
  Loader2,
  Server,
  ExternalLink
} from 'lucide-react'

interface SetupWizardProps {
  initialStep: number
  onComplete: () => void
}

const STEPS = [
  { id: 0, name: 'runtime', title: 'Container Runtime', icon: Container },
  { id: 1, name: 'stack', title: 'Choose Your Stack', icon: Settings2 },
  { id: 2, name: 'api_keys', title: 'API Keys', icon: Key },
  { id: 3, name: 'github', title: 'GitHub Connection', icon: Github },
  { id: 4, name: 'cloudflare', title: 'Cloudflare Connection', icon: Cloud },
  { id: 5, name: 'cluster', title: 'Create Cluster', icon: Server },
]

export function SetupWizard({ initialStep, onComplete }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState(initialStep)
  const [loading, setLoading] = useState(false)
  const { toast } = useToast()

  // Step-specific state
  const [runtimeDetected, setRuntimeDetected] = useState<string | null>(null)
  const [backendStack, setBackendStack] = useState<'go' | 'node'>('go')
  const [cli, setCli] = useState<'claude' | 'factory' | 'codex'>('claude')
  const [apiKey, setApiKey] = useState('')
  const [githubConnected, setGithubConnected] = useState(false)
  const [cloudflareConnected, setCloudflareConnected] = useState(false)
  const [clusterCreated, setClusterCreated] = useState(false)

  const progress = ((currentStep + 1) / STEPS.length) * 100

  async function handleRuntimeDetection() {
    setLoading(true)
    try {
      const result = await invoke<{
        detected: string | null
        available: Array<{ runtime: string; installed: boolean; running: boolean }>
        error: string | null
      }>('detect_container_runtime')

      if (result.detected) {
        setRuntimeDetected(result.detected)
        toast({
          title: 'Runtime Detected',
          description: `Found ${result.detected} running`,
        })
      } else {
        toast({
          title: 'No Runtime Found',
          description: result.error || 'Please install Docker, Colima, or Podman',
          variant: 'destructive',
        })
      }
    } catch (error) {
      toast({
        title: 'Detection Failed',
        description: String(error),
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  async function handleSaveStack() {
    setLoading(true)
    try {
      await invoke('set_config', { key: 'backend_stack', value: backendStack })
      await invoke('set_config', { key: 'cli', value: cli })
      toast({ title: 'Stack Saved' })
      nextStep()
    } catch (error) {
      toast({ title: 'Failed to save', description: String(error), variant: 'destructive' })
    } finally {
      setLoading(false)
    }
  }

  async function handleSaveApiKey() {
    if (!apiKey.trim()) {
      toast({ title: 'API Key Required', variant: 'destructive' })
      return
    }
    setLoading(true)
    try {
      const provider = cli === 'codex' ? 'openai' : 'anthropic'
      await invoke('set_api_key', { provider, apiKey })
      toast({ title: 'API Key Saved' })
      setApiKey('')
      nextStep()
    } catch (error) {
      toast({ title: 'Failed to save', description: String(error), variant: 'destructive' })
    } finally {
      setLoading(false)
    }
  }

  async function handleGitHubConnect() {
    setLoading(true)
    try {
      const authUrl = await invoke<string>('start_github_oauth')
      await openUrl(authUrl)
      toast({
        title: 'Opening Browser',
        description: 'Complete GitHub authorization in your browser',
      })
      // In real implementation, we'd listen for the OAuth callback
      // For now, simulate success after a delay
      setTimeout(() => {
        setGithubConnected(true)
        setLoading(false)
      }, 3000)
    } catch (error) {
      toast({ title: 'Connection Failed', description: String(error), variant: 'destructive' })
      setLoading(false)
    }
  }

  async function handleCloudflareConnect() {
    setLoading(true)
    try {
      const authUrl = await invoke<string>('start_cloudflare_oauth')
      await openUrl(authUrl)
      toast({
        title: 'Opening Browser',
        description: 'Complete Cloudflare authorization in your browser',
      })
      // Simulate success
      setTimeout(() => {
        setCloudflareConnected(true)
        setLoading(false)
      }, 3000)
    } catch (error) {
      toast({ title: 'Connection Failed', description: String(error), variant: 'destructive' })
      setLoading(false)
    }
  }

  async function handleCreateCluster() {
    setLoading(true)
    try {
      toast({ title: 'Creating Cluster', description: 'This may take a few minutes...' })
      await invoke('create_cluster')
      setClusterCreated(true)
      toast({ title: 'Cluster Created', description: 'CTO Lite is ready!' })
    } catch (error) {
      toast({ title: 'Cluster Creation Failed', description: String(error), variant: 'destructive' })
    } finally {
      setLoading(false)
    }
  }

  async function handleComplete() {
    try {
      await invoke('mark_setup_complete')
      onComplete()
    } catch (error) {
      console.error('Failed to mark setup complete:', error)
      onComplete()
    }
  }

  function nextStep() {
    if (currentStep < STEPS.length - 1) {
      setCurrentStep(currentStep + 1)
    }
  }

  function prevStep() {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1)
    }
  }

  function renderStepContent() {
    switch (currentStep) {
      case 0:
        return (
          <div className="space-y-6">
            <p className="text-muted-foreground">
              CTO Lite needs a container runtime to run Kubernetes locally.
              We support Docker Desktop, Colima, and Podman.
            </p>
            {runtimeDetected ? (
              <div className="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                <CheckCircle2 className="h-5 w-5 text-green-500" />
                <span className="text-green-500 font-medium">
                  Detected: {runtimeDetected}
                </span>
              </div>
            ) : (
              <Button onClick={handleRuntimeDetection} disabled={loading}>
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Detect Runtime
              </Button>
            )}
            {runtimeDetected && (
              <Button onClick={nextStep} className="w-full">
                Continue
              </Button>
            )}
          </div>
        )

      case 1:
        return (
          <div className="space-y-6">
            <div className="space-y-4">
              <Label>Backend Stack</Label>
              <div className="grid grid-cols-2 gap-4">
                <button
                  className={`p-4 rounded-lg border-2 text-left transition-colors ${
                    backendStack === 'go'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-primary/50'
                  }`}
                  onClick={() => setBackendStack('go')}
                >
                  <div className="font-semibold">Go (Grizz)</div>
                  <div className="text-sm text-muted-foreground">chi, grpc, pgx</div>
                </button>
                <button
                  className={`p-4 rounded-lg border-2 text-left transition-colors ${
                    backendStack === 'node'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-primary/50'
                  }`}
                  onClick={() => setBackendStack('node')}
                >
                  <div className="font-semibold">Node.js (Nova)</div>
                  <div className="text-sm text-muted-foreground">Elysia, Effect, Bun</div>
                </button>
              </div>
            </div>

            <div className="space-y-4">
              <Label>CLI Tool</Label>
              <div className="grid grid-cols-3 gap-4">
                {(['claude', 'factory', 'codex'] as const).map((option) => (
                  <button
                    key={option}
                    className={`p-4 rounded-lg border-2 text-center transition-colors ${
                      cli === option
                        ? 'border-primary bg-primary/5'
                        : 'border-border hover:border-primary/50'
                    }`}
                    onClick={() => setCli(option)}
                  >
                    <div className="font-semibold capitalize">{option}</div>
                  </button>
                ))}
              </div>
            </div>

            <Button onClick={handleSaveStack} disabled={loading} className="w-full">
              {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Continue
            </Button>
          </div>
        )

      case 2:
        return (
          <div className="space-y-6">
            <p className="text-muted-foreground">
              Enter your {cli === 'codex' ? 'OpenAI' : 'Anthropic'} API key.
              This is stored securely in your system keychain.
            </p>
            <div className="space-y-2">
              <Label htmlFor="apiKey">
                {cli === 'codex' ? 'OpenAI' : 'Anthropic'} API Key
              </Label>
              <Input
                id="apiKey"
                type="password"
                placeholder={cli === 'codex' ? 'sk-proj-...' : 'sk-ant-...'}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
              />
            </div>
            <Button onClick={handleSaveApiKey} disabled={loading || !apiKey.trim()} className="w-full">
              {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Save & Continue
            </Button>
          </div>
        )

      case 3:
        return (
          <div className="space-y-6">
            <p className="text-muted-foreground">
              Connect your GitHub account to allow CTO Lite to create branches and pull requests.
            </p>
            {githubConnected ? (
              <div className="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                <CheckCircle2 className="h-5 w-5 text-green-500" />
                <span className="text-green-500 font-medium">GitHub Connected</span>
              </div>
            ) : (
              <Button onClick={handleGitHubConnect} disabled={loading} className="w-full">
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                <Github className="mr-2 h-4 w-4" />
                Connect GitHub
                <ExternalLink className="ml-2 h-4 w-4" />
              </Button>
            )}
            {githubConnected && (
              <Button onClick={nextStep} className="w-full">
                Continue
              </Button>
            )}
          </div>
        )

      case 4:
        return (
          <div className="space-y-6">
            <p className="text-muted-foreground">
              Connect your Cloudflare account to enable secure webhook tunnels.
              This allows GitHub to send events to your local CTO Lite instance.
            </p>
            {cloudflareConnected ? (
              <div className="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                <CheckCircle2 className="h-5 w-5 text-green-500" />
                <span className="text-green-500 font-medium">Cloudflare Connected</span>
              </div>
            ) : (
              <Button onClick={handleCloudflareConnect} disabled={loading} className="w-full">
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                <Cloud className="mr-2 h-4 w-4" />
                Connect Cloudflare
                <ExternalLink className="ml-2 h-4 w-4" />
              </Button>
            )}
            {cloudflareConnected && (
              <Button onClick={nextStep} className="w-full">
                Continue
              </Button>
            )}
          </div>
        )

      case 5:
        return (
          <div className="space-y-6">
            <p className="text-muted-foreground">
              Create a local Kubernetes cluster to run CTO Lite.
              This uses Kind to spin up a lightweight cluster in your container runtime.
            </p>
            {clusterCreated ? (
              <>
                <div className="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                  <CheckCircle2 className="h-5 w-5 text-green-500" />
                  <span className="text-green-500 font-medium">Cluster Created</span>
                </div>
                <Button onClick={handleComplete} className="w-full">
                  🚀 Launch CTO Lite
                </Button>
              </>
            ) : (
              <Button onClick={handleCreateCluster} disabled={loading} className="w-full">
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                <Server className="mr-2 h-4 w-4" />
                Create Cluster
              </Button>
            )}
          </div>
        )

      default:
        return null
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-8 bg-gradient-to-br from-background to-muted/20">
      <div className="w-full max-w-2xl space-y-8">
        {/* Header */}
        <div className="text-center space-y-2">
          <h1 className="text-3xl font-bold">Welcome to CTO Lite</h1>
          <p className="text-muted-foreground">
            Let's set up your AI development environment
          </p>
        </div>

        {/* Progress */}
        <div className="space-y-2">
          <Progress value={progress} className="h-2" />
          <div className="flex justify-between text-sm text-muted-foreground">
            <span>Step {currentStep + 1} of {STEPS.length}</span>
            <span>{STEPS[currentStep].title}</span>
          </div>
        </div>

        {/* Step Indicators */}
        <div className="flex justify-center gap-2">
          {STEPS.map((step, index) => {
            const Icon = step.icon
            const isComplete = index < currentStep
            const isCurrent = index === currentStep
            return (
              <div
                key={step.id}
                className={`flex items-center justify-center w-10 h-10 rounded-full transition-colors ${
                  isComplete
                    ? 'bg-primary text-primary-foreground'
                    : isCurrent
                    ? 'bg-primary/20 text-primary border-2 border-primary'
                    : 'bg-muted text-muted-foreground'
                }`}
              >
                {isComplete ? (
                  <CheckCircle2 className="h-5 w-5" />
                ) : (
                  <Icon className="h-5 w-5" />
                )}
              </div>
            )
          })}
        </div>

        {/* Main Card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {(() => {
                const Icon = STEPS[currentStep].icon
                return <Icon className="h-5 w-5" />
              })()}
              {STEPS[currentStep].title}
            </CardTitle>
            <CardDescription>
              Step {currentStep + 1} of {STEPS.length}
            </CardDescription>
          </CardHeader>
          <CardContent>{renderStepContent()}</CardContent>
        </Card>

        {/* Navigation */}
        <div className="flex justify-between">
          <Button
            variant="ghost"
            onClick={prevStep}
            disabled={currentStep === 0}
          >
            ← Back
          </Button>
          <Button
            variant="ghost"
            onClick={nextStep}
            disabled={currentStep === STEPS.length - 1}
          >
            Skip →
          </Button>
        </div>
      </div>
    </div>
  )
}
