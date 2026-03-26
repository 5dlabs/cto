# The Bridge — Unified Intake Gateway

## Status: Planned (post-Phase A green)

## Vision

One application. One process. One place to sit.

Like the bridge of a starship — all comms, sensors, ops, and alerts converge in a single command center. Morgan interfaces through the Bridge; the human sits at the Bridge.

## Current State (what gets consolidated)

| App | Role | Port | Becomes |
|-----|------|------|---------|
| `apps/discord-bridge` | Discord bot/webhook adapter | 3200 | `workers/discord.ts` |
| `apps/linear-bridge` | Linear webhook/API adapter | 3100 | `workers/linear.ts` |
| `apps/intake-agent` | PRD processing, Stitch, design | CLI | `workers/agent.ts` + `workers/stitch.ts` |
| `apps/intake-util` | bridge-notify, validate, register-run | CLI | `shared/notify.ts` + `shared/state.ts` |
| `apps/lobster-voice` | ElevenLabs TTS, voice identity | CLI | `workers/voice.ts` |

## Target Architecture

```
apps/bridge/
├── src/
│   ├── index.ts              # Entry point, start workers, single health endpoint
│   ├── workers/
│   │   ├── discord.ts        # Discord.js client + HTTP routes (comms)
│   │   ├── linear.ts         # Linear SDK + webhook handler (ops/mission log)
│   │   ├── voice.ts          # ElevenLabs + process identity (audio alerts)
│   │   ├── stitch.ts         # StitchDirectClient (viewscreen/design)
│   │   └── agent.ts          # PRD processing, task expansion (crew)
│   ├── shared/
│   │   ├── state.ts          # SQLite bridge-state, run registry (ship's computer)
│   │   ├── notify.ts         # Cross-worker notifications (replaces HTTP bridge-notify)
│   │   └── config.ts         # Unified config from cto-config.json + env
│   └── cli/
│       └── commands.ts       # CLI subcommands (validate, register-run, etc.)
├── package.json
└── tsconfig.json
```

## Key Benefits

1. **Eliminates "bridges not running" failures** — the most frequent pipeline blocker
2. **Single `op run`** — one set of secrets, one process
3. **Direct function calls** — `register-run` is a function, not an HTTP POST to localhost
4. **Scales with new channels** — adding Slack, email, web UI is just a new worker file
5. **Single health endpoint** — one URL to check, one thing to monitor

## Runtime Modes

```bash
# Local development — everything
bridge --all

# Local development — specific workers
bridge --workers discord,linear,voice

# Production (k8s) — separate deployments, same image
# Deployment A: bridge --workers discord
# Deployment B: bridge --workers linear
# Deployment C: bridge --workers voice,stitch,agent
```

## Migration Plan

1. Create `apps/bridge/` with the worker architecture
2. Move Discord bridge logic into `workers/discord.ts`
3. Move Linear bridge logic into `workers/linear.ts`
4. Move voice logic into `workers/voice.ts`
5. Move Stitch client into `workers/stitch.ts`
6. Move agent operations into `workers/agent.ts`
7. Collapse `intake-util` CLI into `shared/` + `cli/`
8. Update Lobster workflow steps to call Bridge (direct or single HTTP endpoint)
9. Update `run-local-bridges.sh` → just `bridge --all`
10. Verify pipeline green on new architecture
11. Remove old apps once verified

## What Gets Preserved

- All existing Discord.js client logic
- All existing Linear SDK/webhook logic
- ElevenLabs speaker map and voice identity system
- StitchDirectClient (already custom, just moves)
- SQLite state management
- All Lobster workflow definitions (steps just point to new commands)

## Full Audio Narration ("The Planning Podcast")

The Bridge should narrate the entire planning phase so you can listen without watching a screen. Every meaningful event is spoken aloud with the appropriate voice/tone.

### What gets narrated

| Event | Voice | Example |
|-------|-------|---------|
| Step transitions | System/narrator | "Starting design intake for sigma-1" |
| Parsed tasks | Narrator | "Task 1: Bootstrap Development Infrastructure, assigned to Bolt, Kubernetes stack" |
| Decision points discovered | Alert | "Decision point identified: Should we use Redis or Valkey for caching?" |
| Optimist argument | Optimist voice | Full argument, summarized for speech |
| Pessimist argument | Pessimist voice | Full counterargument, summarized |
| Voter reasoning + vote | Unique voter voice | "Voter 3, Gemini: I side with the optimist on Valkey. The community fork has stronger momentum..." |
| Design detection | Narrator | "Frontend detected. Targets: web and mobile. Stitch generating candidates." |
| Compiled brief | Compiler voice | Key decisions and final synthesis |
| Linear/repo setup | System | "Linear session created. Repository confirmed at 5dlabs/sigma-1" |
| Final summary | Narrator | "Intake complete. 10 tasks expanded, 3 decision points resolved, 12 Linear issues created" |
| Errors/warnings | Alert voice | "Warning: Stitch generation timed out. Falling back to ingest-only mode" |

### Design principle

The entire planning phase should feel like sitting in a meeting room where the team is planning aloud. You hear the whole conversation — scope, concerns, rebuttals, votes, and the final plan. No screen required.

This is the primary UX differentiator of the Bridge: it's not a dashboard you check, it's a room you sit in.

## Prerequisites

- Phase A pipeline verified green on current architecture
- sigma-1 full end-to-end success

## Star Trek Mental Model

| Bridge Station | Worker | Function |
|---------------|--------|----------|
| Communications | `discord.ts` | Hailing frequencies, crew chat |
| Operations | `linear.ts` | Mission log, task tracking |
| Audio/Alert | `voice.ts` | Status reports, red alert |
| Viewscreen | `stitch.ts` | Visual design, UI generation |
| Crew | `agent.ts` | Execute orders, PRD processing |
| Ship's Computer | `shared/state.ts` | State, registry, memory |
