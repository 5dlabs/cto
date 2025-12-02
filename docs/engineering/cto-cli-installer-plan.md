# CTO CLI Installer - Product Requirements Document

## Executive Summary

Create a beautiful, modern TUI (Terminal User Interface) installer for the CTO Platform that provides a delightful onboarding experience. The installer will feature animated agent avatars, sleek styling, and guide users through installing the complete CTO platform on their infrastructure.

## Vision

> "Install an AI development team in under 10 minutes"

The CTO CLI installer should feel like meeting your new AI team members for the first time. Each agent introduces themselves with personality, and the installation process feels like assembling a world-class engineering organization.

---

## Agent Roster & Personalities

### The CTO AI Team

| Agent | Role | Personality | ASCII Art Theme |
|-------|------|-------------|-----------------|
| **Rex** | Lead Developer | Confident, methodical, ships code | 🦖 T-Rex with keyboard |
| **Cleo** | Code Reviewer | Sharp-eyed, quality-focused | 🔍 Detective with magnifying glass |
| **Blaze** | Frontend Dev | Creative, fast, stylish | 🔥 Flame with paintbrush |
| **Tess** | QA Engineer | Thorough, detail-oriented | 🧪 Scientist with test tubes |
| **Cipher** | Security Expert | Vigilant, cryptic, protective | 🔐 Shield with lock |
| **Morgan** | Documentation | Articulate, organized, helpful | 📚 Owl with book |
| **Atlas** | Infrastructure | Powerful, reliable, scalable | 🗺️ Globe with servers |
| **Bolt** | DevOps/Deploy | Fast, automated, efficient | ⚡ Lightning bolt |
| **Stitch** | PR Review Bot | Meticulous, constructive | 🧵 Needle with thread |

---

## TUI Design Specification

### Color Palette

```text
Primary:    #6366F1 (Indigo-500) - Main accent
Secondary:  #8B5CF6 (Violet-500) - Highlights
Success:    #10B981 (Emerald-500) - Checkmarks
Warning:    #F59E0B (Amber-500) - Cautions
Error:      #EF4444 (Red-500) - Failures
Background: #0F172A (Slate-900) - Dark mode
Surface:    #1E293B (Slate-800) - Cards
Text:       #F8FAFC (Slate-50) - Primary text
Muted:      #64748B (Slate-500) - Secondary text
```

### Typography

- Headers: Bold, uppercase where appropriate
- Body: Regular weight, clear spacing
- Code: Monospace, slightly dimmed background
- Status: Color-coded with icons

### Layout Structure

```text
┌─────────────────────────────────────────────────────────────────────────┐
│  ██████╗████████╗ ██████╗     ██████╗ ██╗      █████╗ ████████╗███████╗ │
│ ██╔════╝╚══██╔══╝██╔═══██╗    ██╔══██╗██║     ██╔══██╗╚══██╔══╝██╔════╝ │
│ ██║        ██║   ██║   ██║    ██████╔╝██║     ███████║   ██║   █████╗   │
│ ██║        ██║   ██║   ██║    ██╔═══╝ ██║     ██╔══██║   ██║   ██╔══╝   │
│ ╚██████╗   ██║   ╚██████╔╝    ██║     ███████╗██║  ██║   ██║   ███████╗ │
│  ╚═════╝   ╚═╝    ╚═════╝     ╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝ │
│                                                                         │
│                    Multi-Agent Development Platform                     │
│                         v0.12.0 • 5D Labs                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  [Content Area - Dynamic based on current step]                        │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  [Progress Bar]  Step 3/8: Installing Core Components                  │
│  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  37%  │
├─────────────────────────────────────────────────────────────────────────┤
│  [↑/↓] Navigate  [Enter] Select  [Esc] Back  [q] Quit  [?] Help        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Installation Flows

### Flow 1: Welcome Screen

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                          Welcome to CTO Platform                        │
│                                                                         │
│     ┌─────────────────────────────────────────────────────────────┐    │
│     │                                                             │    │
│     │    "Hi! I'm Rex, your lead developer.                      │    │
│     │     Let me introduce you to the team..."                   │    │
│     │                                                             │    │
│     │         🦖                                                  │    │
│     │        /|  |\                                               │    │
│     │       / |  | \    ⌨️                                        │    │
│     │      /  |__|  \                                             │    │
│     │                                                             │    │
│     └─────────────────────────────────────────────────────────────┘    │
│                                                                         │
│     What would you like to do?                                         │
│                                                                         │
│     ▸ 🚀  Install CTO Platform                                         │
│       🔧  Install CLI Only                                             │
│       📊  Check System Requirements                                    │
│       📖  View Documentation                                           │
│       ❌  Exit                                                          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Flow 2: Cluster Selection

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                         Choose Your Environment                         │
│                                                                         │
│     ┌─────────────────────────────────────────────────────────────┐    │
│     │    "I'll help you set up the infrastructure!"              │    │
│     │                                                             │    │
│     │              🗺️                                             │    │
│     │             /   \      Atlas                                │    │
│     │            |  ◉  |     Infrastructure Agent                 │    │
│     │             \___/                                           │    │
│     └─────────────────────────────────────────────────────────────┘    │
│                                                                         │
│     Where should we deploy?                                            │
│                                                                         │
│     ▸ 🏠  Local Development (Kind)                                     │
│           Perfect for trying out CTO on your machine                   │
│           Requirements: Docker, 8GB RAM, 4 CPU cores                   │
│                                                                         │
│       ☁️   Existing Kubernetes Cluster                                  │
│           Deploy to your existing cluster                              │
│           Requirements: kubectl configured, cluster-admin access       │
│                                                                         │
│       🔧  Bare Metal (Talos) [Coming Soon]                             │
│           Full production setup with Talos Linux                       │
│           Requirements: Dedicated servers, network access              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Flow 3: Component Selection

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                        Select Components to Install                     │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ CORE PLATFORM (Required)                                          │ │
│  │ ───────────────────────────────────────────────────────────────── │ │
│  │ [✓] ArgoCD              GitOps continuous delivery                │ │
│  │ [✓] Argo Workflows      Workflow orchestration                    │ │
│  │ [✓] Argo Events         Event-driven automation                   │ │
│  │ [✓] CTO Controller      Agent orchestration                       │ │
│  │ [✓] CTO Tools           MCP tool server                           │ │
│  │ [✓] OpenMemory          AI agent memory system                    │ │
│  │ [✓] Heal                Self-healing monitor                      │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ INFRASTRUCTURE (Recommended)                                      │ │
│  │ ───────────────────────────────────────────────────────────────── │ │
│  │ [✓] Vault               Secrets management                        │ │
│  │ [✓] Cert-Manager        TLS certificates                          │ │
│  │ [✓] Ingress NGINX       Load balancing                            │ │
│  │ [ ] External DNS        DNS automation                            │ │
│  │ [ ] Cloudflare Tunnel   Secure public access                      │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ OBSERVABILITY (Optional)                                          │ │
│  │ ───────────────────────────────────────────────────────────────── │ │
│  │ [ ] Grafana             Dashboards & visualization                │ │
│  │ [ ] VictoriaMetrics     Metrics storage                           │ │
│  │ [ ] VictoriaLogs        Log aggregation                           │ │
│  │ [ ] OTEL Collector      Distributed tracing                       │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  [Space] Toggle  [a] Select All  [n] Select None  [Enter] Continue    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Flow 4: Installation Progress

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                          Installing CTO Platform                        │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                 │   │
│  │   ⚡ Bolt is deploying your infrastructure...                  │   │
│  │                                                                 │   │
│  │        _____                                                    │   │
│  │       /     \                                                   │   │
│  │      |  ⚡   |   "Deploying at lightning speed!"               │   │
│  │       \_____/                                                   │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Current: Installing Argo Workflows                                    │
│                                                                         │
│  ✓ Kubernetes cluster ready                              [00:12]       │
│  ✓ ArgoCD installed                                      [01:23]       │
│  ◉ Argo Workflows installing...                          [00:45]       │
│  ○ Argo Events                                                         │
│  ○ Vault                                                               │
│  ○ Cert-Manager                                                        │
│  ○ CTO Platform                                                        │
│                                                                         │
│  ████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  42%   │
│                                                                         │
│  Elapsed: 02:20  |  Estimated remaining: 03:15                         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Flow 5: Meet Your Team

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                        🎉 Installation Complete!                        │
│                                                                         │
│                        Meet Your AI Development Team                    │
│                                                                         │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │
│  │  🦖     │ │  🔍     │ │  🔥     │ │  🧪     │ │  🔐     │           │
│  │  Rex    │ │  Cleo   │ │  Blaze  │ │  Tess   │ │  Cipher │           │
│  │  Dev    │ │  Review │ │  UI/UX  │ │  QA     │ │  Sec    │           │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘           │
│                                                                         │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                       │
│  │  📚     │ │  🗺️     │ │  ⚡     │ │  🧵     │                       │
│  │  Morgan │ │  Atlas  │ │  Bolt   │ │  Stitch │                       │
│  │  Docs   │ │  Infra  │ │  Deploy │ │  PR Bot │                       │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘                       │
│                                                                         │
│  Your team is ready! Here's how to get started:                        │
│                                                                         │
│  1. Access ArgoCD:     https://localhost:8080                          │
│  2. View Workflows:    https://localhost:2746                          │
│  3. Configure GitHub:  cto configure github                            │
│  4. Start your first task:  cto task create "Build login page"        │
│                                                                         │
│  [Enter] Open Dashboard  [c] Copy Commands  [q] Exit                   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Technical Architecture

### Rust TUI Stack

```toml
[dependencies]
# TUI Framework
ratatui = "0.29"           # Modern TUI framework (successor to tui-rs)
crossterm = "0.28"          # Terminal manipulation

# Async Runtime
tokio = { version = "1", features = ["full"] }

# CLI Framework
clap = { version = "4", features = ["derive"] }

# Styling
tui-big-text = "0.7"        # ASCII art text
tui-scrollview = "0.5"      # Scrollable content

# Progress & Animation
indicatif = "0.17"          # Progress bars
console = "0.15"            # Terminal colors

# Configuration
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"

# HTTP Client (for downloads)
reqwest = { version = "0.12", features = ["json", "stream"] }

# Process Execution
tokio-process = "0.2"
which = "7.0"

# System Info
sysinfo = "0.33"
```

### Module Structure

```text
cli/src/
├── main.rs                 # Entry point, CLI parsing
├── app.rs                  # Application state machine
├── tui/
│   ├── mod.rs
│   ├── app.rs              # TUI application loop
│   ├── ui.rs               # UI rendering
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── banner.rs       # ASCII art banner
│   │   ├── agent_card.rs   # Agent introduction cards
│   │   ├── progress.rs     # Installation progress
│   │   ├── menu.rs         # Selection menus
│   │   └── checklist.rs    # Component checklist
│   ├── screens/
│   │   ├── mod.rs
│   │   ├── welcome.rs      # Welcome screen
│   │   ├── cluster.rs      # Cluster selection
│   │   ├── components.rs   # Component selection
│   │   ├── install.rs      # Installation progress
│   │   ├── complete.rs     # Completion screen
│   │   └── error.rs        # Error handling
│   └── theme.rs            # Color palette & styling
├── installer/
│   ├── mod.rs
│   ├── cluster/
│   │   ├── mod.rs
│   │   ├── kind.rs         # Kind cluster setup
│   │   ├── remote.rs       # Remote cluster validation
│   │   └── talos.rs        # Talos bare metal (future)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── argocd.rs
│   │   ├── argo_workflows.rs
│   │   ├── argo_events.rs
│   │   ├── vault.rs
│   │   ├── cert_manager.rs
│   │   ├── ingress.rs
│   │   ├── monitoring.rs
│   │   └── cto.rs          # CTO umbrella chart
│   └── scripts/
│       ├── mod.rs
│       ├── kind_setup.sh   # Kind cluster creation
│       └── post_install.sh # Post-installation tasks
├── config/
│   ├── mod.rs
│   ├── install.rs          # Installation configuration
│   ├── cluster.rs          # Cluster configuration
│   └── components.rs       # Component configuration
├── agents/
│   ├── mod.rs
│   └── ascii_art.rs        # Agent ASCII art definitions
└── utils/
    ├── mod.rs
    ├── kubectl.rs          # kubectl wrapper
    ├── helm.rs             # helm wrapper
    └── system.rs           # System checks
```

---

## Component Installation Order

### Phase 1: Infrastructure Foundation

| Order | Component | Namespace | Dependencies | Install Method |
|-------|-----------|-----------|--------------|----------------|
| 1 | Kind Cluster | N/A | Docker | `kind create cluster` |
| 2 | Gateway API CRDs | N/A | Cluster | `kubectl apply` |
| 3 | Cert-Manager | cert-manager | Gateway API | Helm |
| 4 | Ingress NGINX | ingress-nginx | Cert-Manager | Helm |

### Phase 2: GitOps & Workflows

| Order | Component | Namespace | Dependencies | Install Method |
|-------|-----------|-----------|--------------|----------------|
| 5 | ArgoCD | argocd | Ingress | Helm |
| 6 | Argo Workflows | argo | ArgoCD | ArgoCD App |
| 7 | Argo Events | argo-events | Argo Workflows | ArgoCD App |

### Phase 3: Security & Secrets

| Order | Component | Namespace | Dependencies | Install Method |
|-------|-----------|-----------|--------------|----------------|
| 8 | Vault | vault | Argo Events | ArgoCD App |
| 9 | Vault Secrets Operator | vault | Vault | ArgoCD App |
| 10 | External Secrets | external-secrets | Vault | ArgoCD App |

### Phase 4: CTO Platform

| Order | Component | Namespace | Dependencies | Install Method |
|-------|-----------|-----------|--------------|----------------|
| 11 | CTO Umbrella Chart | cto, automation | All above | ArgoCD App |

### Phase 5: Observability (Optional)

| Order | Component | Namespace | Dependencies | Install Method |
|-------|-----------|-----------|--------------|----------------|
| 12 | VictoriaMetrics | observability | CTO | ArgoCD App |
| 13 | VictoriaLogs | observability | VictoriaMetrics | ArgoCD App |
| 14 | Grafana | observability | VictoriaMetrics | ArgoCD App |
| 15 | OTEL Collector | observability | Grafana | ArgoCD App |

---

## Kind Cluster Configuration

### kind-config.yaml

```yaml
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: cto-platform
nodes:
  - role: control-plane
    kubeadmConfigPatches:
      - |
        kind: InitConfiguration
        nodeRegistration:
          kubeletExtraArgs:
            node-labels: "ingress-ready=true"
    extraPortMappings:
      # HTTP
      - containerPort: 80
        hostPort: 80
        protocol: TCP
      # HTTPS
      - containerPort: 443
        hostPort: 443
        protocol: TCP
      # ArgoCD
      - containerPort: 30080
        hostPort: 8080
        protocol: TCP
      # Argo Workflows
      - containerPort: 30746
        hostPort: 2746
        protocol: TCP
      # Grafana
      - containerPort: 30300
        hostPort: 3000
        protocol: TCP
  - role: worker
  - role: worker
containerdConfigPatches:
  - |-
    [plugins."io.containerd.grpc.v1.cri".registry.mirrors."ghcr.io"]
      endpoint = ["https://ghcr.io"]
networking:
  podSubnet: "10.244.0.0/16"
  serviceSubnet: "10.96.0.0/12"
```

---

## CLI Commands

### Primary Commands

```bash
# Interactive TUI installer
cto install

# Non-interactive installation
cto install --profile minimal --cluster kind --yes

# Install CLI only (no cluster components)
cto install --cli-only

# Check system requirements
cto doctor

# Show installation status
cto status

# Upgrade components
cto upgrade [component]

# Uninstall
cto uninstall [--keep-data]
```

### Configuration Commands

```bash
# Configure GitHub integration
cto configure github

# Configure secrets
cto configure secrets

# Generate cto-config.json
cto configure generate

# Validate configuration
cto configure validate
```

### Agent Commands

```bash
# List agents
cto agents list

# Show agent status
cto agents status rex

# View agent logs
cto agents logs cleo --follow
```

---

## Installation Profiles

### Minimal (Local Development)

- **Target**: Single developer, local testing
- **Resources**: 8GB RAM, 4 CPU cores
- **Components**: Core platform only
- **Storage**: Local path provisioner
- **Networking**: NodePort services

### Standard (Team Development)

- **Target**: Small team, shared cluster
- **Resources**: 16GB RAM, 8 CPU cores
- **Components**: Core + Vault + Monitoring
- **Storage**: Local path or cloud provisioner
- **Networking**: Ingress with self-signed certs

### Production (Enterprise)

- **Target**: Production workloads
- **Resources**: 32GB+ RAM, 16+ CPU cores
- **Components**: Full stack with HA
- **Storage**: Cloud provisioner with backups
- **Networking**: Ingress with Let's Encrypt

---

## Error Handling & Recovery

### Pre-flight Checks

1. Docker running and accessible
2. Sufficient disk space (20GB+)
3. Sufficient memory (8GB+)
4. Required ports available (80, 443, 8080, 2746)
5. kubectl installed and accessible
6. helm installed (v3.12+)

### Recovery Actions

| Error | Recovery |
|-------|----------|
| Docker not running | Prompt to start Docker |
| Insufficient memory | Suggest minimal profile |
| Port conflict | Offer alternative ports |
| Cluster exists | Offer to reuse or recreate |
| Component failed | Retry with backoff, show logs |
| Network timeout | Retry with longer timeout |

---

## Success Metrics

### Installation Time Targets

| Profile | Target Time | Acceptable |
|---------|-------------|------------|
| Minimal | < 5 minutes | < 8 minutes |
| Standard | < 10 minutes | < 15 minutes |
| Production | < 20 minutes | < 30 minutes |

### User Experience Goals

- Zero manual kubectl commands during install
- Clear progress indication at all times
- Helpful error messages with solutions
- Easy recovery from failures
- Beautiful, memorable first impression

---

## Future Roadmap

### Phase 2: Talos Bare Metal

- Talos Linux installation
- Multi-node cluster setup
- Network configuration
- Storage provisioning

### Phase 3: Cloud Providers

- AWS EKS integration
- GCP GKE integration
- Azure AKS integration
- DigitalOcean integration

### Phase 4: Advanced Features

- Cluster federation
- Multi-region deployment
- Disaster recovery
- Automated backups

---

## Implementation Tasks

### Sprint 1: TUI Foundation (Week 1-2)

1. Set up ratatui-based TUI framework
2. Implement welcome screen with agent cards
3. Create navigation system
4. Design and implement color theme
5. Build ASCII art for all agents

### Sprint 2: Installation Flow (Week 3-4)

1. Implement cluster selection screen
2. Build component checklist UI
3. Create installation progress screen
4. Implement Kind cluster provisioning
5. Add remote cluster validation

### Sprint 3: Component Installation (Week 5-6)

1. Implement ArgoCD installation
2. Add Argo Workflows/Events installation
3. Implement Vault installation
4. Add CTO umbrella chart deployment
5. Implement monitoring stack (optional)

### Sprint 4: Polish & Testing (Week 7-8)

1. Add error handling and recovery
2. Implement configuration persistence
3. Add upgrade/uninstall commands
4. Create comprehensive tests
5. Documentation and examples

---

## Appendix: Agent ASCII Art

### Rex (Lead Developer)

```text
    ____
   /    \
  | 🦖  |  "Ready to ship code!"
  |  __  |
   \____/
    |  |
   /|  |\
  / |  | \
    ⌨️
```

### Cleo (Code Reviewer)

```text
    ____
   /    \
  | 🔍  |  "Let me take a closer look..."
  |  __  |
   \____/
    |  |
   /|  |\
  📋    📝
```

### Blaze (Frontend Dev)

```text
    ____
   /    \
  | 🔥  |  "Making it beautiful!"
  |  __  |
   \____/
    |  |
   🎨  🖌️
```

### Atlas (Infrastructure)

```text
    ____
   /    \
  | 🗺️  |  "I'll handle the infrastructure"
  |  __  |
   \____/
    |  |
   🖥️  ☁️
```

### Bolt (DevOps)

```text
    ____
   /    \
  | ⚡  |  "Deploying at lightning speed!"
  |  __  |
   \____/
    |  |
   🚀  📦
```

---

## References

- [Ratatui Documentation](https://ratatui.rs/)
- [Kind Documentation](https://kind.sigs.k8s.io/)
- [ArgoCD Getting Started](https://argo-cd.readthedocs.io/)
- [CTO Platform Documentation](https://github.com/5dlabs/cto)

