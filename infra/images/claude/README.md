

# Claude Code Docker Image

A containerized development environment with Claude Code CLI for AI-powered coding assistance.



## Features

- **Claude Code CLI**: AI-powered coding assistant from Anthropic
- **Development Tools**: Node.js 20, git, zsh, fzf, and more
- **Ready for Tasks**: Pre-configured for automated development workflows
- **Multi-platform**: Supports both AMD64 and ARM64 architectures



## Usage

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | ✅ | API key for Claude Code access |
| `TZ` | ❌ | Timezone setting (default: system timezone) |

### Building the Image





```bash


# Build locally
docker build -t claude-code:latest .

# Build with specific Claude version
docker build --build-arg CLAUDE_CODE_VERSION=1.2.3 -t claude-code:latest .








```

### Running Locally





```bash
# Interactive development environment
docker run -it \


  -e ANTHROPIC_API_KEY="your-api-key" \
  -v $(pwd):/workspace \
  claude-code:latest

# With timezone setting
docker run -it \


  -e ANTHROPIC_API_KEY="your-api-key" \


  -e TZ="America/New_York" \
  -v $(pwd):/workspace \
  claude-code:latest








```

### Kubernetes TaskRun Integration





```yaml
apiVersion: tekton.dev/v1beta1
kind: TaskRun
metadata:
  name: claude-development-task
spec:
  taskSpec:
    steps:
    - name: claude-code
      image: claude-code:latest
      env:
      - name: ANTHROPIC_API_KEY
        valueFrom:
          secretKeyRef:
            name: claude-secrets
            key: api-key
      script: |
        #!/bin/bash
        # Your development tasks here
        claude-code "Generate documentation for this project"








```

## Integration with Orchestrator

This image is used by the orchestrator platform for:

1. **Documentation Generation**: Automated project documentation
2. **Code Analysis**: Intelligent code review and suggestions
3. **Task Automation**: AI-powered development task execution
4. **Testing**: Automated test generation and execution



## API Key Setup

Get your Claude API key from [Anthropic](https://console.anthropic.com) and add it to your secrets:





```bash
kubectl create secret generic claude-secrets \


  --from-literal=api-key="your-claude-api-key"








```

## Included Tools

### Core Development Stack
- **Node.js 20** with npm, yarn, and pnpm
- **Python 3** with pip, venv, and development tools
- **Go** with latest stable version
- **Rust** with cargo, clippy, rustfmt, and rust-analyzer
- **PHP, Ruby, Perl** for multi-language development

### Version Control & Git
- **Git** with enhanced tools (git-lfs, git-extras, git-delta)
- **GitHub CLI (gh)** with Copilot and CodeQL extensions
- **Pre-configured git defaults** for development workflows

### Code Quality & Linting
- **ESLint, Prettier, TypeScript** for JavaScript/TypeScript
- **Black, isort, flake8, mypy, ruff** for Python
- **Clippy, rustfmt** for Rust
- **ShellCheck, Yamllint, Markdownlint** for shell and markup
- **SQLFluff** for SQL
- **Bandit, Safety** for security scanning

### Cloud & Infrastructure
- **AWS CLI** for cloud development
- **kubectl, helm, kustomize** for Kubernetes
- **Docker CLI and Docker Compose** for containers
- **Argo CD, Argo Workflows** for GitOps
- **k9s, kubectx, kubens, stern** for K8s debugging
- **Talosctl** for Talos Linux management

### Development Utilities
- **tmux, htop, ncdu, iotop** for system monitoring
- **ripgrep, silversearcher-ag, fd** for fast file searching
- **bat, httpie, jq** for data processing
- **pandoc** for document conversion
- **direnv** for environment management
- **sccache** for faster compilation
- **dive** for Docker image analysis

### Database Tools
- **PostgreSQL, MySQL, Redis, SQLite** clients
- **pgcli, mycli** for enhanced database interaction

### Networking & Debugging
- **mtr, traceroute, nmap** for network diagnostics
- **strace, lsof, psmisc** for process debugging
- **openssl, gnupg** for security operations

### Build Tools
- **ninja-build, meson** for modern build systems
- **pkg-config, build-essential** for compilation
- **pigz, pbzip2, pxz** for fast compression

### Development Workflow Helpers
- **TaskMaster AI** for task management
- **Toolman client** for MCP server interaction
- **Development helper scripts** for quick project setup
- **Pre-configured Python virtual environment template**
- **Shell aliases and git configurations** for productivity