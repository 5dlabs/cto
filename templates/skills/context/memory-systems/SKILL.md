---
name: memory-systems
description: Agent memory architecture including working, short-term, long-term, and temporal knowledge graphs.
agents: [rex, grizz, nova, blaze, tap, spark, bolt, cleo, cipher, tess, atlas, morgan]
triggers: [memory, persistence, state, knowledge graph, entity tracking, temporal]
---

# Memory System Design

Memory provides the persistence layer that allows agents to maintain continuity across sessions and reason over accumulated knowledge. Simple agents rely entirely on context for memory, losing all state when sessions end. Sophisticated agents implement layered memory architectures that balance immediate context needs with long-term knowledge retention.

## When to Activate

Activate this skill when:
- Building agents that must persist across sessions
- Needing to maintain entity consistency across conversations
- Implementing reasoning over accumulated knowledge
- Designing systems that learn from past interactions
- Creating knowledge bases that grow over time
- Building temporal-aware systems that track state changes

## Core Concepts

Memory exists on a spectrum from immediate context to permanent storage. At one extreme, working memory in the context window provides zero-latency access but vanishes when sessions end. At the other extreme, permanent storage persists indefinitely but requires retrieval to enter context.

Simple vector stores lack relationship and temporal structure. Knowledge graphs preserve relationships for reasoning. Temporal knowledge graphs add validity periods for time-aware queries.

## Memory Architecture Fundamentals

### The Context-Memory Spectrum

| Layer | Latency | Persistence | Capacity |
|-------|---------|-------------|----------|
| Working memory | Zero | Volatile | Limited |
| Short-term memory | Low | Session | Medium |
| Long-term memory | Medium | Permanent | Large |
| Permanent memory | High | Archival | Unlimited |

### Why Simple Vector Stores Fall Short

Vector RAG provides semantic retrieval but lacks structure for agent memory:
- Loses relationship information between entities
- No mechanism to distinguish current vs outdated facts
- Cannot answer traversal queries

### Benchmark Performance Comparison

| Memory System | Accuracy | Retrieval Latency | Notes |
|---------------|----------|-------------------|-------|
| Temporal KG | 94.8% | 2.58s | Best accuracy, fast retrieval |
| Knowledge Graph | ~75-85% | Variable | 20-35% gains over baseline |
| Vector RAG | ~60-70% | Fast | Loses relationship structure |
| Recursive Summarization | 35.3% | Low | Severe information loss |

## Memory Layer Architecture

### Layer 1: Working Memory

Working memory is the context window itself. It provides immediate access but has limited capacity and vanishes when sessions end.

Usage patterns:
- Scratchpad calculations
- Conversation history
- Current task state
- Active retrieved documents

### Layer 2: Short-Term Memory

Short-term memory persists across the current session but not across sessions.

Common implementations:
- Session-scoped databases
- File-system storage in session directories
- In-memory caches keyed by session ID

### Layer 3: Long-Term Memory

Long-term memory persists across sessions indefinitely, enabling agents to learn from past interactions.

Use cases:
- Learning user preferences
- Building domain knowledge bases
- Maintaining entity registries
- Storing successful patterns

### Layer 4: Entity Memory

Entity memory tracks information about entities (people, places, concepts, objects) to maintain consistency.

Key functions:
- Entity identity tracking
- Entity property storage
- Entity relationship tracking

### Layer 5: Temporal Knowledge Graphs

Temporal knowledge graphs extend entity memory with explicit validity periods. Facts are not just true or false but true during specific time ranges.

```python
def query_address_at_time(user_id, query_time):
    return temporal_graph.query("""
        MATCH (user)-[r:LIVES_AT]->(address)
        WHERE user.id = $user_id
        AND r.valid_from <= $query_time
        AND (r.valid_until IS NULL OR r.valid_until > $query_time)
        RETURN address
    """, {"user_id": user_id, "query_time": query_time})
```

## Memory Implementation Patterns

### Pattern 1: File-System-as-Memory

The file system itself can serve as a memory layer. Simple, requires no additional infrastructure.

```
/memory/
├── entities/
│   └── user-123.json
├── facts/
│   └── 2024-01-15-preferences.json
└── sessions/
    └── session-abc.json
```

### Pattern 2: Vector RAG with Metadata

Vector stores enhanced with rich metadata provide semantic search with filtering.

Embed facts with metadata including:
- Entity tags
- Temporal validity
- Source attribution
- Confidence scores

### Pattern 3: Knowledge Graph

Knowledge graphs explicitly model entities and relationships.

Define entity types and relationship types, use graph database or property graph storage, maintain indexes for common query patterns.

## Memory Consolidation

Memories accumulate over time and require consolidation:

**Consolidation Triggers**
- Significant memory accumulation
- Too many outdated results in retrieval
- Periodic schedule
- Explicit consolidation request

**Consolidation Process**
1. Identify outdated facts
2. Merge related facts
3. Update validity periods
4. Archive or delete obsolete facts
5. Rebuild indexes

## Guidelines

1. Match memory architecture to query requirements
2. Implement progressive disclosure for memory access
3. Use temporal validity to prevent outdated information conflicts
4. Consolidate memories periodically to prevent unbounded growth
5. Design for memory retrieval failures gracefully
6. Consider privacy implications of persistent memory
7. Implement backup and recovery for critical memories
8. Monitor memory growth and performance over time
