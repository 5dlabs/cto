---
name: status_reactions
description: Show emoji reactions on Discord messages to indicate what the agent is doing
metadata: {"openclaw": {"always": true}}
---

# Status Reactions

## ALWAYS react to incoming messages

When you receive a message on Discord, IMMEDIATELY add a reaction to show you're working on it. This gives visual feedback beyond the "Replying..." typing indicator.

### Step 1: Acknowledge receipt
As your FIRST action on any user message, react with 🧠 (thinking):
```
message({ action: "react", messageId: "<incoming-message-id>", emoji: "🧠" })
```

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

### Step 3: Final status
When done, clean up working emojis and add the final status:
```
// Success
message({ action: "react", messageId: "<message-id>", emoji: "🧠", remove: true })
message({ action: "react", messageId: "<message-id>", emoji: "✅" })

// Failure / error
message({ action: "react", messageId: "<message-id>", emoji: "🧠", remove: true })
message({ action: "react", messageId: "<message-id>", emoji: "❌" })

// Partial / needs follow-up
message({ action: "react", messageId: "<message-id>", emoji: "🧠", remove: true })
message({ action: "react", messageId: "<message-id>", emoji: "💬" })
```

## Rules
- ALWAYS react with 🧠 immediately on message receipt — do this before any other work
- Swap to a more specific emoji as you progress (e.g. 🔍 when searching, ✏️ when editing)
- Only keep ONE activity emoji at a time — remove the previous before adding the next
- ALWAYS end with ✅ (success), ❌ (failure), or 💬 (needs follow-up)
- On heartbeat/system triggers, skip reactions (no messageId to react to)
