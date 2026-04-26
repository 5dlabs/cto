# Dual Ralph Self-Healing System

A dual-agent autonomous monitoring and self-remediation system for the CTO platform lifecycle.

## Overview

The Dual Ralph system uses two specialized AI agents working in coordination:

| Agent | Model | Role |
|-------|-------|------|
| **Monitor** | GPT-5.2 (via `droid`) | Gate checking, failure detection, phase progression |
| **Remediation** | Claude | Issue investigation, fixing, verification |

The agents communicate through a shared coordination file (`ralph-coordination.json`) rather than direct interaction.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Dual Ralph System                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐         ┌─────────────────┐               │
│  │  Monitor Agent  │         │ Remediation Agent│               │
│  │   (GPT-5.2)     │         │    (Claude)      │               │
│  │                 │         │                  │               │
│  │ • Check gates   │         │ • Poll queue     │               │
│  │ • Detect fails  │         │ • Claim issues   │               │
│  │ • Write queue   │         │ • Fix problems   │               │
│  │ • Track phase   │         │ • Verify fixes   │               │
│  └────────┬────────┘         └────────┬─────────┘               │
│           │                           │                         │
│           │    ┌──────────────────┐   │                         │
│           └───▶│ ralph-coordination│◀──┘                         │
│                │      .json       │                             │
│                │                  │                             │
│                │ • issueQueue     │                             │
│                │ • circuitBreaker │                             │
│                │ • session        │                             │
│                │ • stats          │                             │
│                └──────────────────┘                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Key Features Implemented

### 1. Circuit Breaker Pattern

Prevents runaway loops when the system gets stuck:

- **Tracks**: No-progress count, repeated same errors
- **Opens after**: 3 no-progress loops OR 5 same errors
- **Recovery**: Auto-transitions to half-open after 5 minutes
- **Closes**: On successful progress

```json
{
  "circuitBreaker": {
    "state": "closed",        // closed | open | half-open
    "noProgressCount": 0,
    "sameErrorCount": 0,
    "lastError": null,
    "openedAt": null
  }
}
```

### 2. EXIT_SIGNAL Gate

Dual-condition exit logic to prevent premature termination:

- **Requires BOTH**: Completion indicators AND explicit `EXIT_SIGNAL: true`
- **Decision matrix**:
  - indicators >= 2 AND EXIT_SIGNAL=true → Exit
  - indicators >= 2 AND EXIT_SIGNAL=false → Continue
  - indicators >= 2 AND missing → Continue (default false)

The agent should output in their response:
```
EXIT_SIGNAL: true
```

### 3. Session Management

Tracks agent sessions with auto-reset:

- **Session ID**: Unique identifier per run
- **Expiration**: Default 24 hours
- **Auto-reset triggers**:
  - Circuit breaker opens
  - Session expires
  - Manual reset via `ralph-dual.sh reset-session`

### 4. Response Analyzer Library

Shared library (`scripts/2026-01/ralph-response-analyzer.sh`) providing:

- `ra_extract_exit_signal()` - Parse EXIT_SIGNAL from output
- `ra_count_completion_indicators()` - Heuristic completion detection
- `ra_should_exit_gracefully()` - Dual-condition decision
- `ra_detect_struggle()` - Identify agent going in circles
- `ra_detect_progress()` - Identify productive work
- `ra_analyze_response()` - JSON summary

### 5. Struggle Detection

Identifies when agents are stuck:

- "I'm stuck", "I cannot figure out"
- Repeated errors (ERROR:.*ERROR:.*ERROR:)
- Going in circles patterns
- Confusion indicators

## Files

| File | Purpose |
|------|---------|
| `scripts/2026-01/ralph-dual.sh` | Launcher and management CLI |
| `scripts/2026-01/ralph-monitor.sh` | Monitor agent script |
| `scripts/2026-01/ralph-remediation.sh` | Remediation agent script |
| `scripts/2026-01/ralph-response-analyzer.sh` | Shared analysis library |
| `lifecycle-test/ralph-coordination.json` | Coordination state |
| `lifecycle-test/ralph-cto.json` | Configuration (requires `dualAgent` section) |
| `lifecycle-test/monitor-prompt.md` | Monitor agent system prompt |
| `lifecycle-test/remediation-prompt.md` | Remediation agent system prompt |

## Configuration

The system requires a `dualAgent` section in `ralph-cto.json`:

```json
{
  "dualAgent": {
    "enabled": true,
    "monitor": {
      "cli": "droid",
      "command": "droid exec --model gpt-5.2 --auto medium -f",
      "promptPath": "lifecycle-test/monitor-prompt.md"
    },
    "remediation": {
      "cli": "claude",
      "command": "claude --dangerously-skip-permissions -p",
      "promptPath": "lifecycle-test/remediation-prompt.md"
    },
    "coordination": {
      "filePath": "lifecycle-test/ralph-coordination.json",
      "pollIntervalSeconds": 10,
      "maxConcurrentIssues": 1,
      "remediationTimeoutSeconds": 600,
      "maxRetryAfterRemediation": 3
    },
    "circuitBreaker": {
      "noProgressThreshold": 3,
      "sameErrorThreshold": 5,
      "recoveryMinutes": 5
    },
    "session": {
      "expirationHours": 24,
      "autoResetOnCircuitBreak": true
    },
    "exitDetection": {
      "requireExplicitSignal": true,
      "completionIndicatorThreshold": 2
    }
  }
}
```

## Commands

```bash
# Start both agents
./scripts/2026-01/ralph-dual.sh start

# Stop both agents
./scripts/2026-01/ralph-dual.sh stop

# Check status (shows circuit breaker, session, stats)
./scripts/2026-01/ralph-dual.sh status

# View issue queue
./scripts/2026-01/ralph-dual.sh queue

# View recent logs
./scripts/2026-01/ralph-dual.sh logs

# Attach to monitor screen session
./scripts/2026-01/ralph-dual.sh attach monitor

# Attach to remediation screen session
./scripts/2026-01/ralph-dual.sh attach remediation

# Reset everything
./scripts/2026-01/ralph-dual.sh reset

# Reset session only (start fresh context)
./scripts/2026-01/ralph-dual.sh reset-session

# View circuit breaker details
./scripts/2026-01/ralph-dual.sh circuit-status

# Reset circuit breaker to closed
./scripts/2026-01/ralph-dual.sh circuit-reset
```

## Workflow

1. **Monitor Agent** runs through lifecycle phases defined in `ralph-cto.json`
2. For each phase, it executes gates (shell commands that must exit 0)
3. If a gate fails:
   - Monitor writes issue to `issueQueue` in coordination file
   - Monitor waits for remediation (with timeout)
4. **Remediation Agent** polls for pending issues
5. When issue found:
   - Claims it (sets status to "claimed")
   - Builds context from gate failure details
   - Runs Claude to investigate and fix
   - Re-runs gate to verify fix
   - Updates issue status (resolved/failed)
6. Monitor retries gate after remediation
7. On success, advances to next phase

## Handoff Protocol

When Monitor detects a failure:

```json
{
  "id": "issue-1737347353-12345",
  "timestamp": "2026-01-20T04:00:00Z",
  "phase": "intake",
  "gate": "linear-sidecar-running",
  "exitCode": 1,
  "logFile": "/path/to/gate.log",
  "diagnostics": {
    "logTail": "...",
    "command": "kubectl get pods..."
  },
  "status": "pending",
  "retryCount": 0
}
```

Remediation claims and resolves:

```json
{
  "status": "resolved",
  "resolvedAt": "2026-01-20T04:05:00Z",
  "resolution": "Gate passed after remediation"
}
```

## Logs

| Location | Content |
|----------|---------|
| `/tmp/ralph-monitor.log` | Monitor agent output |
| `/tmp/ralph-remediation.log` | Remediation agent output |
| `lifecycle-test/ralph-logs/` | Individual gate execution logs |
| `lifecycle-test/progress.txt` | Human-readable progress log |
| `lifecycle-test/report.json` | Structured event log |

## Troubleshooting

### Circuit Breaker Opens

```bash
# Check why it opened
./scripts/2026-01/ralph-dual.sh circuit-status

# Reset and retry
./scripts/2026-01/ralph-dual.sh circuit-reset
./scripts/2026-01/ralph-dual.sh restart
```

### Agents Not Starting

```bash
# Check if screen is installed
which screen

# Check for existing sessions
screen -list

# Kill orphan processes
pkill -f ralph-monitor.sh
pkill -f ralph-remediation.sh

# Reset and retry
./scripts/2026-01/ralph-dual.sh reset
./scripts/2026-01/ralph-dual.sh start
```

### Missing dualAgent Config

If you see errors about missing config, ensure `ralph-cto.json` has the `dualAgent` section (see Configuration above).

## Session Summary (2026-01-20)

Implemented the Dual Ralph improvements plan:

1. **Circuit Breaker** - Added to `ralph-monitor.sh` with state tracking
2. **EXIT_SIGNAL** - Added parsing and dual-condition logic to `ralph-remediation.sh`
3. **Response Analyzer** - Created `ralph-response-analyzer.sh` library
4. **Session Management** - Added session tracking with auto-reset
5. **Config Updates** - Updated JSON schemas with new sections
6. **New Commands** - Added `reset-session`, `circuit-status`, `circuit-reset`

The system is ready for testing. Start with:

```bash
./scripts/2026-01/ralph-dual.sh start
./scripts/2026-01/ralph-dual.sh status
```

Monitor progress with:

```bash
./scripts/2026-01/ralph-dual.sh logs
# or attach directly
./scripts/2026-01/ralph-dual.sh attach monitor
```
