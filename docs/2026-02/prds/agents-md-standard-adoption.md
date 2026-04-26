# PRD: AGENTS.md Standard Adoption

## Summary

Align CTO's agent instruction files with the industry-standard AGENTS.md format adopted by OpenAI Codex, Cursor, Zed, Devin, GitHub Copilot, and 60k+ repositories.

## Problem Statement

CTO's current `AGENTS.md` is a comprehensive 500+ line developer handbook that mixes:
- Agent-specific instructions (build commands, testing)
- Human developer documentation (port forwards, launchd setup, troubleshooting)
- Project-wide reference (MCP tools, workflow diagrams)

This makes it:
1. **Too large** - Agents load unnecessary context, wasting tokens
2. **Inconsistent** with industry standard - External tools (Cursor, Codex) may not parse it optimally
3. **Monolithic** - Doesn't leverage nested AGENTS.md for per-service instructions

## Proposed Solution

### Phase 1: Restructure Root AGENTS.md

Slim down root `AGENTS.md` to agent-essential content:

```markdown
# AGENTS.md

## Build & Test
- `cargo build --release`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`

## Code Style
- Rust: rustfmt (Edition 2021, max_width=100)
- Use `tracing::*` over `println!`

## PR Guidelines
- Conventional Commits: `feat:`, `fix:`, `chore:`
- Run full validation before PR

## Security
- Never commit secrets
- Use 1Password/OpenBao for credentials
```

Move detailed documentation to `docs/DEVELOPMENT.md` or similar.

### Phase 2: Add Nested AGENTS.md Files

Create per-service agent instructions:

```
cto/
├── AGENTS.md                      # Root (slim)
├── tools/
│   └── intake-agent/
│       └── AGENTS.md              # Intake-specific: TypeScript, pnpm
├── services/
│   └── agent-controller/
│       └── AGENTS.md              # Controller-specific: Rust, K8s patterns
└── infra/
    └── AGENTS.md                  # GitOps-specific: Helm, ArgoCD
```

### Phase 3: Add Standard Metadata (Optional)

Consider machine-readable sections for advanced agent tooling:

```markdown
## Testing Instructions
- Run `cargo test` before commits
- Integration tests require K8s context

## Security Considerations  
- No secrets in code
- Use External Secrets operator
```

## Success Criteria

- [ ] Root AGENTS.md reduced to <150 lines
- [ ] 3+ nested AGENTS.md files for major components
- [ ] External coding agents (Cursor, Codex) can parse instructions correctly
- [ ] Token usage reduced when agents load project context

## Effort Estimate

**Low (1 week)**
- Primarily documentation restructuring
- No code changes required
- Can be done incrementally

## References

- Research entry: `1998450638590804043`
- AGENTS.md specification: https://agents.md
- 60k+ examples: https://github.com/search?q=path%3AAGENTS.md

## Approval

- [ ] Approved for implementation
