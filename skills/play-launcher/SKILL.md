---
name: play-launcher
description: Launch CTO play.lobster.yaml workflows using cto-play with merged platform defaults
triggers:
  - "launch play"
  - "run play"
  - "start play"
  - "execute play"
  - "cto-play"
  - "play.lobster"
---

# Play Launcher (`cto-play`)

CLI utility for launching play.lobster.yaml workflows with CTO platform defaults automatically merged.

## When to Use

- You need to run a play for a project (test-sandbox, test-sandbox-2, or any repo with `.tasks/play-config.yaml`)
- You want defaults from cto-config.json to pass through without manually building args
- You need to launch multiple plays concurrently for different repos

## Quick Reference

```bash
# Launch a play with all defaults from play-config.yaml
cto-play --repo-path /path/to/repo

# Dry run — show merged args without executing
cto-play --repo-path /path/to/repo --dry-run

# Override provider and model
cto-play --repo-path /path/to/repo --provider fireworks --model accounts/fireworks/models/kimi-k2p6

# Enable Discord notifications
cto-play --repo-path /path/to/repo --discord true --discord-bridge-url http://discord-bridge:3001

# Use a specific kubeconfig
cto-play --repo-path /path/to/repo --kubeconfig /path/to/kubeconfig

# Override namespace
cto-play --repo-path /path/to/repo --namespace staging
```

## How It Works

1. Reads `.tasks/play-config.yaml` from the repo for project-specific defaults
2. Reads CTO config from `/etc/cto/config.json`, `CTO_CONFIG` env, or `./cto-config.json`
3. Merges with priority: **CLI flags > play-config.yaml > CTO defaults > lobster defaults**
4. Invokes `lobster run .tasks/docs/play.lobster.yaml --args-json <merged>`

## Config File Location

Each repo must have `.tasks/play-config.yaml` (or `.tasks/docs/play-config.yaml`) with:

```yaml
kubeconfig:
  namespace: cto         # K8s namespace for CRDs
  context: ""            # kubectl context (empty = current)
  path: ""               # kubeconfig path (empty = default)

project:
  repoUrl: "https://github.com/5dlabs/test-sandbox.git"
  service: "test-sandbox"
  baseBranch: "main"

defaults:
  provider: "fireworks"
  model: "accounts/fireworks/models/kimi-k2p6"
  cli: "claude"
  harnessAgent: "openclaw"
  enableDocker: true
  quality: true
  security: true
  testing: true
  deployment: false

discord:
  enabled: false
  bridgeUrl: ""
```

## All CLI Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--repo-path` | Path to repo root | `.` (current dir) |
| `--cto-config` | Path to CTO config JSON | Auto-discovered |
| `--kubeconfig` | Override kubeconfig path | From play-config |
| `--namespace` | Override K8s namespace | `cto` |
| `--provider` | Override inference provider | `fireworks` |
| `--model` | Override model | `kimi-k2p6` |
| `--cli` | Override coding CLI | `claude` |
| `--harness-agent` | Override harness (openclaw/hermes) | `openclaw` |
| `--repo-url` | Override repository URL | From play-config |
| `--discord` | Enable/disable Discord | From play-config |
| `--discord-bridge-url` | Discord bridge URL | From play-config |
| `--linear-session-id` | Linear session ID | Empty |
| `--linear-team-id` | Linear team ID | Empty |
| `--dry-run` | Show args without executing | false |

## Common Workflows

### Launch single project
```bash
cto-play --repo-path /workspace/repos/test-sandbox
```

### Launch two projects concurrently
```bash
cto-play --repo-path /workspace/repos/test-sandbox &
cto-play --repo-path /workspace/repos/test-sandbox-2 &
wait
```

### Ad-hoc repo (clone first, then launch)
```bash
git clone https://github.com/5dlabs/new-project.git /tmp/new-project
cto-play --repo-path /tmp/new-project --dry-run  # verify first
cto-play --repo-path /tmp/new-project
```

## Rules

- ALWAYS use `--dry-run` first when launching a play for the first time to verify merged args
- If a repo doesn't have `.tasks/play-config.yaml`, create one before launching
- The binary expects `lobster` to be on PATH (installed via `npm i -g @clawdbot/lobster`)
- For concurrent plays, ensure CRD names don't collide (each repo should use unique prefixes)
