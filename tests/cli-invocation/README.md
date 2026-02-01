# CLI Invocation Tests

Local Docker-based smoke and integration tests for all CTO agent CLIs and the Linear sidecar.

## Prerequisites

- Docker (or Colima) with buildx
- Pre-built images: `./build-images.sh all`

## Smoke Tests (No Linear)

Run all CLI smoke tests in parallel (version checks, no API keys required):

```bash
docker compose up
```

Run a specific CLI:

```bash
docker compose up claude
```

## Integration Test (Linear Agent Dialog)

To stream real activity to a Linear issue, the sidecar **creates the agent session** from an issue. You supply the issue (by ID or identifier) and Morgan's OAuth token from 1Password; the sidecar creates the session and emits activities.

### 1. Get Morgan's token from 1Password

```bash
# Morgan's Linear OAuth access token (Linear Agent Client Secrets)
export LINEAR_OAUTH_TOKEN=$(op read "op://Development/Linear App Morgan/access_token" --reveal)
```

If your 1Password item/section name differs (e.g. "Linear Agent Client Secrets (Rotated YYYY-MM-DD)" with section "Morgan"):

```bash
export LINEAR_OAUTH_TOKEN=$(op read "op://VaultName/Item Name/Morgan/access_token" --reveal)
```

### 2. Set the target issue

Use either the issue **identifier** (e.g. `CTOPA-2620`) or the issue **UUID**:

```bash
# By identifier (sidecar resolves to UUID and creates session)
export LINEAR_ISSUE_IDENTIFIER=CTOPA-2620

# Or by UUID (from Linear issue URL or API)
export LINEAR_ISSUE_ID=<issue-uuid>
```

You do **not** set `LINEAR_SESSION_ID` — the sidecar creates the session via `agentSessionCreateOnIssue` and then emits to it.

### 3. Run Claude + sidecar with Linear

From repo root, with token and issue set:

```bash
cd tests/cli-invocation

# Run Claude to produce stream, then sidecar to parse and emit to Linear
# (Claude needs ANTHROPIC_API_KEY for real responses; sidecar needs LINEAR_OAUTH_TOKEN + issue)
export LINEAR_OAUTH_TOKEN=$(op read "op://Development/Linear App Morgan/access_token" --reveal)
export LINEAR_ISSUE_IDENTIFIER=CTOPA-2620   # or your test issue
./run-with-linear.sh
```

The script **opens the Linear issue in your browser** before the test runs so you can watch the agent dialog update in real time. If your Linear workspace URL uses a different slug (e.g. `mycompany` instead of `5dlabs`), set:

```bash
export LINEAR_WORKSPACE_SLUG=mycompany
```

Or run the sidecar alone against an existing stream file (e.g. from a previous Claude run):

```bash
export LINEAR_OAUTH_TOKEN=$(op read "op://Development/Linear App Morgan/access_token" --reveal)
export LINEAR_ISSUE_IDENTIFIER=CTOPA-2620
export CLI_TYPE=claude
export STREAM_FILE=$(pwd)/workspaces/claude/stream.jsonl

docker run --rm \
  -v "$(pwd)/workspaces/claude:/workspace:ro" \
  -e LINEAR_OAUTH_TOKEN \
  -e LINEAR_ISSUE_IDENTIFIER \
  -e CLI_TYPE \
  -e STREAM_FILE \
  -e RUST_LOG=info \
  cto-linear-sidecar:local
```

The sidecar will:

1. Resolve `LINEAR_ISSUE_IDENTIFIER` to an issue UUID (if needed).
2. Call Linear `agentSessionCreateOnIssue(issueId)` to create the agent session.
3. Emit init summary, streamed activities, and completion summary to that session.

### 4. Verify in Linear

Open the issue in Linear and check the agent dialog: you should see the init summary, streamed thoughts/actions/responses, and the completion summary.

## Environment Reference

| Variable | Purpose |
|----------|---------|
| `LINEAR_SESSION_ID` | Existing agent session ID (skip session creation) |
| `LINEAR_ISSUE_ID` | Issue UUID; sidecar creates session on this issue |
| `LINEAR_ISSUE_IDENTIFIER` | Issue identifier (e.g. CTOPA-2620); resolved to UUID then session created |
| `LINEAR_OAUTH_TOKEN` | OAuth access token (e.g. Morgan's from 1Password) |
| `LINEAR_API_KEY` | Alternative to `LINEAR_OAUTH_TOKEN` (Personal API key) |
| `DRY_RUN=1` | Skip Linear API; log activities only |
