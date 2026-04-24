---
name: acp_sessions
description: Spawn and manage ACP coding sessions (Claude Code, Codex, Copilot, OpenCode)
metadata: {"openclaw": {"always": true}}
---

# ACP Sessions

## Status reactions (ALWAYS DO THIS)

When you spawn or work with an ACP session, use emoji reactions on the triggering message to show status. The gateway automatically targets the current inbound message — you do NOT need `messageId` or `to`.

### React when ACP starts:
```json
{ "action": "react", "emoji": "⚙️" }
```

### React when ACP completes successfully:
```json
{ "action": "react", "emoji": "✅" }
{ "action": "react", "emoji": "⚙️", "remove": true }
```

### React when ACP fails:
```json
{ "action": "react", "emoji": "❌" }
{ "action": "react", "emoji": "⚙️", "remove": true }
```

### Backend-specific reaction (ALWAYS add alongside ⚙️):
Show WHICH coding CLI is running so users can tell at a glance:
```json
// Claude Code
{ "action": "react", "emoji": "🟣" }
// Codex (OpenAI)
{ "action": "react", "emoji": "🟢" }
// Copilot (GitHub)
{ "action": "react", "emoji": "🔵" }
// OpenCode / Kimi
{ "action": "react", "emoji": "🟠" }
// Gemini
{ "action": "react", "emoji": "🔴" }
// Cursor
{ "action": "react", "emoji": "⚪" }
```

### Multi-backend stacking (concurrent ACP sessions)
Backend color dots are **stackable** — unlike activity emojis (🧠/🔍/✏️), you may have
several color dots on the same message at once, one per active ACP backend. Rules:

- When you spawn backend **X**, add its color dot. Do NOT remove any other backend's dot.
- When backend **X** completes (success or failure), remove **only X's color dot**.
  Leave dots for backends that are still running.
- Keep ⚙️ on the message as long as **any** ACP session is still running. Remove ⚙️
  only when **the last** ACP session completes.
- Add ✅/❌/💬 only once every spawned backend has reported back.

Example — spawn Claude + Codex in parallel, Claude finishes first:
```
spawn claude  → add 🟣, add ⚙️
spawn codex   → add 🟢   (⚙️ stays, 🟣 stays)
claude done   → remove 🟣 (⚙️ stays because codex is still running)
codex done    → remove 🟢, remove ⚙️, add ✅
```

### Reaction reference:
| State | Emoji | Meaning |
|-------|-------|---------|
| ACP spawning | ⚙️ | Working on it with a coding CLI |
| ACP running (long task) | 🔄 | Still running, making progress |
| ACP complete | ✅ | Task finished successfully |
| ACP failed | ❌ | Something went wrong |
| Thinking / planning | 🧠 | Analyzing or planning before acting |
| Using browser | 🌐 | Browsing web content |
| Searching code | 🔍 | Searching files or codebase |

**Backend identity emojis** (add WITH ⚙️, remove when done):
| Backend | Emoji | Color |
|---------|-------|-------|
| Claude Code | 🟣 | Purple |
| Codex (OpenAI) | 🟢 | Green |
| Copilot (GitHub) | 🔵 | Blue |
| OpenCode / Kimi | 🟠 | Orange |
| Gemini | 🔴 | Red |
| Cursor | ⚪ | White |

Always add the ⚙️ AND the backend emoji BEFORE calling `sessions_spawn`. When a backend
completes, remove that backend's color dot immediately; only remove ⚙️ after the last
ACP session finishes.

## Spawning a coding CLI session

Use `sessions_spawn` with `runtime: "acp"` to start an ACP session. The tool returns a `runId` and `childSessionKey` — use those for status checks, not made-up keys.

```
sessions_spawn({
  runtime: "acp",
  agent: "claude",
  message: "Fix the broken test in src/auth.test.ts",
  thread: true
})
```

Valid `agent` values (from embeddedHarness config):
- `claude` — Claude Code CLI (primary, always prefer this)
- `codex` — OpenAI Codex CLI
- `copilot` — GitHub Copilot CLI (Claude Opus 4.6)
- `opencode` — OpenCode (Kimi K2 Turbo)
- `gemini` — Gemini CLI
- `kimi` — Kimi CLI
- `cursor` — Cursor agent

## Checking session status

Use `session_status` or `subagents` with the **returned** `childSessionKey`, never a made-up key:

```
session_status({ sessionKey: "<childSessionKey from spawn>" })
subagents({ action: "list" })
```

## Waiting for results

**Preferred:** Use `sessions_yield` after spawning to wait for completion:

```
sessions_spawn({ runtime: "acp", agent: "claude", message: "..." })
sessions_yield()
// Next message will be the completion result
```

**Alternative:** Fire-and-forget with `sessions_send`:

```
sessions_send({
  sessionKey: "<childSessionKey>",
  message: "How's it going?",
  timeoutSeconds: 30
})
```

## Do NOT

- Do NOT make up session keys like `"acp-codex-test"` — always use keys returned by `sessions_spawn`
- Do NOT poll `session_status` in a loop — use `sessions_yield` instead
- Do NOT call `session_status` on a session that was never started
