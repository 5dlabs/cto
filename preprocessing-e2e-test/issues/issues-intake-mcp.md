# Issues Log: intake-mcp

## ISSUE-1: cto-mcp binary schema mismatch
- **Status**: OPEN
- **Severity**: BLOCKING
- **Discovered**: 2026-01-28T20:03:00-08:00
- **Updated**: 2026-01-28T21:30:00-08:00
- **Description**: The cto-mcp CLI tool fails to parse cto-config.json with error "missing field `tools` at line 282 column 11".
- **Root Cause**: The cto-mcp binary (built: 2026-01-27T10:47:06-0800) was compiled with a different config schema than the current cto-config.json structure. The binary expects a root-level `tools` field in a specific location that doesn't match the current config structure.
  - Verified: cto-config.json is valid JSON with root-level `tools` field present
  - Verified: The binary loads the correct config file at /Users/jonathonfritz/cto-e2e-testing/cto-config.json
  - Root cause: Binary schema mismatch - needs recompilation with updated config types
- **Affected Binary**: ~/.local/bin/cto-mcp (Mach-O 64-bit executable arm64)
- **Expected Behavior**: The cto-mcp intake tool should:
  1. Create a Linear project named "AlertHub-Preprocessing-Test"
  2. Create a PRD issue with content from test-data/prd.md
  3. Attach architecture document from test-data/architecture.md
  4. Add research documents (effect-ts, grpc-patterns, resources)
  5. Create cto-config.json document in Linear
  6. Sync ConfigMap to Kubernetes
- **Current Workaround**: None available without rebuilding cto-mcp binary with updated config schema
- **Resolution Required**:
  1. Update the cto-mcp binary's config schema to match current cto-config.json structure
  2. OR use an alternative method for Linear project creation (e.g., Linear API directly)
