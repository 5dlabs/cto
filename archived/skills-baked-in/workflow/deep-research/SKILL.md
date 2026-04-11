---
name: deep-research
description: Deep technical research using Firecrawl Agent for autonomous web investigation, competitive analysis, and implementation pattern discovery.
agents: [morgan, cleo, rex, nova, blaze, grizz]
triggers: [research, competitive, examples, how do others, best practices, like X, similar to, compare, industry standard]
globs:
  - "**/prd*.md"
  - "**/prd*.txt"
  - "**/intake/**"
  - "**/docs/**"
---

# Deep Research Skill

Perform comprehensive technical research using the Firecrawl Agent API for autonomous web investigation. Use this skill when tasks require understanding external patterns, competitive analysis, or finding implementation examples.

## When to Trigger Deep Research

Scan for these patterns in PRDs and task requirements:

| Pattern | Example | Research Action |
|---------|---------|-----------------|
| "like X" references | "authentication like Auth0" | Research how Auth0 implements it |
| "similar to" comparisons | "similar to Stripe webhooks" | Study Stripe's webhook patterns |
| Competitive mentions | "compete with Notion" | Analyze Notion's architecture |
| Best practices requests | "follow industry standards" | Survey how leaders solve it |
| Unfamiliar tech | "use CRDT for sync" | Find CRDT implementation examples |
| "how do others" questions | "how do others handle this?" | Multi-source investigation |

## Research Protocol

### Step 1: Identify Research Needs

Before generating tasks, scan the PRD for:

1. **External references** - Named products, services, or standards
2. **Comparative requirements** - "better than", "like", "similar to"
3. **Technical unknowns** - Unfamiliar patterns or technologies
4. **Best practice requests** - "industry standard", "production-ready"

### Step 2: Choose the Right Tool

| Research Type | Tool | Why |
|---------------|------|-----|
| **Competitive analysis** | `firecrawl_agent` | Multi-site autonomous research |
| **Implementation patterns** | `octocode_githubSearchCode` | Searches actual production code across GitHub |
| **Library documentation** | `context7` | Official, structured docs |
| **Code examples from GitHub** | `octocode_githubSearchCode` | Real production code with semantic search |
| **How major projects solve X** | `octocode_githubSearchRepositories` | Find reference implementations |
| **PR discussions/fixes** | `octocode_githubSearchPullRequests` | Learn how issues were resolved |
| **Specific page content** | `firecrawl_scrape` | Known URL, faster |

### Step 3: Execute Research

#### Using Firecrawl Agent

```
firecrawl_agent({
  prompt: "YOUR RESEARCH QUESTION - be specific",
  schema: {
    "type": "object",
    "properties": {
      "findings": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "source": { "type": "string" },
            "approach": { "type": "string" },
            "details": { "type": "string" },
            "tradeoffs": { "type": "string" }
          }
        }
      },
      "recommendation": { "type": "string" }
    }
  }
})
```

### Step 4: Structure Output

Always format research findings as:

```markdown
## Research: [Topic]

### Summary
[2-3 sentence key takeaway]

### Findings

| Source | Approach | Key Details |
|--------|----------|-------------|
| Auth0 | JWT + refresh rotation | 15min access, 7d refresh |
| Clerk | Session tokens | Server-side validation |

### Recommendation
[How this applies to the current task]

### Sources
- [URL 1] - Description
- [URL 2] - Description
```

## Common Research Patterns

### Competitive Analysis

When PRD mentions competitors or "like X":

```
firecrawl_agent({
  prompt: "Compare how [Competitor A], [Competitor B], and [Competitor C] implement [feature]. Focus on [specific aspects from PRD].",
  schema: {
    "type": "object",
    "properties": {
      "providers": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "name": { "type": "string" },
            "approach": { "type": "string" },
            "strengths": { "type": "string" },
            "weaknesses": { "type": "string" }
          }
        }
      },
      "recommendation": { "type": "string" }
    }
  }
})
```

### Implementation Patterns

When PRD requires unfamiliar technology:

```
firecrawl_agent({
  prompt: "Find production examples of [technology] being used for [use case]. Include code patterns, gotchas, and performance considerations.",
  schema: {
    "type": "object",
    "properties": {
      "examples": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "source": { "type": "string" },
            "pattern": { "type": "string" },
            "code_example": { "type": "string" },
            "gotchas": { "type": "string" }
          }
        }
      }
    }
  }
})
```

### Architecture Research

When designing new systems:

```
firecrawl_agent({
  prompt: "What architectures do major [domain] platforms use for [requirement]? Compare approaches from [Company A], [Company B], etc.",
  schema: {
    "type": "object",
    "properties": {
      "architectures": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "company": { "type": "string" },
            "architecture": { "type": "string" },
            "scale": { "type": "string" },
            "tradeoffs": { "type": "string" }
          }
        }
      }
    }
  }
})
```

### Best Practices

When PRD requests "industry standard" approaches:

```
firecrawl_agent({
  prompt: "What are current best practices for [topic] in [year]? Focus on [specific requirements]. Include examples from production systems."
})
```

## Integrating Research into Tasks

Research findings should be embedded in task `details` fields:

```json
{
  "id": "5",
  "title": "Nova: Implement Refresh Token Rotation",
  "agentHint": "nova",
  "details": "## Requirements\nImplement refresh token rotation for session management.\n\n## Research Findings\nBased on competitive analysis:\n- Auth0: 15min access tokens, 7-day refresh tokens with rotation\n- Clerk: Session-based with server validation\n- Supabase: JWT with configurable expiry\n\n## Recommended Approach\nFollow Auth0 pattern with:\n- 15-minute access token lifetime\n- 7-day refresh token with single-use rotation\n- Revocation on suspicious activity\n\n## Code Signatures\n```typescript\nexport const refreshToken = Effect.gen(function* () {\n  // Implementation based on research\n})\n```"
}
```

## Cost Management

Firecrawl Agent pricing is dynamic. Optimize costs:

1. **Be specific** - Vague prompts cost more
2. **Use schemas** - Structured output reduces processing
3. **Provide URLs when known** - Narrows search scope
4. **Batch related questions** - One comprehensive query vs multiple small ones

## When NOT to Use Deep Research

- **Library docs exist in Context7** - Use `context7` instead
- **You know the exact URL** - Use `firecrawl_scrape`
- **Simple factual lookup** - Use `firecrawl_search`
- **Code examples from GitHub repos** - Use `octocode_githubSearchCode` (semantic search across repos)
- **How React/major OSS projects do X** - Use OctoCode to search their source

## OctoCode Integration

For implementation pattern research, combine Firecrawl (web) with OctoCode (code):

```
# 1. Research how competitors approach the problem (web)
firecrawl_agent({ prompt: "How does Auth0 implement refresh token rotation?" })

# 2. Find actual implementations in open source (code)
octocode_githubSearchCode({
  query: "refresh token rotation",
  language: "typescript",
  stars: ">500"
})

# 3. Get library docs for the chosen approach
context7_get_library_docs({ libraryId: "/better-auth/better-auth", topic: "refresh tokens" })
```

## Research Checklist

Before finalizing research-informed tasks:

- [ ] All "like X" and "similar to" references researched
- [ ] Competitive mentions analyzed
- [ ] Unfamiliar technologies investigated
- [ ] Research findings embedded in relevant task details
- [ ] Sources cited for verification
- [ ] Recommendations align with PRD requirements
