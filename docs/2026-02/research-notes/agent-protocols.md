# Agent Communication Protocols Research

*Compiled: 2026-01-31*

## Key Insight

> "ACP connects agents to agents; MCP connects agents to their tools and knowledge."
> — IBM Research

## Protocol Overview

| Protocol | Purpose | Creator | Transport |
|----------|---------|---------|-----------|
| **MCP** | LLM ↔ Tools/Data | Anthropic | JSON-RPC 2.0 |
| **ACP** | Agent ↔ Agent | IBM/BeeAI → Linux Foundation | REST over HTTP |
| **A2A** | Agent ↔ Agent | Google | JSON-RPC 2.0, SSE |

## ACP (Agent Communication Protocol)

**Source**: https://research.ibm.com/blog/agent-communication-protocol-ai

### What It Is
- Open protocol for agent interoperability
- RESTful architecture over HTTP
- Supports sync AND async agent interactions
- No SDK required (curl/Postman work), but Python/TS SDKs available
- Agents carry their own metadata for discovery (even air-gapped)

### Key Features
- **Peer-to-peer**: Agents can interact directly, not just through a "manager"
- **Task lifecycle**: submitted → working → completed
- **Offline discovery**: Metadata embedded in packages
- **Multi-modal**: Any mimetype supported

### ACP vs MCP
```
┌─────────────────────────────────────────────────────────┐
│                    MULTI-AGENT SYSTEM                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│   Agent A ◄──── ACP ────► Agent B                       │
│      │                        │                          │
│      │ MCP                    │ MCP                      │
│      ▼                        ▼                          │
│   [Tools]                  [Tools]                       │
│   [Data]                   [Data]                        │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Example Use Case (from IBM)
A triage agent receives customer complaint → passes to service agent with conversation history → service agent resolves independently. No central manager needed.

## A2A (Agent-to-Agent Protocol)

**Source**: https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/

### What It Is
- Google's agent interoperability protocol
- JSON-RPC 2.0 based
- Structured task states
- SSE (Server-Sent Events) for real-time updates

### Task Lifecycle
```
submitted → working → completed
```

### Example Message
```json
{
  "jsonrpc": "2.0",
  "method": "tasks/send",
  "params": {
    "id": "task-uuid",
    "message": {
      "role": "user",
      "parts": [{ "type": "text", "text": "..." }]
    }
  }
}
```

## Protocol Convergence

- ACP is merging into A2A under Linux Foundation
- Both can coexist with MCP (different layers)
- Expect consolidation as real-world usage increases

## Relevance to CTO

### What We Have
- MCP for tools (already using)
- Ad-hoc HTTP to pod sidecars for agent comms

### What We Need
- Standardized agent-to-agent protocol for Morgan
- Task state synchronization
- Agent discovery
- Real-time status streaming

### Recommendation
Adopt ACP/A2A patterns for Morgan's agent coordination. Start with:
1. Define agent metadata schema (discovery)
2. Implement task lifecycle (submitted/working/completed)
3. Add status streaming (SSE or webhooks)
4. Build Morgan as the coordinator that speaks this protocol

## Resources

- ACP Spec: https://agentcommunicationprotocol.dev/
- ACP GitHub: https://github.com/i-am-bee/acp
- A2A Blog: https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/
- BeeAI Platform: https://beeai.dev/
- IBM Blog: https://research.ibm.com/blog/agent-communication-protocol-ai
