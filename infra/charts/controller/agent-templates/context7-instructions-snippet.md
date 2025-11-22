# Context7 Documentation Tools

You have access to **Context7** for real-time, up-to-date library documentation.

## Available Tools

1. **`context7_resolve_library_id`** - Resolves library names to Context7 IDs
2. **`context7_get_library_docs`** - Fetches documentation using the library ID

## Usage Workflow

**Step 1: Resolve the library**
```
resolve_library_id({ libraryName: "tokio" })
```
Returns multiple options with scores. Choose based on:
- Benchmark Score (higher is better)
- Source Reputation (High is best)
- Code Snippets count (more is better)

**Step 2: Get documentation**
```
get_library_docs({
  context7CompatibleLibraryID: "/websites/rs_tokio_tokio",
  topic: "async runtime setup and basic usage"
})
```

## Best Practices

✅ **DO:**
- Always resolve the library name first
- Choose libraries with high scores and reputation
- Be specific in your topic queries
- Use Context7 before implementing unfamiliar features

❌ **DON'T:**
- Skip the resolve step
- Use vague topics
- Query for general programming concepts

## Language-Specific Examples

**Rust:**
```
resolve_library_id({ libraryName: "tokio" })
get_library_docs({
  context7CompatibleLibraryID: "/websites/rs_tokio_tokio",
  topic: "async runtime patterns"
})
```

**TypeScript:**
```
resolve_library_id({ libraryName: "react" })
get_library_docs({
  context7CompatibleLibraryID: "/facebook/react",
  topic: "hooks with TypeScript"
})
```

Remember: Context7 provides **current** documentation. Use it whenever you need up-to-date library information!

