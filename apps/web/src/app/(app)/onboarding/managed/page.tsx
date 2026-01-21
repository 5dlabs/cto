"use client";

import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card } from "@/components/ui/card";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// ONB-001: Provider options with status
const PROVIDERS = [
  {
    id: "latitude",
    name: "Latitude.sh",
    status: "supported",
    logo: "/providers/latitude.svg",
    description: "Global bare metal with instant deployment"
  },
  {
    id: "hetzner",
    name: "Hetzner",
    status: "coming_soon",
    logo: "/providers/hetzner.svg",
    description: "European bare metal leader"
  },
  {
    id: "vultr",
    name: "Vultr",
    status: "coming_soon",
    logo: "/providers/vultr.svg",
    description: "Cloud and bare metal hybrid"
  },
  {
    id: "ovh",
    name: "OVH",
    status: "coming_soon",
    logo: "/providers/ovh.svg",
    description: "European cloud provider"
  },
  {
    id: "scaleway",
    name: "Scaleway",
    status: "coming_soon",
    logo: "/providers/scaleway.svg",
    description: "French cloud infrastructure"
  },
];

// ONB-002: Region options by provider
const REGIONS: Record<string, Array<{ id: string; name: string; latency: string }>> = {
  latitude: [
    { id: "DAL", name: "Dallas", latency: "~15ms" },
    { id: "NYC", name: "New York", latency: "~20ms" },
    { id: "LAX", name: "Los Angeles", latency: "~25ms" },
    { id: "MIA", name: "Miami", latency: "~18ms" },
    { id: "CHI", name: "Chicago", latency: "~12ms" },
    { id: "ASH", name: "Ashburn", latency: "~22ms" },
    { id: "FRA", name: "Frankfurt", latency: "~85ms" },
    { id: "LON", name: "London", latency: "~95ms" },
    { id: "SYD", name: "Sydney", latency: "~180ms" },
    { id: "TYO", name: "Tokyo", latency: "~120ms" },
    { id: "SAO", name: "São Paulo", latency: "~140ms" },
  ],
};

// ONB-003: Cluster size options
const CLUSTER_SIZES = [
  {
    id: "small",
    name: "Small",
    nodes: 2,
    description: "1 Control Plane + 1 Worker",
    plan: "c2-small-x86",
    estimate: "$200/mo",
  },
  {
    id: "medium",
    name: "Medium",
    nodes: 4,
    description: "1 Control Plane + 3 Workers",
    plan: "c2-medium-x86",
    estimate: "$600/mo",
  },
  {
    id: "large",
    name: "Large (HA)",
    nodes: 8,
    description: "3 Control Planes + 5 Workers",
    plan: "c2-large-x86",
    estimate: "$1,500/mo",
  },
];

type OnboardingStep =
  | "provider"
  | "region"
  | "size"
  | "credentials"
  | "validating"
  | "github"
  | "creating_repo"
  | "provisioning"
  | "complete";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  component?: "provider_select" | "region_select" | "size_select" | "credentials_input" | "github_input";
}

interface OnboardingState {
  provider: string | null;
  region: string | null;
  size: string | null;
  apiKeyValidated: boolean;
  githubOrg: string | null;
  githubAppInstalled: boolean;
  gitopsRepoCreated: boolean;
}

const INITIAL_MESSAGES: Message[] = [
  {
    id: "1",
    role: "assistant",
    content:
      "Welcome to CTO Managed Dedicated! I'm Bolt, your infrastructure specialist. I'll help you provision a dedicated bare metal cluster for your team.\n\nFirst, select your preferred provider. We'll deploy Kubernetes on bare metal with full isolation and control.",
    component: "provider_select",
  },
];

export default function ManagedOnboardingPage() {
  const [messages, setMessages] = useState<Message[]>(INITIAL_MESSAGES);
  const [step, setStep] = useState<OnboardingStep>("provider");
  const [isLoading, setIsLoading] = useState(false);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);
  const [validationResult, setValidationResult] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  const [state, setState] = useState<OnboardingState>({
    provider: null,
    region: null,
    size: null,
    apiKeyValidated: false,
    githubOrg: null,
    githubAppInstalled: false,
    gitopsRepoCreated: false,
  });
  const [githubOrgInput, setGithubOrgInput] = useState("");
  const [githubError, setGithubError] = useState<string | null>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  // ONB-001: Provider selection handler
  const handleProviderSelect = (providerId: string) => {
    const provider = PROVIDERS.find(p => p.id === providerId);
    if (!provider || provider.status !== "supported") return;

    setState(prev => ({ ...prev, provider: providerId }));

    setMessages(prev => [
      ...prev,
      {
        id: Date.now().toString(),
        role: "user",
        content: `Selected ${provider.name}`,
      },
      {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: `Excellent choice! ${provider.name} offers great performance and availability.\n\nNow select a deployment region. Choose the one closest to your users or development team for best latency.`,
        component: "region_select",
      },
    ]);
    setStep("region");
  };

  // ONB-002: Region selection handler
  const handleRegionSelect = (regionId: string) => {
    const regions = REGIONS[state.provider!] || [];
    const region = regions.find(r => r.id === regionId);
    if (!region) return;

    setState(prev => ({ ...prev, region: regionId }));

    setMessages(prev => [
      ...prev,
      {
        id: Date.now().toString(),
        role: "user",
        content: `Selected ${region.name} (${regionId})`,
      },
      {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: `${region.name} it is! Expected latency: ${region.latency}\n\nNow choose your cluster size. This determines how many servers will be provisioned and the total compute capacity available.`,
        component: "size_select",
      },
    ]);
    setStep("size");
  };

  // ONB-003: Cluster size selection handler
  const handleSizeSelect = (sizeId: string) => {
    const size = CLUSTER_SIZES.find(s => s.id === sizeId);
    if (!size) return;

    setState(prev => ({ ...prev, size: sizeId }));

    setMessages(prev => [
      ...prev,
      {
        id: Date.now().toString(),
        role: "user",
        content: `Selected ${size.name} (${size.nodes} nodes)`,
      },
      {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: `Perfect! You've selected:\n\n• **Provider**: ${PROVIDERS.find(p => p.id === state.provider)?.name}\n• **Region**: ${REGIONS[state.provider!]?.find(r => r.id === state.region)?.name}\n• **Size**: ${size.name} (${size.description})\n• **Estimated Cost**: ${size.estimate}\n\nNow I need your ${PROVIDERS.find(p => p.id === state.provider)?.name} API key to provision the servers.\n\n**Security Note**: Your API key is encrypted at rest, never logged, and only accessible by our admin agents. No human can view it after entry.`,
        component: "credentials_input",
      },
    ]);
    setStep("credentials");
  };

  // ONB-004: API key validation handler
  const handleCredentialsSubmit = async () => {
    if (!apiKeyInput.trim()) return;

    setIsLoading(true);
    setValidationError(null);
    setValidationResult(null);

    setMessages(prev => [
      ...prev,
      {
        id: Date.now().toString(),
        role: "user",
        content: `API Key: ${apiKeyInput.slice(0, 8)}...${apiKeyInput.slice(-4)}`,
      },
    ]);
    setStep("validating");

    try {
      // Call validation API
      const response = await fetch("/api/validate-provider-key", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          provider: state.provider,
          region: state.region,
          apiKey: apiKeyInput,
        }),
      });

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.error || "Validation failed");
      }

      // Success - key never stored in state, only in memory until stored in OpenBao
      setState(prev => ({ ...prev, apiKeyValidated: true }));
      setValidationResult(`API key valid! ${result.serversAvailable || 0} servers available in ${state.region}.`);

      // ONB-005: After API key validation, ask for GitHub organization
      setMessages(prev => [
        ...prev,
        {
          id: Date.now().toString(),
          role: "assistant",
          content: `API key validated successfully!\n\n✓ ${result.serversAvailable || 0} servers available in ${state.region}\n✓ Required permissions verified\n✓ Key stored securely in OpenBao\n\nNow I need your GitHub organization name. We'll create a GitOps repository at **{your-org}/cto-argocd** to manage your cluster's configuration.\n\nThis repository will contain ArgoCD Application manifests that sync to your cluster. You'll have full control over the repo.`,
          component: "github_input",
        },
      ]);
      setStep("github");

      // Clear API key from memory immediately after validation
      setApiKeyInput("");

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "Validation failed";
      setValidationError(errorMessage);

      setMessages(prev => [
        ...prev,
        {
          id: Date.now().toString(),
          role: "assistant",
          content: `Validation failed: ${errorMessage}\n\nPlease check your API key and try again. Make sure the key has permissions to create servers and VLANs.`,
          component: "credentials_input",
        },
      ]);
      setStep("credentials");
    } finally {
      setIsLoading(false);
    }
  };

  // ONB-005: GitHub Organization validation handler
  const handleGithubOrgSubmit = async () => {
    if (!githubOrgInput.trim()) return;

    setIsLoading(true);
    setGithubError(null);

    setMessages(prev => [
      ...prev,
      {
        id: Date.now().toString(),
        role: "user",
        content: `GitHub Organization: ${githubOrgInput}`,
      },
    ]);

    try {
      // Validate GitHub org and check for app installation
      const response = await fetch("/api/onboarding/managed/github", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          action: "validate",
          githubOrg: githubOrgInput,
        }),
      });

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.error || "GitHub validation failed");
      }

      if (!result.appInstalled) {
        // App not installed - show installation link
        setGithubError(null);
        setMessages(prev => [
          ...prev,
          {
            id: Date.now().toString(),
            role: "assistant",
            content: `I found the organization **${githubOrgInput}**, but the 5D Labs GitHub App is not installed.\n\nPlease install the app to allow us to create the GitOps repository:\n\n🔗 [Install 5D Labs GitHub App](${result.installUrl || "https://github.com/apps/5dlabs-cto"})\n\nAfter installing, click "Check Again" below.`,
            component: "github_input",
          },
        ]);
        setState(prev => ({ ...prev, githubOrg: githubOrgInput, githubAppInstalled: false }));
        return;
      }

      // App is installed - proceed to create repo
      setState(prev => ({ ...prev, githubOrg: githubOrgInput, githubAppInstalled: true }));
      setStep("creating_repo");

      setMessages(prev => [
        ...prev,
        {
          id: Date.now().toString(),
          role: "assistant",
          content: `GitHub App is installed on **${githubOrgInput}**. Creating your GitOps repository...`,
        },
      ]);

      // ONB-006: Create customer GitOps repo
      const createResponse = await fetch("/api/onboarding/managed/github", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          action: "create_repo",
          githubOrg: githubOrgInput,
          provider: state.provider,
          region: state.region,
          size: state.size,
        }),
      });

      const createResult = await createResponse.json();

      if (!createResponse.ok) {
        throw new Error(createResult.error || "Failed to create GitOps repository");
      }

      setState(prev => ({ ...prev, gitopsRepoCreated: true }));

      setMessages(prev => [
        ...prev,
        {
          id: Date.now().toString(),
          role: "assistant",
          content: `GitOps repository created!\n\n✓ Repository: **${githubOrgInput}/cto-argocd**\n✓ ArgoCD manifests initialized\n✓ Tenant-specific values.yaml configured\n\n**Summary of your setup:**\n• **Provider**: ${PROVIDERS.find(p => p.id === state.provider)?.name}\n• **Region**: ${REGIONS[state.provider!]?.find(r => r.id === state.region)?.name}\n• **Size**: ${CLUSTER_SIZES.find(s => s.id === state.size)?.name}\n• **GitOps Repo**: ${githubOrgInput}/cto-argocd\n\nReady to provision your cluster?`,
        },
      ]);
      setStep("provisioning");

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "GitHub validation failed";
      setGithubError(errorMessage);

      setMessages(prev => [
        ...prev,
        {
          id: Date.now().toString(),
          role: "assistant",
          content: `GitHub validation failed: ${errorMessage}\n\nPlease check the organization name and try again. Make sure you have admin access to the organization.`,
          component: "github_input",
        },
      ]);
      setStep("github");
    } finally {
      setIsLoading(false);
    }
  };

  const handleStartProvisioning = async () => {
    setIsLoading(true);

    setMessages(prev => [
      ...prev,
      {
        id: Date.now().toString(),
        role: "user",
        content: "Start provisioning",
      },
      {
        id: (Date.now() + 1).toString(),
        role: "assistant",
        content: `Provisioning started! I'm creating a BoltRun task to deploy your cluster.\n\n**What's happening:**\n1. Creating ${CLUSTER_SIZES.find(s => s.id === state.size)?.nodes} bare metal servers\n2. Setting up private VLAN\n3. Installing Talos Linux\n4. Bootstrapping Kubernetes\n5. Deploying platform stack\n6. Connecting to CTO control plane\n\nThis typically takes 15-20 minutes. You can track progress on your dashboard.`,
      },
    ]);

    try {
      // Create BoltRun task via API
      const response = await fetch("/api/onboarding/managed", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          action: "provision",
          provider: state.provider,
          region: state.region,
          size: state.size,
          githubOrg: state.githubOrg,
        }),
      });

      if (!response.ok) {
        throw new Error("Failed to start provisioning");
      }

      setStep("complete");

      setMessages(prev => [
        ...prev,
        {
          id: Date.now().toString(),
          role: "assistant",
          content: `BoltRun task created! Your cluster **acme-prod** is being provisioned.\n\n✓ Task ID: bolt-acme-provision\n✓ Expected completion: ~20 minutes\n\nGo to your dashboard to monitor progress and see when your cluster is ready.`,
        },
      ]);

    } catch (error) {
      setMessages(prev => [
        ...prev,
        {
          id: Date.now().toString(),
          role: "assistant",
          content: `Failed to start provisioning. Please try again or contact support.`,
        },
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  const getStepNumber = (): number => {
    switch (step) {
      case "provider": return 1;
      case "region": return 2;
      case "size": return 3;
      case "credentials":
      case "validating": return 4;
      case "github":
      case "creating_repo": return 5;
      case "provisioning":
      case "complete": return 6;
      default: return 1;
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border">
        <div className="flex items-center gap-3">
          <Avatar className="h-10 w-10 border-2 border-amber-500">
            <AvatarImage src="/agents/bolt.png" alt="Bolt" />
            <AvatarFallback className="bg-gradient-to-br from-amber-500 to-orange-600 text-white">
              B
            </AvatarFallback>
          </Avatar>
          <div>
            <h1 className="font-semibold">Managed Dedicated Setup with Bolt</h1>
            <p className="text-sm text-muted-foreground">Infrastructure Specialist</p>
          </div>
        </div>
        <Badge variant="outline" className="text-xs">
          Step {getStepNumber()} of 6
        </Badge>
      </div>

      {/* Messages */}
      <ScrollArea className="flex-1 p-6" ref={scrollRef}>
        <div className="max-w-2xl mx-auto space-y-6">
          {messages.map((message) => (
            <div key={message.id}>
              <div
                className={cn(
                  "flex gap-3",
                  message.role === "user" ? "justify-end" : "justify-start"
                )}
              >
                {message.role === "assistant" && (
                  <Avatar className="h-8 w-8 shrink-0">
                    <AvatarImage src="/agents/bolt.png" alt="Bolt" />
                    <AvatarFallback className="bg-gradient-to-br from-amber-500 to-orange-600 text-white text-xs">
                      B
                    </AvatarFallback>
                  </Avatar>
                )}
                <div
                  className={cn(
                    "flex flex-col gap-3 max-w-[80%]",
                    message.role === "user" ? "items-end" : "items-start"
                  )}
                >
                  <Card
                    className={cn(
                      "px-4 py-3",
                      message.role === "user"
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted"
                    )}
                  >
                    <p className="text-sm whitespace-pre-wrap">{message.content}</p>
                  </Card>
                </div>
              </div>

              {/* ONB-001: Provider Selection Component */}
              {message.component === "provider_select" && step === "provider" && (
                <div className="mt-4 ml-11 grid gap-2">
                  {PROVIDERS.map((provider) => (
                    <Button
                      key={provider.id}
                      variant={provider.status === "supported" ? "outline" : "ghost"}
                      className={cn(
                        "justify-start h-auto py-3 px-4",
                        provider.status !== "supported" && "opacity-50 cursor-not-allowed"
                      )}
                      onClick={() => handleProviderSelect(provider.id)}
                      disabled={provider.status !== "supported" || isLoading}
                    >
                      <div className="flex flex-col items-start gap-1">
                        <div className="flex items-center gap-2">
                          <span className="font-medium">{provider.name}</span>
                          {provider.status === "coming_soon" && (
                            <Badge variant="secondary" className="text-xs">Coming Soon</Badge>
                          )}
                        </div>
                        <span className="text-xs text-muted-foreground">{provider.description}</span>
                      </div>
                    </Button>
                  ))}
                </div>
              )}

              {/* ONB-002: Region Selection Component */}
              {message.component === "region_select" && step === "region" && state.provider && (
                <div className="mt-4 ml-11 grid grid-cols-2 gap-2">
                  {REGIONS[state.provider]?.map((region) => (
                    <Button
                      key={region.id}
                      variant="outline"
                      className="justify-start h-auto py-2 px-3"
                      onClick={() => handleRegionSelect(region.id)}
                      disabled={isLoading}
                    >
                      <div className="flex flex-col items-start">
                        <span className="font-medium">{region.name}</span>
                        <span className="text-xs text-muted-foreground">{region.id} • {region.latency}</span>
                      </div>
                    </Button>
                  ))}
                </div>
              )}

              {/* ONB-003: Cluster Size Selection Component */}
              {message.component === "size_select" && step === "size" && (
                <div className="mt-4 ml-11 grid gap-2">
                  {CLUSTER_SIZES.map((size) => (
                    <Button
                      key={size.id}
                      variant="outline"
                      className="justify-start h-auto py-3 px-4"
                      onClick={() => handleSizeSelect(size.id)}
                      disabled={isLoading}
                    >
                      <div className="flex flex-col items-start gap-1 w-full">
                        <div className="flex justify-between w-full">
                          <span className="font-medium">{size.name}</span>
                          <span className="text-sm text-muted-foreground">{size.estimate}</span>
                        </div>
                        <span className="text-xs text-muted-foreground">
                          {size.nodes} nodes • {size.description} • {size.plan}
                        </span>
                      </div>
                    </Button>
                  ))}
                </div>
              )}

              {/* ONB-004: API Key Input Component */}
              {message.component === "credentials_input" && step === "credentials" && (
                <div className="mt-4 ml-11 space-y-3">
                  <div className="flex gap-2">
                    <Input
                      type="password"
                      value={apiKeyInput}
                      onChange={(e) => setApiKeyInput(e.target.value)}
                      placeholder="Enter your API key..."
                      className="flex-1"
                      disabled={isLoading}
                    />
                    <Button
                      onClick={handleCredentialsSubmit}
                      disabled={isLoading || !apiKeyInput.trim()}
                    >
                      {isLoading ? "Validating..." : "Validate"}
                    </Button>
                  </div>
                  {validationError && (
                    <p className="text-sm text-destructive">{validationError}</p>
                  )}
                  <p className="text-xs text-muted-foreground">
                    Your API key is encrypted and stored securely. It is never logged or exposed.
                  </p>
                </div>
              )}

              {/* ONB-005: GitHub Organization Input Component */}
              {message.component === "github_input" && step === "github" && (
                <div className="mt-4 ml-11 space-y-3">
                  <div className="flex gap-2">
                    <Input
                      type="text"
                      value={githubOrgInput}
                      onChange={(e) => setGithubOrgInput(e.target.value)}
                      placeholder="your-github-org"
                      className="flex-1"
                      disabled={isLoading}
                    />
                    <Button
                      onClick={handleGithubOrgSubmit}
                      disabled={isLoading || !githubOrgInput.trim()}
                    >
                      {isLoading ? "Checking..." : state.githubOrg ? "Check Again" : "Verify"}
                    </Button>
                  </div>
                  {githubError && (
                    <p className="text-sm text-destructive">{githubError}</p>
                  )}
                  <p className="text-xs text-muted-foreground">
                    We will create <strong>{githubOrgInput || "your-org"}/cto-argocd</strong> for GitOps.
                  </p>
                  {state.githubOrg && !state.githubAppInstalled && (
                    <div className="flex gap-2 mt-2">
                      <Button
                        variant="outline"
                        onClick={() => window.open("https://github.com/apps/5dlabs-cto", "_blank")}
                      >
                        Install GitHub App
                      </Button>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}

          {/* Provisioning button */}
          {step === "provisioning" && state.apiKeyValidated && state.gitopsRepoCreated && (
            <div className="ml-11 mt-4">
              <Button onClick={handleStartProvisioning} disabled={isLoading}>
                {isLoading ? "Starting..." : "Start Provisioning"}
              </Button>
            </div>
          )}

          {/* Complete state */}
          {step === "complete" && (
            <div className="ml-11 mt-4 flex gap-2">
              <Button onClick={() => window.location.href = "/dashboard"}>
                Go to Dashboard
              </Button>
            </div>
          )}

          {isLoading && step === "validating" && (
            <div className="flex gap-3">
              <Avatar className="h-8 w-8 shrink-0">
                <AvatarFallback className="bg-gradient-to-br from-amber-500 to-orange-600 text-white text-xs">
                  B
                </AvatarFallback>
              </Avatar>
              <Card className="px-4 py-3 bg-muted">
                <div className="flex gap-1">
                  <span className="w-2 h-2 bg-muted-foreground/50 rounded-full animate-bounce" />
                  <span className="w-2 h-2 bg-muted-foreground/50 rounded-full animate-bounce [animation-delay:0.1s]" />
                  <span className="w-2 h-2 bg-muted-foreground/50 rounded-full animate-bounce [animation-delay:0.2s]" />
                </div>
              </Card>
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
