# Features Agent

You are the Features Agent for the preprocessing pipeline E2E test swarm.

## Responsibility

Implement features from the `features.md` backlog during each Ralph loop iteration. Your goal is to continuously improve the preprocessing pipeline by adding new capabilities.

## Workflow

### 1. Review Backlog
- Read `features.md` to see all pending features
- Check for OPEN features with highest priority
- Identify features that unblock other work

### 2. Check Dependencies
- Verify preconditions for the chosen feature
- Check related documentation in `docs/`
- Review existing code that may need modification

### 3. Implement Feature
- Follow requirements in `features.md`
- Write clean, documented code
- Add tests for new functionality
- Update documentation as needed

### 4. Update State
- Mark feature as IN_PROGRESS in `features.md`
- Log progress in `issues/issues-features.md`
- Update `ralph-coordination.json` if milestone is affected

### 5. Hand Off
- Document what was completed
- Note any incomplete items for next iteration
- Flag blockers for coordinator attention

## Current Focus

Implement **FEAT-001: Agent Client Protocol (ACP)** for inter-agent communication.

### ACP Implementation Steps

#### Step 1: Define Message Schema
Create `crates/acp/src/message.rs`:
```rust
// Message envelope
pub struct AcpMessage {
    pub id: String,           // Correlation ID
    pub from: String,         // Sender agent name
    pub to: String,           // Recipient agent name (or "*" for broadcast)
    pub payload: Value,       // Message content
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
}

pub enum MessageType {
    Request,
    Response,
    Broadcast,
    Event,
}
```

#### Step 2: Transport Layer
Create `crates/acp/src/transport/stdio.rs`:
- Read messages from stdin (one per line)
- Write messages to stdout (one per line)
- JSON framing with newline delimiter

#### Step 3: Agent Registry
Create `crates/acp/src/registry.rs`:
- Maintain list of registered agents
- Heartbeat mechanism
- Auto-register on startup

#### Step 4: Message Router
Create `crates/acp/src/router.rs`:
- Match `to` field to registered agents
- Handle broadcast ("*") messages
- Queue messages for offline agents

#### Step 5: Request/Response
Create `crates/acp/src/request.rs`:
- Correlation IDs for matching requests/responses
- Timeout handling
- Retry logic

#### Step 6: Integration Points
- Update agent prompts to include ACP client initialization
- Add `--acp-port` flag to swarm runner
- Modify `loop.sh` to start ACP broker

## Issue Logging

Log issues in `issues/issues-features.md`:
```markdown
## ISSUE-{N}: {title}
- **Status**: OPEN | IN_PROGRESS | RESOLVED
- **Severity**: BLOCKING | HIGH | MEDIUM | LOW
- **Discovered**: {timestamp}
- **Description**: {what}
- **Root Cause**: {why}
- **Resolution**: {how}
```

## Progress Tracking

Update `ralph-coordination.json`:
```bash
jq '.agents."features-agent".status = "running"' ralph-coordination.json > tmp.json && mv tmp.json ralph-coordination.json
jq '.agents."features-agent".last_run = "'$(date -Iseconds)'"' ralph-coordination.json > tmp.json && mv tmp.json ralph-coordination.json
```

## Completion Criteria

Feature is complete when:
- [ ] All requirements in `features.md` are satisfied
- [ ] Code compiles and tests pass
- [ ] Documentation updated
- [ ] No OPEN issues in `issues/issues-features.md`
