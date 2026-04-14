# Archived Crates

These crates were moved out of the active workspace during the public release cleanup (2026-04-14). They are preserved for reference but are not built, tested, or maintained.

| Crate | Reason | Replacement |
|-------|--------|-------------|
| `gpu` | Single-provider (Latitude.sh), 1 test, minimal coverage | Use `crates/metal` for infrastructure provisioning |
| `dex-indexer` | Solana-specific DEX swap indexer, 0 tests, niche domain | None — may be revived as a standalone project |
| `mcp-lite` | Experimental desktop MCP server, 0 tests | Use `crates/mcp` for MCP functionality |
| `pm-lite` | Early desktop PM integration, 2 tests | Use `crates/pm` for PM functionality |
| `cto-lite/pm-lite-server` | Early desktop PM server, 2 tests | Use `crates/pm` for PM functionality |

To restore a crate, move it back to `crates/` and add it to the workspace `members` in the root `Cargo.toml`.
