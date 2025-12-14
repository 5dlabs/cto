# Agent × CLI End-to-End Tests

Run actual agents against real CLIs and capture their code output.

## How It Works

1. **Empty repo** - Test repo starts with just a task definition
2. **Single CodeRun** - Submits one task to the agent
3. **Agent generates code** - The agent implements the task from scratch
4. **Capture everything** - Code + stdout/logs saved to disk

## Directory Structure

```
e2e/
├── README.md
├── scripts/
│   ├── run-single-task.sh    # Run one agent/CLI, capture output
│   └── collect-output.sh     # Collect from completed runs
└── outputs/                  # Results (gitignored)
    └── {agent}-{cli}/
        ├── run-info.json     # Metadata
        ├── code/             # Generated source code
        ├── code.patch        # Git diff
        └── logs/
            └── stdout.log    # Full container output
```

## Quick Start

```bash
# Run Rex with Claude CLI
./scripts/run-single-task.sh rex claude

# Results appear in:
# - outputs/rex-claude/code/      (generated Rust code)
# - outputs/rex-claude/logs/      (stdout from container)
```

## Prerequisites

- `kubectl` configured for CTO cluster
- `gh` CLI authenticated
- Vault access (API keys pulled automatically)

## Test Scenarios

Each agent gets a realistic task matching their specialty:

| Agent | Task | Language |
|-------|------|----------|
| Rex | HTTP retry logic | Rust |
| Blaze | DataTable component | React/TS |
| Grizz | Rate limiter middleware | Go |
| Nova | WebSocket notifications | Node.js |
| Tap | Biometric auth | Expo |
| Spark | System tray | Electron |

## Cost & Time

| Scope | Time | Est. Cost |
|-------|------|-----------|
| 1 run | 5-15 min | $0.50-2 |
| 1 agent (6 CLIs) | 30-90 min | $3-12 |
| Full matrix (48) | 4-8 hours | $24-96 |

