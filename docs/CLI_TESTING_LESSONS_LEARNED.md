# CLI Integration Testing - Lessons Learned

This document captures lessons learned from building and testing the Linear sidecar integration for CLI agents. Use this as a reference when working on additional CLIs.

## Critical Issues & Solutions

### 1. Branch Switching by Other Agents
**Problem:** Other agents switched the git branch, causing loss of uncommitted work.

**Solution:**
- Commit frequently after each milestone
- Always verify current branch before making changes: `git branch --show-current`
- Consider using worktrees for isolation

### 2. Wrong Sidecar Binary
**Problem:** Used `pm::status-sync` instead of `linear-sink::linear-sidecar`.

**Solution:**
- The correct binary is `linear-sidecar` from the `linear-sink` crate
- Located at: `crates/linear-sync/src/bin/linear-sidecar.rs`
- Build with: `cargo build --release -p linear-sink --bin linear-sidecar`

### 3. GraphQL Type Mismatch
**Problem:** Session creation failing with type error.

**Solution:**
```rust
// WRONG:
mutation CreateAgentSession($input: AgentSessionCreateInput!) { ... }

// CORRECT:
mutation CreateAgentSession($input: AgentSessionCreateOnIssue!) {
    agentSessionCreateOnIssue(input: $input) { ... }
}
```

### 4. Skills Parsing Format
**Problem:** Skills in Claude's init message are plain strings, not objects.

**Solution:**
```rust
// Claude sends: "skills": ["skill-name", "another-skill"]
// NOT: "skills": [{"name": "skill-name"}]

// Handle both formats:
if let Some(skills) = value.get("skills").and_then(|v| v.as_array()) {
    for skill in skills {
        if let Some(name) = skill.as_str() {
            // Plain string format
            info.skills.push(name.to_string());
        } else if let Some(name) = skill.get("name").and_then(|n| n.as_str()) {
            // Object format (fallback)
            info.skills.push(name.to_string());
        }
    }
}
```

### 5. Activity Content Types
**Problem:** Init activity not showing in Linear agent dialog.

**Solution:** Use `"response"` type, NOT `"thought"` type:
```rust
// WRONG - thoughts don't render visibly
let content = json!({"type": "thought", "body": body});

// CORRECT - responses render as visible messages
let content = json!({"type": "response", "body": body});
```

### 6. Init Activity Missing After Refactor
**Problem:** The `emit_init_activity()` function was removed during FluentD refactor.

**Solution:** Added `post_init_activity()` to post init summary after session creation:
```rust
async fn post_init_activity(
    state: &AppState,
    session_id: &str,
    model: &str,
    tools: &[String],
    skills: &[String]
) -> Result<()> {
    // Build formatted summary
    let body = format!("🚀 **Agent Initialized**\n\n{}", sections.join("\n"));
    
    // Post as "response" type
    let content = json!({"type": "response", "body": body});
    // ... post to Linear
}
```

## Architecture Notes

### Sidecar Design
- **File watching**: Uses tail-style polling (100ms interval)
- **Session creation**: Creates session on first init event
- **Activity posting**: Posts activities for each log entry type
- **Completion**: Posts "✅ Session completed" on result event

### Content Types for Linear Activities
| Type | Use Case | Visibility |
|------|----------|------------|
| `response` | AI messages, init summary | ✅ Visible |
| `action` | Tool calls with params | ✅ Visible |
| `error` | Error messages | ✅ Visible |
| `thought` | Internal reasoning | ❌ Hidden |

### CLI Output Formats
| CLI | Format | Init Event |
|-----|--------|------------|
| Claude | JSONL stream | `{"type":"system","subtype":"init",...}` |
| Droid/Factory | JSONL stream | `{"type":"tool_call","toolName":"...",...}` |
| Codex | Thread events | `{"type":"thread.started",...}` |

## Test Configuration

### Required Environment Variables
```bash
LINEAR_OAUTH_TOKEN=lin_oauth_...  # From 1Password: op://Automation/Linear Morgan OAuth/developer_token
LINEAR_ISSUE_IDENTIFIER=CTOPA-XXX  # Issue to post to
CTO_AGENT_NAME=rex  # Agent name for config lookup
```

### Running Tests
```bash
cd tests/cli-invocation

# 1. Setup skills + tools for agent
./setup.sh rex coder

# 2. Configure .env
cp .env.example .env
# Edit with secrets

# 3. Build images
./build-images.sh all

# 4. Run test
docker compose up claude claude-sidecar
```

## Deferred Issues

### MCP Tools "0/X used"
The tracking shows 0 tools used even when tools are called. Deferred to bulk testing phase where longer runs make debugging easier.

**Root cause hypothesis:** Tool usage is tracked by name matching, but the format may differ between init message and tool call events.

### Skills "0/X used"
Same issue as MCP tools - tracking shows 0 even when skills are available.

## Key Files Reference

| File | Purpose |
|------|---------|
| `crates/linear-sync/src/bin/linear-sidecar.rs` | Sidecar binary |
| `tests/cli-invocation/docker-compose.yml` | Test services |
| `tests/cli-invocation/setup.sh` | Skills/tools setup |
| `config/cto-config.json` | Agent configurations |

## Commit Checkpoints

Always commit after:
1. ✅ Fixing a bug
2. ✅ Adding new feature
3. ✅ Successful test run
4. ✅ Before switching context

## Verification Checklist

Before considering a test complete, verify in Linear:
- [ ] Session created (check Activity section)
- [ ] Init activity shows Model, Tools, Skills
- [ ] Tool calls appear with parameters
- [ ] AI messages render correctly
- [ ] Completion summary shows stats
