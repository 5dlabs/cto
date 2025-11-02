# CTO CLI - Platform Installer

Beautiful, interactive CLI for installing and managing the CTO multi-agent development platform.

## Features

- 🎨 **Beautiful UI** - Interactive prompts with progress indicators
- 🚀 **Quick Installation** - Get started in minutes
- 📦 **Multiple Profiles** - Minimal, Standard, and Production configurations
- 🔧 **Flexible Deployment** - Local kind cluster or remote Kubernetes
- ⚙️ **Auto-Configuration** - Generates `cto-config.json` automatically

## Installation Profiles

### Minimal (8GB RAM)
- Core components only (ArgoCD, Argo Workflows, Controller)
- Perfect for local development
- Fast installation (~5-10 minutes)

### Standard (16GB RAM)
- Full monitoring stack (Grafana, VictoriaMetrics)
- Database operators (PostgreSQL, Redis, QuestDB)
- Team development ready

### Production (32GB RAM)
- High availability setup
- Enterprise security features
- Backup and disaster recovery

## Usage

### Interactive Installation

```bash
cargo run -p cto-cli -- install
```

The CLI will guide you through:
1. Selecting an installation profile
2. Choosing local (kind) or remote cluster
3. Configuring GitHub integration (optional)
4. Setting up container registry
5. Selecting optional features

### Non-Interactive Installation

```bash
# Minimal local installation
cargo run -p cto-cli -- install --profile minimal --local --non-interactive

# Standard on remote cluster
cargo run -p cto-cli -- install --profile standard --remote
```

### Using a Config File

```bash
cargo run -p cto-cli -- install --config cto-install.yaml
```

## What Gets Installed

### Core Components (All Profiles)
- ✅ ArgoCD - GitOps deployment
- ✅ Argo Workflows - Workflow orchestration
- ✅ Argo Events - Event-driven automation
- ✅ CTO Controller - Multi-agent orchestration

### Optional Components
- 📊 **Monitoring Stack** - Grafana, VictoriaMetrics, VictoriaLogs
- 🗄️ **Database Operators** - PostgreSQL, Redis, QuestDB
- 🔐 **Security Features** - Backup operators, secret management

### Generated Files
- `cto-config.json` - Platform configuration with agent defaults
- `.kube/config` - Kubernetes configuration (if using kind)

## Prerequisites

The CLI automatically checks for:
- ✅ Docker
- ✅ kubectl
- ✅ Helm
- ✅ kind (for local installations)
- ✅ Sufficient system resources (8GB+ RAM)

## Post-Installation

After installation completes, you'll get instructions for:

1. **Accessing ArgoCD**
   ```bash
   kubectl port-forward svc/argocd-server -n argocd 8080:443
   # Visit https://localhost:8080
   ```

2. **Accessing Argo Workflows**
   ```bash
   kubectl port-forward svc/argo-workflows-server -n argo 2746:2746
   # Visit http://localhost:2746
   ```

3. **Configuring GitHub Apps**
   - Create GitHub Apps for your agents (Rex, Cleo, Tess, etc.)
   - Update `cto-config.json` with app IDs
   - Add secrets to Kubernetes

## Development

```bash
# Build
cargo build -p cto-cli

# Run
cargo run -p cto-cli -- --help

# Release build
cargo build --release -p cto-cli
```

## Architecture

```
cli/
├── src/
│   ├── main.rs              # CLI entry point & banner
│   ├── commands/
│   │   ├── mod.rs
│   │   └── install.rs       # Installation command
│   ├── config.rs            # Configuration data structures
│   ├── installer/
│   │   ├── mod.rs           # Main installer orchestration
│   │   ├── cluster.rs       # Kubernetes cluster provisioning
│   │   ├── components.rs    # Component installation (Argo, etc.)
│   │   └── config_generator.rs # cto-config.json generation
│   ├── ui.rs                # Beautiful UI helpers
│   └── validator.rs         # Prerequisite validation
└── Cargo.toml
```

## Future Commands

Coming soon:
- `cto status` - Show platform status
- `cto upgrade` - Upgrade platform components
- `cto uninstall` - Clean removal
- `cto validate` - Validate installation

## License

AGPL-3.0 - See LICENSE file for details

