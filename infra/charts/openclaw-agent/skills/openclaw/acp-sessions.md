---
name: acp_sessions
description: Spawn and manage ACP coding sessions (Claude Code, Codex, Copilot, OpenCode)
metadata: {"openclaw": {"always": true}}
---

# ACP Sessions

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
