# Multi-Agent Platform: Packaging Implementation Roadmap

## Quick Start: Proof of Concept Script

### Immediate Action: Create Minimal Installer Script

```bash
#!/bin/bash
# install-agent-platform.sh - Proof of concept installer

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PLATFORM_VERSION="0.1.0"
K3S_VERSION="v1.31.0+k3s1"
PLATFORM_DIR="$HOME/.agent-platform"

# Functions
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker first."
        exit 1
    fi
    
    # Check system resources
    MEM_GB=$(free -g | awk '/^Mem:/{print $2}')
    if [ "$MEM_GB" -lt 8 ]; then
        log_warn "System has less than 8GB RAM. Minimal profile recommended."
    fi
}

# Install k3s cluster
install_k3s() {
    log_info "Installing k3s Kubernetes cluster..."
    curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION=$K3S_VERSION sh -s - \
        --disable traefik \
        --write-kubeconfig-mode 644
    
    export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
    kubectl wait --for=condition=Ready nodes --all --timeout=60s
}

# Create GitHub Apps automatically
create_github_apps() {
    log_info "Setting up GitHub integration..."
    # This would use GitHub API to create apps
    # For now, we'll use environment variables
    
    read -p "Enter your GitHub organization/username: " GITHUB_ORG
    read -p "Enter repository for testing: " GITHUB_REPO
    
    # In production, this would create apps via API
    # For now, guide user through manual creation
    echo "Please create GitHub Apps manually for now."
    echo "Visit: https://github.com/settings/apps/new"
}

# Main installation
main() {
    echo "╔══════════════════════════════════════════════════════╗"
    echo "║     Multi-Agent Development Platform Installer       ║"
    echo "╚══════════════════════════════════════════════════════╝"
    echo ""
    
    check_prerequisites
    install_k3s
    create_github_apps
    
    log_info "Installation complete!"
}

main "$@"
```

## Technical Implementation Steps

### Step 1: Create Abstraction Layer (Week 1)

#### 1.1 Replace Hardcoded Values

**File: `kubernetes/controller/values.yaml`**
```yaml
# BEFORE (Hardcoded)
image:
  repository: ghcr.io/5dlabs/multi-agent-controller
  
github:
  app_id: "1723711"
  client_id: "Iv23liXbJaNAQELWXIYD"

# AFTER (Templated)
image:
  repository: "{{ .Values.registry.url }}/{{ .Values.registry.namespace }}/multi-agent-controller"
  
github:
  app_id: "{{ .Values.github.apps.rex.id }}"
  client_id: "{{ .Values.github.apps.rex.client_id }}"
```

#### 1.2 Create Configuration Schema

**File: `pkg/config/schema.go`**
```go
package config

type PlatformConfig struct {
    Version  string                 `yaml:"version"`
    Platform PlatformSettings       `yaml:"platform"`
    GitHub   GitHubIntegration      `yaml:"github"`
    Registry RegistryConfig         `yaml:"registry"`
    Agents   map[string]AgentConfig `yaml:"agents"`
    Features FeatureFlags           `yaml:"features"`
}

type PlatformSettings struct {
    Name      string `yaml:"name"`
    Domain    string `yaml:"domain"`
    Namespace string `yaml:"namespace"`
}

type GitHubIntegration struct {
    Organization string              `yaml:"organization"`
    Repository   string              `yaml:"repository"`
    Apps         map[string]AppConfig `yaml:"apps"`
}

type RegistryConfig struct {
    Type      string `yaml:"type"` // dockerhub, ghcr, ecr, local
    URL       string `yaml:"url"`
    Namespace string `yaml:"namespace"`
    Username  string `yaml:"username,omitempty"`
    Password  string `yaml:"password,omitempty"`
}
```

### Step 2: Build CLI Tool (Week 2)

#### 2.1 CLI Structure

```go
// cmd/agent-platform/main.go
package main

import (
    "github.com/spf13/cobra"
    "github.com/charmbracelet/bubbletea"
)

var rootCmd = &cobra.Command{
    Use:   "agent-platform",
    Short: "Multi-Agent Development Platform CLI",
}

func init() {
    rootCmd.AddCommand(
        installCmd,
        configureCmd,
        statusCmd,
        taskCmd,
        upgradeCmd,
    )
}

var installCmd = &cobra.Command{
    Use:   "install",
    Short: "Install the platform",
    Run: func(cmd *cobra.Command, args []string) {
        installer := NewInteractiveInstaller()
        if err := installer.Run(); err != nil {
            log.Fatal(err)
        }
    },
}
```

#### 2.2 Interactive Installer

```go
// pkg/installer/wizard.go
package installer

import (
    "fmt"
    tea "github.com/charmbracelet/bubbletea"
)

type InstallWizard struct {
    step     int
    profile  string
    config   PlatformConfig
    progress []Step
}

type Step struct {
    Name     string
    Status   Status
    Progress float64
}

func (w *InstallWizard) Init() tea.Cmd {
    return w.checkPrerequisites
}

func (w *InstallWizard) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
    switch msg := msg.(type) {
    case prerequisitesChecked:
        w.step++
        return w, w.selectProfile
    case profileSelected:
        w.profile = msg.profile
        w.step++
        return w, w.installKubernetes
    }
    return w, nil
}
```

### Step 3: Kubernetes Abstraction (Week 3)

#### 3.1 Cluster Provisioners

```go
// pkg/cluster/provisioner.go
package cluster

type ClusterProvisioner interface {
    Install(config ClusterConfig) error
    Validate() error
    Destroy() error
}

type K3sProvisioner struct {
    config K3sConfig
}

func (p *K3sProvisioner) Install(config ClusterConfig) error {
    script := fmt.Sprintf(`
        curl -sfL https://get.k3s.io | \
        INSTALL_K3S_VERSION=%s sh -s - \
        --disable traefik \
        --write-kubeconfig-mode 644
    `, config.Version)
    
    return exec.Command("sh", "-c", script).Run()
}

type KindProvisioner struct {
    config KindConfig
}

func (p *KindProvisioner) Install(config ClusterConfig) error {
    kindConfig := `
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
  - containerPort: 443
    hostPort: 443`
    
    return kind.CreateCluster(kindConfig)
}
```

#### 3.2 Component Installer

```go
// pkg/components/installer.go
package components

type ComponentInstaller struct {
    kubeClient kubernetes.Interface
    helmClient helm.Interface
}

func (i *ComponentInstaller) InstallCore(config PlatformConfig) error {
    components := []Component{
        NewArgoCD(config),
        NewArgoWorkflows(config),
        NewArgoEvents(config),
        NewController(config),
    }
    
    for _, comp := range components {
        if err := i.installComponent(comp); err != nil {
            return fmt.Errorf("failed to install %s: %w", comp.Name(), err)
        }
    }
    return nil
}

func (i *ComponentInstaller) InstallOptional(features FeatureFlags) error {
    if features.Monitoring {
        if err := i.installComponent(NewMonitoringStack()); err != nil {
            return err
        }
    }
    
    if features.Databases {
        components := []Component{
            NewPostgreSQLOperator(),
            NewRedisOperator(),
        }
        for _, comp := range components {
            if err := i.installComponent(comp); err != nil {
                return err
            }
        }
    }
    
    return nil
}
```

### Step 4: GitHub Automation (Week 4)

#### 4.1 GitHub App Factory

```go
// pkg/github/app_factory.go
package github

import (
    "github.com/google/go-github/v50/github"
)

type AppFactory struct {
    client *github.Client
    org    string
}

func (f *AppFactory) CreateAgentApps() (map[string]*GitHubApp, error) {
    agents := []string{"rex", "cleo", "tess"}
    apps := make(map[string]*GitHubApp)
    
    for _, agent := range agents {
        app, err := f.createApp(agent)
        if err != nil {
            return nil, err
        }
        apps[agent] = app
    }
    
    return apps, nil
}

func (f *AppFactory) createApp(agentName string) (*GitHubApp, error) {
    manifest := &github.AppManifest{
        Name:        fmt.Sprintf("%s-%s-agent", f.org, agentName),
        URL:         fmt.Sprintf("https://github.com/%s", f.org),
        HookURL:     fmt.Sprintf("https://webhook.%s.local/%s", f.org, agentName),
        RedirectURL: "http://localhost:8080/callback",
        Description: fmt.Sprintf("%s agent for multi-agent platform", agentName),
        Public:      false,
        DefaultPermissions: &github.InstallationPermissions{
            Contents:     github.String("write"),
            PullRequests: github.String("write"),
            Issues:       github.String("write"),
            Metadata:     github.String("read"),
        },
        DefaultEvents: []string{
            "pull_request",
            "push",
            "issue_comment",
            "pull_request_review",
        },
    }
    
    // Use GitHub API to create app from manifest
    // This requires OAuth flow or manual creation
    return f.client.Apps.CreateFromManifest(manifest)
}
```

#### 4.2 Webhook Router

```go
// pkg/webhook/router.go
package webhook

type WebhookRouter struct {
    tunnelProvider TunnelProvider
    localPort      int
}

func (r *WebhookRouter) SetupRouting(apps map[string]*GitHubApp) error {
    // For local development, use tunnel provider
    if r.tunnelProvider != nil {
        url, err := r.tunnelProvider.CreateTunnel(r.localPort)
        if err != nil {
            return err
        }
        
        for agent, app := range apps {
            webhookURL := fmt.Sprintf("%s/webhook/%s", url, agent)
            if err := app.UpdateWebhookURL(webhookURL); err != nil {
                return err
            }
        }
    }
    
    return nil
}

// Tunnel providers for webhook access
type LocalTunnelProvider struct{}

func (p *LocalTunnelProvider) CreateTunnel(port int) (string, error) {
    cmd := exec.Command("lt", "--port", strconv.Itoa(port))
    output, err := cmd.Output()
    if err != nil {
        return "", err
    }
    return extractURL(output), nil
}
```

### Step 5: Validation System (Week 5)

#### 5.1 Health Checks

```go
// pkg/validation/health.go
package validation

type HealthChecker struct {
    kubeClient kubernetes.Interface
    config     PlatformConfig
}

func (h *HealthChecker) RunAllChecks() (*ValidationReport, error) {
    checks := []Check{
        h.checkKubernetesConnectivity,
        h.checkCoreComponents,
        h.checkGitHubIntegration,
        h.checkWorkflowEngine,
        h.checkEventProcessing,
        h.checkAgentReadiness,
    }
    
    report := &ValidationReport{
        Timestamp: time.Now(),
        Checks:    []CheckResult{},
    }
    
    for _, check := range checks {
        result := check()
        report.Checks = append(report.Checks, result)
        if !result.Passed {
            report.Failed++
        }
    }
    
    return report, nil
}

func (h *HealthChecker) checkWorkflowEngine() CheckResult {
    // Create test workflow
    testWorkflow := &v1alpha1.Workflow{
        ObjectMeta: metav1.ObjectMeta{
            Name: "validation-test",
        },
        Spec: v1alpha1.WorkflowSpec{
            EntryPoint: "echo",
            Templates: []v1alpha1.Template{
                {
                    Name: "echo",
                    Container: &corev1.Container{
                        Image:   "alpine:latest",
                        Command: []string{"echo", "test"},
                    },
                },
            },
        },
    }
    
    // Submit and wait for completion
    submitted, err := h.submitWorkflow(testWorkflow)
    if err != nil {
        return CheckResult{
            Name:   "Workflow Engine",
            Passed: false,
            Error:  err.Error(),
        }
    }
    
    return CheckResult{
        Name:   "Workflow Engine",
        Passed: true,
        Detail: fmt.Sprintf("Workflow %s completed", submitted.Name),
    }
}
```

#### 5.2 End-to-End Test

```go
// pkg/validation/e2e.go
package validation

func (v *Validator) RunE2ETest() error {
    // 1. Create test task via API
    task := &Task{
        Name:        "e2e-test",
        Description: "Write hello world function",
        Agent:       "rex",
    }
    
    if err := v.createTask(task); err != nil {
        return fmt.Errorf("failed to create task: %w", err)
    }
    
    // 2. Wait for Rex to create PR
    pr, err := v.waitForPR(task.Name, 5*time.Minute)
    if err != nil {
        return fmt.Errorf("Rex didn't create PR: %w", err)
    }
    
    // 3. Verify Cleo activation
    if err := v.waitForAgent("cleo", pr.Number, 2*time.Minute); err != nil {
        return fmt.Errorf("Cleo didn't activate: %w", err)
    }
    
    // 4. Verify Tess activation  
    if err := v.waitForAgent("tess", pr.Number, 2*time.Minute); err != nil {
        return fmt.Errorf("Tess didn't activate: %w", err)
    }
    
    // 5. Clean up
    return v.cleanup(task, pr)
}
```

### Step 6: Package Distribution (Week 6)

#### 6.1 Build Script

```bash
#!/bin/bash
# build-release.sh

VERSION=${1:-"0.1.0"}
PLATFORMS=("darwin/amd64" "darwin/arm64" "linux/amd64" "linux/arm64" "windows/amd64")

for PLATFORM in "${PLATFORMS[@]}"; do
    GOOS=${PLATFORM%/*}
    GOARCH=${PLATFORM#*/}
    
    echo "Building for $GOOS/$GOARCH..."
    
    env GOOS=$GOOS GOARCH=$GOARCH go build \
        -ldflags "-X main.Version=$VERSION" \
        -o "dist/agent-platform-${GOOS}-${GOARCH}" \
        ./cmd/agent-platform
    
    # Package with assets
    tar -czf "dist/agent-platform-${VERSION}-${GOOS}-${GOARCH}.tar.gz" \
        -C dist "agent-platform-${GOOS}-${GOARCH}" \
        -C .. templates/ scripts/ config/
done
```

#### 6.2 Homebrew Formula

```ruby
# Formula/agent-platform.rb
class AgentPlatform < Formula
  desc "Multi-Agent Software Development Orchestration Platform"
  homepage "https://github.com/yourusername/agent-platform"
  version "0.1.0"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/yourusername/agent-platform/releases/download/v#{version}/agent-platform-#{version}-darwin-arm64.tar.gz"
    sha256 "xxx"
  elsif OS.mac?
    url "https://github.com/yourusername/agent-platform/releases/download/v#{version}/agent-platform-#{version}-darwin-amd64.tar.gz"
    sha256 "xxx"
  elsif OS.linux?
    url "https://github.com/yourusername/agent-platform/releases/download/v#{version}/agent-platform-#{version}-linux-amd64.tar.gz"
    sha256 "xxx"
  end
  
  depends_on "kubectl"
  depends_on "helm"
  
  def install
    bin.install "agent-platform"
    (share/"agent-platform").install "templates", "scripts", "config"
  end
  
  def post_install
    (var/"agent-platform").mkpath
    (etc/"agent-platform").mkpath
  end
  
  test do
    assert_match version.to_s, shell_output("#{bin}/agent-platform version")
  end
end
```

## Configuration File Examples

### Minimal Config
```yaml
# ~/.agent-platform/config.yaml
version: "0.1.0"
profile: minimal

platform:
  name: my-dev-platform
  namespace: agent-platform

github:
  organization: myusername
  repository: test-repo

registry:
  type: local
  
agents:
  rex:
    enabled: true
    
features:
  monitoring: false
  databases: false
```

### Production Config
```yaml
# ~/.agent-platform/config.yaml
version: "0.1.0"
profile: production

platform:
  name: enterprise-platform
  domain: agents.company.com
  namespace: agent-platform

github:
  organization: company-org
  repository: platform-test
  apps:
    rex:
      id: "${REX_APP_ID}"
      client_id: "${REX_CLIENT_ID}"
      private_key_path: "/secrets/rex.pem"
    cleo:
      id: "${CLEO_APP_ID}"
      client_id: "${CLEO_CLIENT_ID}"
      private_key_path: "/secrets/cleo.pem"
    tess:
      id: "${TESS_APP_ID}"
      client_id: "${TESS_CLIENT_ID}"
      private_key_path: "/secrets/tess.pem"

registry:
  type: ecr
  url: xxx.dkr.ecr.us-west-2.amazonaws.com
  namespace: company

agents:
  rex:
    enabled: true
    resources:
      cpu: 4
      memory: 8Gi
  cleo:
    enabled: true
    resources:
      cpu: 2
      memory: 4Gi
  tess:
    enabled: true
    resources:
      cpu: 2
      memory: 4Gi
    features:
      kubernetes_testing: true

features:
  monitoring: true
  databases: true
  mail_notifications: true
  backup: true
  high_availability: true
```

## Testing Matrix

| Component | Local (k3s) | Kind | Minikube | EKS | GKE | AKS |
|-----------|-------------|------|----------|-----|-----|-----|
| Core Platform | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| GitHub Integration | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Local Tunnels | ✓ | ✓ | ✓ | - | - | - |
| Cloud Webhooks | - | - | - | ✓ | ✓ | ✓ |
| Monitoring Stack | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Database Operators | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Migration Scripts

### From 5D Labs to Generic

```bash
#!/bin/bash
# migrate-from-5dlabs.sh

# Replace all 5dlabs references
find kubernetes/ -type f -name "*.yaml" -exec sed -i \
    's/ghcr.io\/5dlabs/{{ .Values.registry.url }}\/{{ .Values.registry.namespace }}/g' {} \;

# Replace hardcoded GitHub Apps
sed -i 's/"1723711"/"{{ .Values.github.apps.rex.id }}"/g' kubernetes/controller/values.yaml

# Remove organization-specific components
rm -f kubernetes/applications/mail-server.yaml
rm -f kubernetes/applications/twingate-*.yaml
rm -f kubernetes/sendgrid-dns.yaml

# Create template structure
mkdir -p templates/{core,optional,configs}
mv kubernetes/controller templates/core/
mv kubernetes/monitoring templates/optional/
```

## Success Metrics

- **Installation Time**: < 5 minutes for minimal, < 15 minutes for production
- **Configuration Steps**: < 10 interactive prompts
- **Success Rate**: > 95% on supported platforms
- **Error Recovery**: Automatic rollback on failure
- **Documentation Coverage**: 100% of user-facing features

## Release Checklist

- [ ] All hardcoded values replaced with templates
- [ ] CLI tool tested on all target platforms
- [ ] GitHub App automation working
- [ ] Installation wizard validated by non-developers
- [ ] Documentation complete and reviewed
- [ ] E2E tests passing on all Kubernetes distributions
- [ ] Homebrew formula tested
- [ ] Docker images published to public registry
- [ ] Release notes prepared
- [ ] Demo video recorded