# ACP Alignment Analysis for CTO Intake Agent

> Analysis of how our multi-agent architecture aligns with Agent Communication Protocol best practices.

## Executive Summary

Our intake-agent implements several ACP patterns implicitly but lacks formal protocol compliance. The debate-planning system is architecturally sound for multi-agent collaboration but could benefit from ACP's standardization for future extensibility.

**Current Status: ~60% aligned with ACP patterns**

---

## ACP Pattern Comparison

### 1. Message Structure ✅ Partial

**ACP Pattern:**
```json
{
  "role": "agent/pessimist",
  "parts": [
    { "content_type": "text/plain", "content": "..." },
    { "name": "/tasks.json", "content_type": "application/json", "content": "..." }
  ]
}
```

**Our Current:**
```typescript
// We use simple strings in prompts and JSON responses
const { text, usage } = await queryProvider(config.providers.proposer, prompt, context);
return parseJSON(text, { solution: '', tasks: [], keyPoints: [] });
```

**Gap:** No structured message format with content types and artifacts.

**Recommendation:**
```typescript
interface ACPMessage {
  role: 'user' | `agent/${string}`;
  parts: Array<{
    content_type: string;
    content?: string;
    content_url?: string;
    name?: string; // For artifacts
  }>;
}
```

---

### 2. Agent Manifest ⚠️ Missing

**ACP Pattern:**
```json
{
  "name": "pessimist",
  "description": "Risk-focused planning agent",
  "input_content_types": ["text/plain", "application/json"],
  "output_content_types": ["application/json"],
  "metadata": {
    "capabilities": ["risk-analysis", "task-generation"],
    "framework": "intake-agent"
  }
}
```

**Our Current:** Agents are functions with prompts, no formal manifest.

**Recommendation:** Define manifests for each virtual agent:
```typescript
const AGENT_MANIFESTS: Record<string, AgentManifest> = {
  pessimist: {
    name: 'pessimist',
    description: 'Identifies risks, failure modes, and conservative estimates',
    input_content_types: ['text/plain'],
    output_content_types: ['application/json'],
    metadata: {
      capabilities: ['risk-analysis', 'dependency-tracking'],
      domainFocus: ['security', 'reliability'],
    }
  },
  optimist: { ... },
  fullstack: { ... },
  synthesizer: { ... },
  critic: { ... },
};
```

---

### 3. Run Lifecycle ⚠️ Missing

**ACP Pattern:**
- States: `created` → `in-progress` → `completed`/`failed`/`cancelled`
- Supports `awaiting` state for human-in-the-loop

**Our Current:** Synchronous execution, no state tracking.

**Gap:** No ability to:
- Track long-running operations
- Resume/cancel mid-execution
- Report progress to callers

**Recommendation for Future:**
```typescript
interface DebateRun {
  id: string;
  status: 'created' | 'in-progress' | 'awaiting' | 'completed' | 'failed';
  currentPhase: 'research' | 'proposals' | 'debate' | 'consensus' | 'critique' | 'remediation';
  progress: number; // 0-100
  partialResults?: Partial<DebatePlanningResult>;
}
```

---

### 4. Composition Patterns ✅ Good

**ACP Patterns & Our Implementation:**

| Pattern | ACP | Our Implementation |
|---------|-----|-------------------|
| **Chaining** | Sequential agent execution | ✅ Research → Proposals → Debate → Consensus → Critique → Remediation |
| **Parallelization** | `asyncio.gather` / `Promise.all` | ✅ Proposals run in parallel, critiques run in parallel |
| **Routing** | Router selects specialist | ✅ Provider selection per role (proposer/critic/synthesizer) |
| **Multi-Agent Collaboration** | Multiple agents debate | ✅ Pessimist vs Optimist vs Fullstack with cross-critique |

**What We Do Well:**
```typescript
// Parallelization - all proposals run concurrently
const results = await Promise.all(
  agents.map(async ({ key, prompt }) => {
    const { text, usage } = await queryProvider(config.providers.proposer, prompt, context);
    // ...
  })
);

// Routing - different providers for different roles
const config = {
  providers: {
    proposer: { provider: 'claude' },
    critic: { provider: 'minimax' },  // Different model for critique
    synthesizer: { provider: 'claude' },
  }
};
```

---

### 5. Sessions/State Management ⚠️ Missing

**ACP Pattern:**
```python
async with client.session() as session:
    run1 = await session.run_sync(agent="echo", input=[...])
    run2 = await session.run_sync(agent="echo", input=[...])  # Has context from run1
```

**Our Current:** Each operation is stateless.

**Gap:** No conversation history between debate rounds, no context carryover.

**Potential Benefit:** Could allow:
- Resuming failed debates mid-way
- Iterative refinement with human feedback
- Multi-turn planning conversations

---

### 6. Error Structure ✅ Good

**ACP Pattern:**
```json
{
  "code": "invalid_input",
  "message": "Missing prd_content in payload"
}
```

**Our Implementation:**
```typescript
function errorResponse(error: string, errorType: ErrorType = 'unknown', details?: string): AgentErrorResponse {
  return {
    success: false,
    error,
    error_type: errorType,  // validation_error, api_error, parse_error, etc.
    ...(details ? { details } : {}),
  };
}
```

**Status:** Well-aligned with ACP error structure.

---

### 7. Multi-Provider Architecture ✅ Excellent

**ACP Vision:** Framework-agnostic, any backend can implement the protocol.

**Our Implementation:**
```typescript
export interface ModelProvider {
  readonly name: ProviderName;
  readonly defaultModel: string;
  isAvailable(): boolean;
  generate(prompt: string, systemPrompt: string, options?: ProviderOptions, model?: string): Promise<ProviderResponse>;
}

// Providers are interchangeable
const providers = new Map<ProviderName, ModelProvider>([
  ['claude', claudeProvider],
  ['minimax', minimaxProvider],
  ['codex', codexProvider],
]);
```

**Status:** Our provider abstraction is excellent and aligns with ACP's philosophy of backend agnosticism.

---

## Recommendations for Scaling to More Agents

### Immediate (Low Effort)

1. **Add Agent Manifests** - Define metadata for each virtual agent
2. **Structured Messages** - Add content_type to responses
3. **Artifact Naming** - Use ACP artifact patterns for task outputs

### Medium-term (Moderate Effort)

4. **Run Tracking** - Add run IDs and state for long operations
5. **Progress Events** - Emit events during debate phases for UI updates
6. **Session Support** - Allow multi-turn refinement

### Long-term (Higher Effort)

7. **ACP Server** - Expose agents via REST endpoints
8. **Agent Discovery** - Implement `/agents` endpoint for manifest listing
9. **Cross-Process Agents** - Allow agents to run as separate services

---

## Code Changes for Basic ACP Alignment

### 1. Add Message Types

```typescript
// src/types/acp.ts
export interface ACPMessagePart {
  content_type: string;
  content?: string;
  content_url?: string;
  content_encoding?: 'plain' | 'base64';
  name?: string; // Artifacts have names
}

export interface ACPMessage {
  role: 'user' | 'agent' | `agent/${string}`;
  parts: ACPMessagePart[];
}

export interface AgentManifest {
  name: string;
  description: string;
  input_content_types: string[];
  output_content_types: string[];
  metadata?: {
    capabilities?: Array<{ name: string; description: string }>;
    domains?: string[];
    tags?: string[];
  };
}
```

### 2. Add Agent Registry

```typescript
// src/agents/registry.ts
export const agentRegistry = new Map<string, AgentManifest>([
  ['pessimist', {
    name: 'pessimist',
    description: 'Risk-focused planning agent that identifies failure modes and conservative estimates',
    input_content_types: ['text/plain', 'application/json'],
    output_content_types: ['application/json'],
    metadata: {
      capabilities: [
        { name: 'Risk Analysis', description: 'Identifies security vulnerabilities and scalability limits' },
        { name: 'Task Generation', description: 'Creates risk-mitigated task lists' },
      ],
      domains: ['security', 'reliability'],
    },
  }],
  ['optimist', { ... }],
  ['fullstack', { ... }],
]);
```

---

## Conclusion

Our debate-planning system implements the **spirit** of ACP (multi-agent collaboration, provider abstraction, composition patterns) without the **letter** (formal message structure, manifests, run lifecycle).

**For 2 agents:** Current architecture is fine.

**For 5+ agents:** Consider:
- Agent manifests for discoverability
- Structured messages for interoperability
- Run lifecycle for long operations

**For external agents:** Would need full ACP compliance (REST server, formal protocol).

The good news: our provider abstraction and composition patterns provide a solid foundation. Adding ACP compliance would be additive, not a rewrite.
