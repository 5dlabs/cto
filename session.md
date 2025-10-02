## Session Summary (2024-10-02)

- Reviewed latest git history (maxRetries config change) to regain context.
- Investigated MCP server startup failure in `rust-basic-api`; root cause was outdated global `cto-mcp` binary missing `maxRetries` support.
- Rebuilt MCP server (`cargo build --release`) and replaced `/opt/homebrew/bin/cto-mcp` with the new build to unblock development.
- Reminder: remove exposed GitHub PAT from global MCP config and regenerate a secure token.
