# CTO Lite

AI-powered development for individual developers. Run the full CTO platform on your local machine.

## Overview

CTO Lite is a desktop application that brings AI-assisted development to your workstation. It runs a local Kubernetes cluster (Kind) with the CTO agent system, allowing you to:

- **Develop with AI agents** - Morgan for intake, Grizz/Nova for backend, Blaze for frontend
- **Quality assurance** - Cleo for code quality, Cipher for security, Tess for testing
- **Local deployment** - Bolt deploys to Docker on your machine

## Quick Start

### 1. Requirements

- **Docker Desktop**, **OrbStack**, **Colima**, or **Podman** - Container runtime
- **API Key** - From [Anthropic](https://console.anthropic.com/) or [OpenAI](https://platform.openai.com/)

### 2. Install

Download from [cto.dev](https://cto.dev/download):

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | [CTO Lite.dmg](https://cto.dev/download/macos-arm64) |
| macOS (Intel) | [CTO Lite.dmg](https://cto.dev/download/macos-x64) |
| Windows | [CTO Lite.msi](https://cto.dev/download/windows) |
| Linux | [CTO Lite.AppImage](https://cto.dev/download/linux) |

### 3. Setup

Launch CTO Lite and complete the setup wizard:

1. **Runtime Check** - Verifies Docker is installed and running
2. **Stack Selection** - Choose Go (Grizz) or TypeScript (Nova) backend
3. **API Keys** - Enter your Anthropic or OpenAI API key (stored in system keychain)
4. **GitHub** - Connect for repository access (optional)
5. **Cloudflare** - Enable webhook tunnels (optional)
6. **Create Cluster** - Creates local Kind cluster (~1GB RAM)
7. **Deploy** - Installs CTO Lite services

### 4. Use

From the Dashboard:

1. Click **+** to create a new workflow
2. Enter your GitHub repository URL
3. Describe what you want to build
4. Watch the agents work!

## Features

### Agents

| Agent | Role | Description |
|-------|------|-------------|
| Morgan | Intake | Analyzes your request, creates tasks |
| Grizz | Backend | Go development (chi, grpc, pgx) |
| Nova | Backend | TypeScript development (Elysia, Effect, Bun) |
| Blaze | Frontend | React + TypeScript |
| Cleo | Quality | Code review, best practices |
| Cipher | Security | Security analysis |
| Tess | Testing | Test generation and coverage |
| Bolt | Deploy | Local Docker deployment |

### Workflow

```
Request → Morgan → [Grizz/Nova + Blaze] → Cleo → Cipher → Tess → Bolt
```

Each step creates commits in a feature branch, culminating in a PR.

## Architecture

```
┌─────────────────────────────────────────────┐
│              CTO Lite App                    │
│  ┌─────────────┐  ┌───────────────────────┐ │
│  │ Setup       │  │ Dashboard             │ │
│  │ Wizard      │  │ (Workflows, Logs)     │ │
│  └─────────────┘  └───────────────────────┘ │
└────────────────────┬────────────────────────┘
                     │
┌────────────────────┴────────────────────────┐
│              Kind Cluster                    │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Argo     │  │ Controller│  │ PM Server │  │
│  │ Workflows│  │          │  │           │  │
│  └──────────┘  └──────────┘  └───────────┘  │
│                                              │
│  Agent Pods (on-demand):                     │
│  [Morgan] [Grizz/Nova] [Blaze] [Cleo] ...   │
└──────────────────────────────────────────────┘
```

## Troubleshooting

### Docker not detected

Make sure Docker Desktop, OrbStack, or Colima is running:

```bash
# Check Docker
docker ps

# Start Colima
colima start

# Start OrbStack
open -a OrbStack
```

### Kind cluster won't start

Check resources available to Docker:

- Docker Desktop: Preferences → Resources → Memory (4GB+ recommended)
- Colima: `colima start --memory 4`

### Workflow stuck

1. Check workflow logs in Dashboard
2. Verify API key is valid
3. Check cluster pods: `kubectl get pods -n cto-lite`

### Reset everything

```bash
# Delete cluster
kind delete cluster --name cto-lite

# Clear app data (macOS)
rm -rf ~/Library/Application\ Support/ai.5dlabs.cto-lite
```

## Development

### Building from source

```bash
# Clone
git clone https://github.com/5dlabs/cto
cd cto

# Install UI dependencies
cd crates/cto-lite/ui
npm install
npm run build

# Build Tauri app
cd ../tauri
npm install
npm run tauri build
```

### Running in development

```bash
# Start frontend dev server
cd crates/cto-lite/ui
npm run dev

# In another terminal, start Tauri
cd crates/cto-lite/tauri
npm run tauri dev
```

## License

Proprietary - 5D Labs

## Support

- Issues: [GitHub Issues](https://github.com/5dlabs/cto/issues)
- Discord: [CTO Community](https://discord.gg/cto)
- Email: support@5dlabs.ai
