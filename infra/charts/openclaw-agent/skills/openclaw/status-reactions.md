---
name: status_reactions
description: Show emoji reactions on Discord messages to indicate what the agent is doing
metadata: {"openclaw": {"always": true}}
---

# Status Reactions

## ALWAYS react to incoming Discord messages

When you receive a user message on Discord, IMMEDIATELY add an emoji reaction to show you're working. This gives visual feedback beyond the "Replying..." typing indicator.

### How to react

Use the `message` tool with `action: "react"`. The gateway automatically targets the current inbound message — you do NOT need to provide `messageId` or `to`.

```json
{ "action": "react", "emoji": "🧠" }
```

### Step 1: Acknowledge receipt
As your VERY FIRST tool call on any user message, react with 🧠:
```json
{ "action": "react", "emoji": "🧠" }
```
This MUST be the first tool call you make — before exec, read, web, or any other tool.

### Step 2: Show what you're doing
As you work, swap the reaction to show your current activity:

| Activity | Emoji | When to use |
|----------|-------|-------------|
| Thinking / planning | 🧠 | Initial receipt, analyzing the request |
| Reading code / files | 🔍 | Searching codebase, reading files |
| Writing code | ✏️ | Editing files, writing implementations |
| Running commands | ⚙️ | Running builds, tests, CLI commands |
| Browsing web | 🌐 | Fetching URLs, web research |
| ACP session active | 🔄 | Delegated to a coding CLI (see acp-sessions skill) |
| Waiting on external | ⏳ | Waiting for CI, API response, deployment |

To swap: remove the old emoji, then add the new one:
```json
{ "action": "react", "emoji": "🧠", "remove": true }
{ "action": "react", "emoji": "🔍" }
```

### Step 3: Final status
Remove working emoji and add final status:
```json
{ "action": "react", "emoji": "🔍", "remove": true }
{ "action": "react", "emoji": "✅" }
```

Use ❌ for failure, 💬 for needs-follow-up.

## Rules
- ALWAYS react with 🧠 immediately — this MUST be your FIRST tool call on any Discord user message
- Swap to a more specific emoji as you progress
- Only keep ONE **activity** emoji at a time (🧠/🔍/✏️/⚙️/🌐/🔄/⏳) — remove the previous before adding the next
- **Exception:** backend-identity color dots (🟣🟢🔵🟠🔴⚪ from the `acp_sessions` skill) are **stackable**
  and live alongside the activity emoji. They follow their own add/remove rules — see that skill.
- ALWAYS end with ✅, ❌, or 💬
- On heartbeat/system triggers (no inbound message context), skip reactions — they will fail
