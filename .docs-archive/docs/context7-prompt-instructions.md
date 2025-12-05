# Context7 Tools - Agent Prompt Instructions

**Include this section in agent system prompts to enable effective Context7 usage.**

---

## Context7 Documentation Tools

You have access to **two Context7 tools** that work together:

### 1. `resolve_library_id` - Library Discovery
Converts a library name into a Context7-compatible library ID.

**Example:**
```javascript
resolve_library_id({ libraryName: "tokio" })
// Returns: "/tokio-rs/tokio"
```

### 2. `get_library_docs` - Documentation Retrieval
Fetches **real-time, up-to-date documentation** using a library ID.

**Example:**
```javascript
get_library_docs({
  library_id: "/tokio-rs/tokio",
  query: "async runtime setup and basic usage"
})
```

### Recommended Workflow

**Step 1: Resolve the library name to get the correct ID**
```javascript
resolve_library_id({ libraryName: "tokio" })
```

This returns multiple options with scores. **Choose the best match** based on:
- Name match
- Source Reputation (High is best)
- Benchmark Score (higher is better)
- Code Snippets count (more is better)

Example response:
```
- Title: Tokio
- Context7-compatible library ID: /websites/rs_tokio_tokio
- Code Snippets: 2936
- Source Reputation: High
- Benchmark Score: 93.8
```

**Step 2: Use that library ID to get documentation**
```javascript
get_library_docs({
  context7CompatibleLibraryID: "/websites/rs_tokio_tokio",
  topic: "async runtime setup and basic usage"
})
```

**Note:** The parameter names are:
- `context7CompatibleLibraryID` (not `library_id`)
- `topic` (not `query`)

### Language Specificity

The language is determined by **the library ID itself**:
- `/websites/rs_tokio_tokio` → Rust (Tokio)
- `/facebook/react` → JavaScript/TypeScript (React)
- `/pallets/django` → Python (Django)

Each library is language-specific, so you don't need to specify the language separately.

### When to Use Context7

Use Context7 when you need:
- Current library/framework documentation
- Version-specific API information
- Code examples and usage patterns
- Breaking changes or new features
- Best practices for unfamiliar libraries

### Complete Examples

**Rust - Tokio:**
```javascript
// Step 1: Resolve
resolve_library_id({ libraryName: "tokio" })
// Returns multiple options, choose: /websites/rs_tokio_tokio (Score: 93.8)

// Step 2: Get docs
get_library_docs({
  context7CompatibleLibraryID: "/websites/rs_tokio_tokio",
  topic: "async runtime setup and basic usage"
})
```

**TypeScript - React:**
```javascript
// Step 1: Resolve
resolve_library_id({ libraryName: "react" })
// Choose: /facebook/react

// Step 2: Get docs
get_library_docs({
  context7CompatibleLibraryID: "/facebook/react",
  topic: "hooks useState and useEffect with TypeScript"
})
```

### Best Practices

✅ **DO:**
- Always resolve the library name first to get the correct ID
- Choose libraries with high benchmark scores and reputation
- Be specific in your topic queries
- Focus on one topic per query

❌ **DON'T:**
- Skip the resolve step (you need the exact library ID)
- Use vague topics like "documentation" or "usage"
- Combine multiple unrelated questions in one topic
- Query for general programming concepts (use your knowledge instead)

### Workflow Integration

When implementing a feature with an unfamiliar library:

1. **Resolve the library** to get the correct ID
2. **Get documentation** for your specific need
3. **Implement** based on the documentation
4. **Verify** with tests

Example:
```
I need to implement JWT authentication in Rust.

1. Resolving library:
   resolve_library_id({ libraryName: "jsonwebtoken" })
   // Choose: /keats/jsonwebtoken

2. Getting documentation:
   get_library_docs({
     context7CompatibleLibraryID: "/keats/jsonwebtoken",
     topic: "JWT encoding and decoding"
   })

3. Implementing based on the documentation...

4. Verifying with tests...
```

### Quick Reference

**Two-Step Process:**
```javascript
// Step 1: Resolve
resolve_library_id({ libraryName: "library-name" })

// Step 2: Get docs
get_library_docs({
  context7CompatibleLibraryID: "/org/project",
  topic: "specific feature or topic"
})
```

**Common Libraries:**
- Rust: `tokio`, `serde`, `axum`, `sqlx`
- TypeScript: `react`, `next.js`, `react-query`
- Python: `fastapi`, `pandas`, `pytest`

Remember: Context7 provides **current** documentation. Always use the two-step process!

---

**End of Context7 Instructions**

