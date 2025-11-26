# Claude Code Agent Image

This is the Claude Code agent image (`ghcr.io/5dlabs/claude:latest`) that provides a specialized development environment with Claude AI integration, built on top of the runtime base image.

## Included Tools

The Claude Code agent inherits all tools from the runtime base image, plus specialized Claude Code integration.

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

### Claude-Specific Features
- **Claude Code Integration** - Native Claude AI assistant integration
- **Custom Claude Configuration** - Agent-specific settings and prompts
- **Claude Git Identity** - Configured as "Claude Code Agent" with proper email
- **Minimal Overhead** - Only Claude-specific customizations on top of comprehensive runtime base