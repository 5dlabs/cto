# Rule: Use Context7 for Rust Best Practices

## Summary

Before implementing any significant Rust code, consult Context7 for current documentation on the libraries being used. This ensures idiomatic, up-to-date implementations.

## Two-Step Workflow

**Step 1: Resolve the library**

```javascript
resolve_library_id({ libraryName: "tokio rust" })
```

Select the result with the highest benchmark score and code snippet count.

**Step 2: Get topic-specific documentation**

```javascript
get_library_docs({
  context7CompatibleLibraryID: "/websites/rs_tokio_tokio",
  topic: "async runtime setup and error handling"
})
```

## Essential Rust Libraries

| Library | Context7 ID | Use Case |
|---------|-------------|----------|
| Tokio | `/websites/rs_tokio_tokio` | Async runtime, I/O, networking |
| Serde | `/websites/serde_rs` | Serialization/deserialization |
| Clippy | `/rust-lang/rust-clippy` | Linting, pedantic best practices |
| Anyhow | `/dtolnay/anyhow` | Application error handling |
| Thiserror | `/dtolnay/thiserror` | Custom error types for libraries |
| Axum | Query for current ID | HTTP framework |
| SQLx | Query for current ID | Database access |
| Tracing | `/tokio-rs/tracing` | Structured logging |

## When to Use Context7

**Always query Context7 when:**

- Implementing async code with Tokio
- Setting up error handling (anyhow vs thiserror patterns)
- Using serde with custom types or attributes
- Configuring Clippy lints
- Implementing HTTP handlers with Axum
- Writing database queries with SQLx
- Adding structured logging with tracing

**Query examples:**

```javascript
// Error handling patterns
get_library_docs({
  context7CompatibleLibraryID: "/dtolnay/anyhow",
  topic: "context error handling best practices"
})

// Clippy configuration
get_library_docs({
  context7CompatibleLibraryID: "/rust-lang/rust-clippy",
  topic: "pedantic lints configuration"
})

// Async patterns
get_library_docs({
  context7CompatibleLibraryID: "/websites/rs_tokio_tokio",
  topic: "spawn task cancellation graceful shutdown"
})
```

## Best Practices

1. **Resolve first** - Always resolve library names to get current IDs
2. **Be specific** - Use focused topic queries like "error handling context" not "documentation"
3. **Check versions** - Some libraries have versioned IDs (e.g., `/org/project/v1.0.0`)
4. **High scores win** - Prefer libraries with higher benchmark scores and snippet counts
5. **Single topic** - Query one topic at a time for best results

