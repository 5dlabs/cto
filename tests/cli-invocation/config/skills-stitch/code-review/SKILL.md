---
name: code-review
description: Code quality review patterns including quality gates, checklists, and language-specific checks.
agents: [cleo]
triggers: [quality, review, lint, coverage, code smell]
---

# Code Quality Review

Quality analysis patterns for maintaining healthy, maintainable codebases.

## Execution Rules

1. **Quality gates.** All checks must pass before approval
2. **Constructive feedback.** Be specific and actionable
3. **Test coverage.** Aim for 80%+, 100% on critical paths
4. **Documentation.** Code should be self-documenting with good names
5. **Consistency.** Follow project conventions

## Review Checklist

### Code Quality

- [ ] Clear, meaningful names
- [ ] Small, focused functions (< 40 lines)
- [ ] No code duplication (DRY)
- [ ] Proper error handling
- [ ] No magic numbers/strings

### Testing

- [ ] Unit tests for logic
- [ ] Integration tests for workflows
- [ ] Edge cases covered
- [ ] Mocks used appropriately

### Security

- [ ] No secrets in code
- [ ] Input validation
- [ ] Output encoding
- [ ] Auth/authz checks

### Performance

- [ ] No N+1 queries
- [ ] Appropriate caching
- [ ] Efficient algorithms

## Language-Specific Checks

### Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic
cargo test --workspace
cargo tarpaulin --out Html  # Coverage
```

**Rust-Specific:**
- Verify `#[must_use]` attributes on functions returning values
- Check for proper error handling with `anyhow`/`thiserror`
- Ensure no `unwrap()` in production code paths
- Verify `tracing` macros used instead of `println!`
- Check clippy pedantic lints are satisfied

### TypeScript

```bash
pnpm lint
pnpm typecheck || npx tsc --noEmit
pnpm test --coverage
pnpm build
```

**Effect-Specific:**
- Verify `Effect.Schema` is used for validation (not Zod)
- Check that errors use `Schema.TaggedError` for type safety
- Ensure services use `Context.Tag` for dependency injection
- Verify `Effect.retry` uses proper `Schedule` patterns
- Check that `Effect.gen` is used for complex pipelines

**React/Next.js:**
- Verify proper use of `use client` / `use server` directives
- Check for proper error boundaries
- Ensure accessibility attributes present

### Go

```bash
go fmt ./...
golangci-lint run
go test ./... -cover
go vet ./...
```

**Go-Specific:**
- Verify proper error handling (no ignored errors)
- Check for goroutine leaks
- Ensure context propagation
- Verify interface segregation

## Complexity Analysis

```bash
# Line counts by language
tokei .

# Check complexity
scc --complexity .
```

## Quality Guidelines

- Follow project style guide
- Keep functions small and focused (< 40 lines)
- Use meaningful names
- Write self-documenting code
- Maintain high test coverage
- Address tech debt incrementally

## Definition of Done

Before approving:

- All quality checks pass (lint, format, type check)
- Test coverage meets project threshold
- No critical code smells or complexity issues
- Documentation is complete and accurate
- Review comments have been addressed
- Changes follow project conventions
