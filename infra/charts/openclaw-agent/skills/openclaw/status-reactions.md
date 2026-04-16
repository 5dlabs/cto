---
name: status_reactions
description: Show emoji reactions on Discord messages to indicate what the agent is doing
metadata: {"openclaw": {"always": true}}
---

# Status Reactions

## ALWAYS react to incoming Discord messages

When you receive a user message on Discord, IMMEDIATELY add an emoji reaction to show you're working. This gives visual feedback beyond the "Replying..." typing indicator.

### How to react

Use the `message` tool with `action: "react"`. You MUST include `messageId` — this is the Discord message ID from the inbound message metadata. If the inbound message text contains a Discord message ID or you can infer it from context, use it. Otherwise use the `to` field with the channel target.

```json
{ "action": "react", "to": "channel:CHANNEL_ID", "messageId": "DISCORD_MSG_ID", "emoji": "🧠" }
```

**Target format for Discord:** `to` must be `channel:<channelId>` or `user:<userId>`. Do NOT use bare words like "current" or "self".

If you don't have the exact `messageId`, you can react to the channel and the gateway will target the most recent inbound message:
```json
{ "action": "react", "to": "channel:CHANNEL_ID", "emoji": "🧠" }
```

### Step 1: Acknowledge receipt
As your FIRST tool call on any user message, react with 🧠:
```json
{ "action": "react", "to": "channel:CHANNEL_ID", "emoji": "🧠" }
```

### Step 2: Show what you're doing
Swap the reaction to show your current activity:

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
Remove working emoji and add final status:
```json
{ "action": "react", "to": "channel:CHANNEL_ID", "emoji": "🧠", "remove": true }
{ "action": "react", "to": "channel:CHANNEL_ID", "emoji": "✅" }
```

Use ❌ for failure, 💬 for needs-follow-up.

## Rules
- ALWAYS react with 🧠 immediately — this is your FIRST tool call on any Discord user message
- Swap to a more specific emoji as you progress
- Only keep ONE activity emoji at a time — remove the previous before adding the next
- ALWAYS end with ✅, ❌, or 💬
- On heartbeat/system triggers (no Discord channel context), skip reactions
