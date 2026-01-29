# Feature Backlog

Feature backlog for the preprocessing pipeline E2E test. Features are implemented by the features-agent during each iteration.

## Priority: High

### FEAT-001: Agent Communication Protocol (ACP) for Inter-Agent Communication

**Status**: OPEN
**Priority**: P0
**Estimated Effort**: 2-3 iterations

**Description**:
Implement the [Agent Communication Protocol (ACP)](https://agentcommunicationprotocol.dev/) - an open standard under the Linux Foundation for agent interoperability. This enables direct communication between swarm subagents via a standardized RESTful API instead of relying solely on the coordinator.

**Benefits**:
- **Latency**: Direct agent-to-agent HTTP communication
- **Reliability**: Reduced single-point-of-failure; fault-tolerant sessions
- **Flexibility**: Agents can be dynamically discovered and composed
- **Interoperability**: Works with any agent framework (BeeAI, LangChain, CrewAI, custom)

**ACP Core Concepts**:
- **REST-based Communication**: Standard HTTP patterns, no special runtime required
- **MIME-based Messages**: Any content type supported (text, images, JSON, binary)
- **Async-first**: Built for long-running agent tasks with sync support
- **Offline Discovery**: Agents discoverable even when inactive via manifests

**REST API Endpoints**:

| Method | Endpoint | Purpose |
|--------|----------|---------|
| `GET` | `/agents` | List all available agents |
| `GET` | `/agents/{name}` | Get agent manifest (capabilities, metadata) |
| `POST` | `/runs` | Create and start a new agent run |
| `GET` | `/runs/{run_id}` | Get run status and output |
| `POST` | `/runs/{run_id}` | Resume an awaiting run |
| `POST` | `/runs/{run_id}/cancel` | Cancel a run |
| `GET` | `/runs/{run_id}/events` | Stream run events |
| `GET` | `/sessions/{session_id}` | Get session descriptor |

**Run Lifecycle States**:

```
created → in-progress → completed
                     → failed
                     → awaiting → (resumed) → in-progress
                     → cancelling → cancelled
```

| State | Description |
|-------|-------------|
| `created` | Run request accepted, processing not started |
| `in-progress` | Agent actively processing |
| `awaiting` | Paused, waiting for client input |
| `completed` | Successfully finished |
| `failed` | Encountered error |
| `cancelling` | Cancellation in progress |
| `cancelled` | Successfully cancelled |

**Message Structure**:

```json
{
  "role": "user | agent | agent/{name}",
  "parts": [
    {
      "content_type": "text/plain",
      "content": "Hello, world!",
      "content_encoding": "plain | base64",
      "content_url": "https://...",
      "name": "artifact-name",
      "metadata": { "kind": "citation", ... }
    }
  ],
  "created_at": "2025-01-28T12:00:00Z"
}
```

**Agent Manifest Schema**:

```json
{
  "name": "my-agent",
  "description": "Agent description",
  "input_content_types": ["text/plain", "application/json"],
  "output_content_types": ["text/plain"],
  "metadata": {
    "framework": "BeeAI",
    "capabilities": [...],
    "tags": ["chat", "research"]
  }
}
```

**Implementation Requirements**:

1. **ACP Server** - Rust/Axum HTTP server exposing ACP endpoints
   - Agent registration and discovery
   - Run lifecycle management
   - Session state management (distributed sessions via URL references)
   
2. **ACP Client** - Client library for inter-agent communication
   - `run_sync()` - Synchronous execution
   - `run_async()` - Async with polling
   - `run_stream()` - Streaming execution
   
3. **Agent Registry** - Peer discovery via `/agents` endpoint
   - Agent manifest publishing
   - Health checking
   
4. **Session Management** - Distributed sessions without shared infrastructure
   - History stored as URL references
   - Cross-server session continuity
   
5. **MCP Integration** - Extend existing MCP tools via ACP
   - ACP-to-MCP adapter (see [MCP Adapter docs](https://agentcommunicationprotocol.dev/integrations/mcp-adapter))

**TypeScript Client Example** (for optimistic/pessimistic agent communication):

```typescript
import { Client, Message, MessagePart } from '@anthropic-ai/acp-sdk';

// Create ACP client
const client = new Client({ baseUrl: 'http://localhost:8000' });

// Synchronous run - wait for completion
const run = await client.runSync({
  agent: 'pessimistic-evaluator',
  input: [
    {
      role: 'agent/optimistic',
      parts: [
        {
          contentType: 'application/json',
          content: JSON.stringify({
            proposal: 'Add new feature X',
            confidence: 0.85,
            evidence: ['test passes', 'lint clean']
          })
        }
      ]
    }
  ]
});

console.log(run.output);
```

**Streaming Example** (for real-time evaluation feedback):

```typescript
// Stream events as they happen
for await (const event of client.runStream({
  agent: 'optimistic-proposer',
  input: [{ role: 'user', parts: [{ contentType: 'text/plain', content: taskPrompt }] }]
})) {
  if (event.type === 'message') {
    console.log('Agent output:', event.data);
  } else if (event.type === 'thought') {
    console.log('Agent thinking:', event.data);
  }
}
```

**Inter-Agent Communication Pattern**:

```typescript
// Optimistic agent proposes → Pessimistic agent evaluates → Loop until consensus
async function runConsensusLoop(task: string): Promise<Result> {
  let proposal = await client.runSync({ agent: 'optimistic', input: [makeMessage(task)] });
  
  while (true) {
    const evaluation = await client.runSync({
      agent: 'pessimistic',
      input: [makeMessage(JSON.stringify(proposal.output))]
    });
    
    if (evaluation.output.approved) {
      return proposal.output;
    }
    
    // Pessimistic feedback → Optimistic refines
    proposal = await client.runSync({
      agent: 'optimistic',
      input: [makeMessage(JSON.stringify(evaluation.output.feedback))]
    });
  }
}
```

**Related Documentation**:
- [ACP Welcome](https://agentcommunicationprotocol.dev/introduction/welcome)
- [ACP Quickstart](https://agentcommunicationprotocol.dev/introduction/quickstart)
- [ACP Architecture](https://agentcommunicationprotocol.dev/core-concepts/architecture)
- [Run Lifecycle](https://agentcommunicationprotocol.dev/core-concepts/agent-run-lifecycle)
- [Message Structure](https://agentcommunicationprotocol.dev/core-concepts/message-structure)
- [OpenAPI Spec](https://github.com/i-am-bee/acp/blob/main/docs/spec/openapi.yaml)
- [TypeScript SDK](https://github.com/i-am-bee/acp/tree/main/typescript)

**Implementation Notes**:
- ACP works alongside MCP (MCP = model-to-tools, ACP = agent-to-agent)
- Use TypeScript SDK for swarm agent communication
- Use distributed sessions for fault tolerance across agent restarts
- The `awaiting` state enables human-in-the-loop checkpoints

---

## Priority: Medium

### FEAT-002: Dynamic Agent Scaling

**Status**: OPEN
**Priority**: P1
**Estimated Effort**: 2 iterations

**Description**:
Allow the swarm to dynamically add/remove agents based on workload.

**Requirements**:
1. Agent registration API
2. Workload metrics collection
3. Scaling trigger conditions
4. Agent lifecycle management

---

### FEAT-003: Shared Memory Space

**Status**: OPEN
**Priority**: P1
**Estimated Effort**: 1-2 iterations

**Description**:
Provide agents with a shared memory space for caching intermediate results and avoiding redundant work.

**Requirements**:
1. Key-value store with TTL
2. Namespace isolation per agent
3. Watch机制 for invalidation
4. Persistence layer

---

## Priority: Low

### FEAT-004: Human-in-the-Loop Checkpoints

**Status**: OPEN
**Priority**: P2
**Estimated Effort**: 1 iteration

**Description**:
Pause execution at critical points for human approval.

**Requirements**:
1. Checkpoint annotation in agent prompts
2. Human notification (Slack/email)
3. Approval workflow
4. Resume mechanism

---

### FEAT-005: Enhanced Progress Reporting

**Status**: OPEN
**Priority**: P2
**Estimated Effort**: 1 iteration

**Description**:
Real-time progress dashboard with agent activity visualization.

**Requirements**:
1. WebSocket-based updates
2. Agent timeline view
3. Resource utilization metrics
4. Export capability
