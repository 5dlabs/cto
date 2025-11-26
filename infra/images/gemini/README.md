# Google Gemini Agent Image

This is the Google Gemini agent image (`ghcr.io/5dlabs/gemini:latest`) that provides a specialized development environment with Google Gemini AI integration, built on top of the runtime base image.

## Included Tools

The Gemini agent inherits all tools from the runtime base image, plus specialized Gemini AI integration.

### Core Development Stack *(from runtime base)*
- **Node.js 20 LTS** with npm, yarn, and pnpm
- **Python 3** with pip, venv, and setuptools
- **Go** with latest stable version and GOPATH setup
- **Rust** with stable toolchain, clippy, rustfmt, and rust-analyzer
- **PHP CLI, Ruby, Perl** for multi-language development
- **Git** with enhanced tools and GitHub CLI integration

### Networking & Network Analysis *(from runtime base)*
- **nmap, ncat, netstat, ss, ip** for network scanning and connections
- **tcpdump, wireshark-cli** for network packet analysis
- **mtr, traceroute, ping, arping** for network path analysis
- **iperf3, hping3** for network performance testing
- **socat, telnet, openssh-client** for network utilities
- **dig, nslookup, host** for DNS resolution
- **curl, wget** for HTTP/HTTPS testing
- **ethtool, bridge-utils, vlan** for network interface management

### System Administration & Monitoring *(from runtime base)*
- **strace, lsof, psmisc, htop, iotop** for process debugging and monitoring
- **ripgrep, silversearcher-ag, fd, bat** for fast file searching and processing
- **tmux, ncdu, tree, fzf, zsh** for terminal and shell enhancements
- **httpie, jq, yq** for API testing and data processing
- **openssl, gnupg-agent** for security operations
- **sudo** for privileged operations

### Code Quality & Linting *(from runtime base)*
- **ESLint, Prettier, TypeScript** for JavaScript/TypeScript
- **Black, isort, flake8, mypy, ruff** for Python
- **Clippy, rustfmt** for Rust
- **ShellCheck, Yamllint, Markdownlint** for shell and markup
- **SQLFluff** for SQL
- **Bandit, Safety** for security scanning

### Cloud & Infrastructure *(from runtime base)*
- **AWS CLI** for cloud development
- **kubectl, helm, kustomize** for Kubernetes
- **Docker CLI and Docker Compose** for containers
- **Argo CD, Argo Workflows** for GitOps
- **k9s, kubectx, kubens, stern** for K8s debugging
- **Talosctl** for Talos Linux management

### Database Tools *(from runtime base)*
- **PostgreSQL, MySQL, Redis, SQLite** clients
- **pgcli, mycli** for enhanced database interaction

### Build Tools *(from runtime base)*
- **ninja-build, meson** for modern build systems
- **pkg-config, build-essential** for compilation
- **pigz, pbzip2, pxz** for fast compression
- **Clang/LLVM, mold, lld** for compilation and linking

### Development Utilities *(from runtime base)*
- **pandoc** for document conversion
- **direnv** for environment management
- **sccache** for faster compilation
- **dive** for Docker image analysis
- **TaskMaster AI** for task management
- **Tools Client** for MCP server integration
- **Pre-commit** for Git hooks framework
- **Pip-tools** for Python dependency management
- **Node.js development tools** (nodemon, concurrently, cross-env, dotenv-cli, zx)

### Gemini-Specific Features
- **Gemini CLI Integration** - Native Google Gemini AI assistant integration
- **Custom Gemini Configuration** - Agent-specific settings and prompts
- **Gemini Git Identity** - Configured as "5D Labs Agent" with proper email
- **Minimal Overhead** - Only Gemini-specific customizations on top of comprehensive runtime base

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GOOGLE_API_KEY` | ✅ | API key for Google Gemini access |
| `TZ` | ❌ | Timezone setting (default: system timezone) |

## Building the Image

```bash
# Build locally
docker build -t gemini:latest .

# Build with specific Gemini version
docker build --build-arg VERSION=1.0.0 -t gemini:latest .
```

## Running Locally

```bash
# Interactive development environment
docker run -it \
  -e GOOGLE_API_KEY="your-api-key" \
  -v $(pwd):/workspace \
  gemini:latest

# With timezone setting
docker run -it \
  -e GOOGLE_API_KEY="your-api-key" \
  -e TZ="America/New_York" \
  -v $(pwd):/workspace \
  gemini:latest
```

## Kubernetes TaskRun Integration

```yaml
apiVersion: tekton.dev/v1beta1
kind: TaskRun
metadata:
  name: gemini-development-task
spec:
  taskSpec:
    steps:
    - name: gemini-code
      image: gemini:latest
      env:
      - name: GOOGLE_API_KEY
        valueFrom:
          secretKeyRef:
            name: gemini-secrets
            key: api-key
      script: |
        #!/bin/bash
        # Your development tasks here
        gemini-cli "Generate documentation for this project"
```

## API Key Setup

Get your Google API key from [Google AI Studio](https://makersuite.google.com/app/apikey) and add it to your secrets:

```bash
kubectl create secret generic gemini-secrets \
  --from-literal=api-key="your-google-api-key"
```

## Notes

**⚠️ Important**: The Gemini CLI package name in the Dockerfile may need to be updated based on Google's official release. The current Dockerfile includes fallback logic to handle different possible package names. Please verify the correct package name and update the Dockerfile accordingly.

Possible package names to check:
- `@google-cloud/generative-ai`
- `@google/generative-ai`
- `@google/gemini-cli`
- Or a standalone installer script from Google

## Integration with Orchestrator

This image is used by the orchestrator platform for:

1. **Code Generation**: AI-powered code generation using Gemini
2. **Code Analysis**: Intelligent code review and suggestions
3. **Task Automation**: AI-powered development task execution
4. **Testing**: Automated test generation and execution


