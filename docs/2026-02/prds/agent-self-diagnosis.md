# PRD: Agent Self-Diagnosis via Trace Collection

## Summary

Implement a trace collection system that enables agents to debug their own failures by analyzing execution traces, identifying error patterns, and suggesting remediation strategies. This enables recursive improvement where agents learn from their mistakes.

## Problem Statement

When agents fail, the current state is:
1. **Opaque failures** - "Task failed" with no visibility into why
2. **Manual debugging** - Humans must trace through logs to understand
3. **No learning loop** - Same failures repeat across sessions
4. **Context loss** - Error context is lost between attempts

Agents lack the ability to introspect their own execution and understand "why did I fail?"

## Proposed Solution

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Agent Self-Diagnosis System                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Trace     │  │   Pattern   │  │   Remediation│         │
│  │   Collector │  │   Matcher   │  │   Generator  │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                  │
│         └────────────────┼────────────────┘                  │
│                          ▼                                   │
│                 ┌─────────────────┐                          │
│                 │   Diagnosis     │                          │
│                 │   Engine        │                          │
│                 └────────┬────────┘                          │
│                          │                                   │
│         ┌────────────────┼────────────────┐                  │
│         ▼                ▼                ▼                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
│  │  Agent   │    │  Human   │    │  Memory  │              │
│  │  Prompt  │    │  Report  │    │  Store   │              │
│  └──────────┘    └──────────┘    └──────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### Trace Collection

Agents emit structured traces during execution:

```typescript
interface ExecutionTrace {
  trace_id: string;
  agent_id: string;
  task_id: string;
  timestamp: number;
  events: TraceEvent[];
  outcome: 'success' | 'failure' | 'timeout';
  context: {
    model: string;
    temperature: number;
    tools_used: string[];
    input_size: number;
    output_size: number;
  };
}

interface TraceEvent {
  timestamp: number;
  type: 'thought' | 'tool_call' | 'tool_result' | 'decision' | 'error';
  data: Record<string, unknown>;
  duration_ms?: number;
}
```

### Pattern Detection

The system identifies common failure patterns:

| Pattern | Description | Example |
|---------|-------------|---------|
| `tool_timeout` | Tool taking too long | curl timeout after 30s |
| `context_overflow` | Context window exceeded | Prompt too long for model |
| `permission_denied` | Access denied | File/API permission error |
| `schema_mismatch` | Data format error | Invalid JSON response |
| `recursive_loop` | Agent stuck in loop | Repeating same tool call |
| `hallucination` | Agent made up info | Non-existent file/function |

### Remediation Suggestions

When a pattern is detected, suggest fixes:

```
Pattern: tool_timeout
Suggestion: "The curl tool timed out. Try:
1. Increase timeout with timeout_ms parameter
2. Break request into smaller chunks
3. Use streaming response for large data"
```

## Technical Implementation

### Trace Storage

- Store traces in OpenSearch/Elasticsearch for querying
- Partition by agent_id, date, outcome
- Enable full-text search on trace content

### Pattern Matching Engine

```rust
struct PatternMatcher {
    patterns: Vec<FailurePattern>,
    matcher: Regex,
}

impl PatternMatcher {
    fn analyze(&self, trace: &ExecutionTrace) -> Vec<MatchResult> {
        // Check each pattern against trace events
        // Return matching patterns with confidence scores
    }
}
```

### Diagnosis API

```typescript
interface DiagnosisAPI {
  submitTrace(trace: ExecutionTrace): void;
  diagnose(traceId: string): DiagnosisResult;
  getRemediation(pattern: string): RemediationSuggestion;
  getCommonFailures(agentId: string): FailureStats[];
}
```

## Success Criteria

- [ ] All agents emit structured traces during execution
- [ ] Trace collection has <1% overhead on agent performance
- [ ] Pattern detection accuracy >80% for top 10 failure types
- [ ] Remediation suggestions resolve >50% of matched failures
- [ ] Agents show measurable improvement in repeat scenarios

## Effort Estimate

**Medium-High (4-5 weeks)**
- Week 1-2: Trace collection infrastructure, SDK
- Week 3: Pattern detection engine
- Week 4: Remediation suggestion system
- Week 5: Integration with all agents, testing

## Open Questions

- How long to retain traces? (Cost vs value tradeoff)
- Should traces include full prompt/output or just metadata?
- How to handle sensitive data in traces?

## References

- OpenTelemetry traces: https://opentelemetry.io/docs/concepts/signals/traces/
- Claude trace research: https://twitter.com/manthanguptaa/status/1998615541574152257
