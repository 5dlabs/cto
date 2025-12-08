# CTO CLI Quickstart Guide

## What We Built

A beautiful, production-ready CLI installer for the CTO platform that:

- ✨ **Interactive** - Guides you through installation with beautiful prompts
- 🎨 **Pretty** - Colored output, progress indicators, ASCII art banner
- 🚀 **Fast** - Gets you from zero to running platform in minutes
- 🔧 **Flexible** - Supports local development or production deployments
- 📦 **Complete** - Installs all dependencies and generates config files

## Try It Out

### 1. Run the Installer

```bash
cd /Users/jonathonfritz/code/work-projects/5dlabs/cto
cargo run -p cto-cli -- install
```

You'll see:
```
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║   ██████╗████████╗ ██████╗     ██████╗ ██╗      █████╗  ║
    ║  ██╔════╝╚══██╔══╝██╔═══██╗    ██╔══██╗██║     ██╔══██╗ ║
    ║  ██║        ██║   ██║   ██║    ██████╔╝██║     ███████║ ║
    ║  ██║        ██║   ██║   ██║    ██╔═══╝ ██║     ██╔══██║ ║
    ║  ╚██████╗   ██║   ╚██████╔╝    ██║     ███████╗██║  ██║ ║
    ║   ╚═════╝   ╚═╝    ╚═════╝     ╚═╝     ╚══════╝╚═╝  ╚═╝ ║
    ║                                                           ║
    ║         Multi-Agent Development Orchestration            ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝

🚀 CTO Platform Installation
══════════════════════════════════════════════════════════════════
```

### 2. Choose Your Profile

The installer will ask you to select:

**Minimal** (Recommended for getting started)
- 8GB RAM requirement
- Core components only
- Perfect for local development
- Fast installation (~5 minutes)

**Standard** (Team development)
- 16GB RAM requirement
- Full monitoring stack
- Database operators
- Production-ready features

**Production** (Enterprise)
- 32GB RAM requirement
- High availability
- Enterprise security
- Full feature set

### 3. Select Cluster Type

**Local kind cluster** (Recommended)
- Creates a local Kubernetes cluster using kind
- Port forwards: ArgoCD (8080), Argo Workflows (2746)
- Completely isolated from other clusters
- Perfect for development

**Remote Kubernetes cluster**
- Uses your existing kubeconfig
- Assumes you already have a cluster running
- Good for team or production installations

### 4. Configure (Optional)

The CLI will ask about:
- GitHub integration (can configure later)
- Container registry (defaults to ghcr.io)
- Custom domain (production only)
- Monitoring stack (standard/production)
- Database operators (standard/production)

### 5. Installation Happens Automatically

The CLI will:
1. ✅ Validate prerequisites (Docker, kubectl, Helm, kind)
2. 🎯 Create kind cluster (if local)
3. 📦 Install ArgoCD
4. 📦 Install Argo Workflows
5. 📦 Install Argo Events
6. 📦 Install CTO Controller
7. 📊 Install monitoring (if selected)
8. 🗄️ Install database operators (if selected)
9. 🔧 Build CTO MCP binary
10. 📝 Generate cto-config.json

### 6. Access Your Platform

After installation, you'll get instructions like:

```bash
# Access ArgoCD
kubectl port-forward svc/argocd-server -n argocd 8080:443
# Visit: https://localhost:8080
# Username: admin
# Password: kubectl -n argocd get secret argocd-initial-admin-secret \
#            -o jsonpath="{.data.password}" | base64 -d

# Access Argo Workflows
kubectl port-forward svc/argo-workflows-server -n argo 2746:2746
# Visit: http://localhost:2746
```

## Quick Install (Non-Interactive)

For CI/CD or scripted installations:

```bash
# Minimal local install
cargo run -p cto-cli -- install \
  --profile minimal \
  --local \
  --non-interactive

# Standard with monitoring
cargo run -p cto-cli -- install \
  --profile standard \
  --local
```

## What Gets Created

After installation:

```
cto-platform/                    # kind cluster
├── Namespaces:
│   ├── cto          # CTO components
│   ├── argocd                  # GitOps
│   ├── argo                    # Workflows & Events
│   ├── observability (optional) # Prometheus, Loki, Alertmanager, Grafana
│   └── *-operator (optional)   # Database operators
│
└── cto-config.json             # Generated configuration
```

## Next Steps After Installation

1. **Configure GitHub Apps**
   - Create apps for Rex, Cleo, Tess, Blaze, Cipher, Morgan
   - Update `cto-config.json` with app IDs
   - Add secrets to Kubernetes

2. **Deploy Your First Workflow**
   ```bash
   # Using the CTO MCP tool
   cto play --task-id 1
   ```

3. **Monitor Your Platform**
   - Check ArgoCD for deployment status
   - View workflows in Argo Workflows UI
   - Monitor logs and metrics (if monitoring enabled)

## Development Features

The CLI is built with:
- **clap** - Modern CLI framework
- **dialoguer** - Beautiful interactive prompts
- **indicatif** - Progress bars and spinners
- **colored** - Colorful terminal output
- **tokio** - Async runtime for concurrent operations

## Architecture

```
CLI Flow:
┌─────────────────┐
│ Prerequisites   │ Check Docker, kubectl, Helm, kind, RAM
├─────────────────┤
│ Configuration   │ Interactive prompts or config file
├─────────────────┤
│ Cluster Setup   │ Create kind cluster or validate remote
├─────────────────┤
│ Core Install    │ ArgoCD → Workflows → Events → Controller
├─────────────────┤
│ Optional        │ Monitoring, Databases (if selected)
├─────────────────┤
│ Binary Build    │ Build CTO MCP from source
├─────────────────┤
│ Config Gen      │ Generate cto-config.json
└─────────────────┘
```

## Troubleshooting

### "Docker not found"
```bash
# Install Docker Desktop
brew install --cask docker
```

### "kind cluster creation failed"
```bash
# Delete existing cluster
kind delete cluster --name cto-platform

# Try again
cargo run -p cto-cli -- install
```

### "Insufficient memory"
The CLI checks your available RAM. For minimal installation, you need at least 8GB total system RAM.

### "kubectl cannot connect"
```bash
# For kind cluster
kind get kubeconfig --name cto-platform > ~/.kube/cto-config
export KUBECONFIG=~/.kube/cto-config

# Verify
kubectl cluster-info
```

## Feedback

This is the first version of the CLI installer! Feedback welcome:
- What features are missing?
- What's confusing?
- What would make it better?

