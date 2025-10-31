# Runtime Base Image

This is the base runtime image (`ghcr.io/5dlabs/runtime:latest`) that serves as the foundation for all agent containers in the 5DLabs CTO platform.

## Included Tools

### Core Development Stack
- **Node.js 20 LTS** with npm, yarn, and pnpm (latest)
- **Python 3** with pip, venv, and setuptools
- **Go** with latest stable version and GOPATH setup
- **Rust** with stable toolchain, clippy, rustfmt, and rust-analyzer
- **Git** with enhanced tools and GitHub CLI integration

### Networking & Network Analysis
- **nmap** - Network scanning and discovery
- **netcat (nc)** - Network connections and data transfer
- **netstat** - Network statistics and active connections
- **ss** - Modern socket statistics (from iproute2)
- **ip** - Modern network configuration (from iproute2)
- **tcpdump** - Command-line packet analyzer
- **tshark** - Wireshark command-line network protocol analyzer
- **mtr** - Network diagnostic tool combining traceroute and ping
- **traceroute** - Network path tracing
- **arping** - ARP ping utility for Layer 2 connectivity
- **iperf3** - Network bandwidth testing
- **hping3** - TCP/IP packet assembler and analyzer
- **socat** - Multipurpose relay tool for bidirectional data transfer
- **telnet** - User interface to TELNET protocol
- **OpenSSH Client** - SSH client for secure remote access
- **dig, nslookup, host** - DNS resolution and querying tools
- **curl, wget** - HTTP/HTTPS clients for API testing
- **ethtool** - Display and change Ethernet device settings
- **bridge-utils** - Ethernet bridging utilities
- **vlan** - VLAN configuration utilities

### System Administration & Monitoring
- **strace** - System call tracer for debugging
- **lsof** - List open files and network sockets
- **psmisc** - Process utilities (pstree, killall, etc.)
- **htop** - Interactive process viewer
- **iotop** - I/O monitoring tool
- **procps** - System and process utilities
- **tree** - Directory tree display
- **ripgrep (rg)** - Fast text search tool
- **fd** - Modern find replacement
- **bat** - Enhanced cat with syntax highlighting
- **httpie** - Modern HTTP client
- **fzf** - Fuzzy finder for command-line
- **zsh** - Advanced shell with completion
- **sudo** - Superuser privileges
- **openssl** - SSL/TLS toolkit
- **gnupg-agent** - GNU Privacy Guard

### Code Quality & Linting
- **shellcheck** - Shell script static analysis
- **yamllint** - YAML file linter
- **markdownlint** - Markdown file linter
- **ESLint** - JavaScript/TypeScript linter
- **Prettier** - Code formatter
- **Black** - Python code formatter
- **isort** - Python import sorter
- **flake8** - Python style guide enforcement
- **mypy** - Python static type checker
- **ruff** - Fast Python linter and formatter
- **bandit** - Python security linter
- **safety** - Python dependency vulnerability scanner

### Development Utilities
- **tmux** - Terminal multiplexer
- **ncdu** - Disk usage analyzer
- **silversearcher-ag** - Fast text search tool
- **direnv** - Environment variable manager
- **sd** - Simple sed replacement

### Cloud Development Tools
- **AWS CLI v2** - Amazon Web Services command-line interface (latest version)
- **Docker CLI** - Container management
- **Docker Compose** - Multi-container orchestration

### Container Tools
- **Docker CLI** - Container management and inspection

### Database Clients
- **PostgreSQL client** - psql command-line tool
- **MySQL client** - mysql command-line tool
- **Redis client** - redis-cli command-line tool
- **SQLite3** - Lightweight database client

### Documentation Tools
- **pandoc** - Universal document converter

### Build Tools
- **ninja-build** - Fast build system
- **meson** - Software build system
- **pkg-config** - Library configuration tool
- **build-essential** - GCC, make, and standard build tools
- **Clang/LLVM** - Modern C/C++ compiler
- **mold** - Fast linker for faster compilation
- **lld** - LLVM linker
- **binutils** - Binary utilities (objdump, nm, etc.)

### Text Processing & Utilities
- **jq** - JSON processor
- **yq** - YAML processor
- **pandoc** - Universal document converter
- **unzip** - Archive extraction
- **pigz, pbzip2** - Fast compression utilities
- **ca-certificates** - SSL certificate bundle
- **man-db** - Manual pages

### Additional Programming Languages
- **PHP CLI** - PHP command-line interpreter
- **Ruby** - Ruby programming language
- **Perl** - Perl programming language

### Kubernetes & Cloud Native
- **kubectl** - Kubernetes command-line tool
- **Helm** - Kubernetes package manager
- **Argo CD CLI** - GitOps continuous delivery tool
- **Argo Workflows CLI** - Workflow orchestration tool
- **k9s** - Terminal-based Kubernetes UI
- **kubectx/kubens** - Kubernetes context and namespace switching
- **stern** - Multi-pod log tailing for Kubernetes
- **kustomize** - Kubernetes native configuration management
- **Talosctl** - Talos Linux cluster management

### Development & Build Tools
- **TaskMaster AI** - AI-powered task management
- **Toolman Client** - MCP server integration
- **Pre-commit** - Git hooks framework
- **SQLFluff** - SQL linter and formatter
- **Pip-tools** - Python dependency management
- **TypeScript** - TypeScript compiler
- **shadcn CLI** - shadcn/ui component CLI for modern UI development
- **Node.js development tools** - nodemon, concurrently, cross-env, dotenv-cli, zx

### Development Environment
- **Shell Enhancements** - Vim editor, useful aliases (ll, grep with color)
- **Git Configuration** - Optimized defaults (main branch, rebase, auto-setup remote)
- **Python Virtual Environment Template** - Ready-to-use venv setup
- **Development Helper Scripts** - Quick project setup utilities
- **Zsh with Plugins** - Advanced shell with git and fzf integration
- **Command History Persistence** - Preserved across container restarts

### Version Control & Git Tools
- **Git** with advanced features and optimized defaults
- **GitHub CLI (gh)** - GitHub command-line interface with Copilot and CodeQL extensions
- **git-delta** - Enhanced git diff viewer
- **git-lfs** - Git Large File Storage
- **git-extras** - Additional Git utilities