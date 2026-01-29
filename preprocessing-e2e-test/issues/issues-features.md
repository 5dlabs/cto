# Features Agent Issue Log

Tracking issues for the Features Agent's implementation work.

---

## ISSUE-001: ACP Message Schema Design
- **Status**: OPEN
- **Severity**: HIGH
- **Discovered**: 2026-01-28
- **Description**: Need to finalize the ACP message schema before implementation
- **Root Cause**: Schema design is a prerequisite for all other ACP components
- **Resolution**:
  1. Review existing agent communication patterns
  2. Define schema with input from other agents
  3. Create JSON Schema for validation
  4. Validate against real-world use cases

---

## ISSUE-002: ACP Transport Selection
- **Status**: OPEN
- **Severity**: MEDIUM
- **Discovered**: 2026-01-28
- **Description**: Decide between stdio and WebSocket for initial transport
- **Root Cause**: Multiple viable options with different trade-offs
- **Resolution**:
  - Start with stdio (simplest, no new ports needed)
  - Plan for WebSocket migration in FEAT-001 phase 2
