# Linear Agent API Reference (scraped 2026-03-28)

## Agent Sessions
- Tracks lifecycle of agent run
- States: `pending`, `active`, `error`, `awaitingInput`, `complete`, `stale`
- Auto-created when agent is @mentioned or delegated an issue
- Must respond with activity within **10 seconds** or marked unresponsive
- Sessions go stale after **30 minutes** of inactivity (recoverable)
- `externalUrls` — link to external dashboard/PR
- Proactive creation: `agentSessionCreateOnIssue`, `agentSessionCreateOnComment`

## Agent Activities (5 types)
1. **thought** — internal note: `{type: "thought", body: "..."}`
2. **elicitation** — request user input: `{type: "elicitation", body: "..."}`
3. **action** — tool invocation: `{type: "action", action: "Searching", parameter: "...", result?: "..."}`
4. **response** — final result: `{type: "response", body: "..."}`
5. **error** — failure: `{type: "error", body: "..."}`
- Ephemeral activities: temporary, replaced by next activity (thought/action only)
- Markdown supported in body fields
- Linear URLs in markdown auto-render as mentions

## Agent Plans (tech preview)
- Session-level checklist via `agentSessionUpdate` mutation
- Steps: `{content: string, status: "pending" | "inProgress" | "completed" | "canceled"}`
- Full array replacement (cannot update single item)

## Signals
- **stop** (human→agent): halt immediately, emit response/error
- **auth** (agent→human): request account linking with URL
- **select** (agent→human): present options list for elicitation
  - `signalMetadata.options: [{label, value}, ...]`
  - User can pick or reply in free text

## Session Webhooks
- `created` — new session (mention/delegation). Use `promptContext` for formatted context.
- `prompted` — user sent new message. Body in `agentActivity.body`.
- Must return within 5 seconds

## Best Practices
- Emit thought within 10s of receiving created event
- Move issue to first "started" status
- Set self as delegate if none set
- Use Agent Activities (not comments) for conversation history
- Support inbox notifications + permission change webhooks

## GraphQL Mutations
- `agentActivityCreate(input: {agentSessionId, content: {...}, signal?, signalMetadata?})`
- `agentSessionUpdate(id, input: {plan: [...], externalUrls: [...]})`
- `agentSessionCreateOnIssue(input: {issueId})`
- `agentSessionCreateOnComment(input: {commentId})`
