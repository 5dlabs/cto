# PRD: agent-browser for Visual Testing

## Summary

Replace or supplement Playwright MCP with Vercel's `agent-browser` for faster, AI-optimized browser automation in Blaze and Tess workflows.

## Problem Statement

Current visual testing with Playwright MCP has limitations:
1. **Slow** - Full Playwright instance for each operation
2. **Not AI-optimized** - Returns DOM/selectors that LLMs struggle to parse reliably
3. **Verbose** - Agents must construct complex selectors
4. **Token-heavy** - DOM dumps consume significant context

Blaze needs to:
- Capture screenshots for PR validation
- Verify UI renders correctly across viewports
- Test interactive states (hover, focus, loading)

## Proposed Solution

### Adopt agent-browser

**agent-browser** is Vercel's Playwright alternative specifically designed for AI agents:

| Feature | Playwright MCP | agent-browser |
|---------|----------------|---------------|
| Speed | Slower (full browser per op) | Fast (persistent daemon) |
| AI Format | DOM/selectors | Accessibility tree with refs |
| CLI | Node.js | Native Rust |
| Selection | CSS/XPath selectors | Deterministic `@e1`, `@e2` refs |

### Architecture

```
┌──────────────┐     ┌─────────────────┐     ┌─────────────┐
│  Blaze/Tess  │────▶│  Rust CLI       │────▶│  Node.js    │
│  Agent       │     │  (fast parsing) │     │  Daemon     │
└──────────────┘     └─────────────────┘     │  (browser)  │
                                             └─────────────┘
```

### Usage Pattern

```bash
# Navigate and get AI-friendly snapshot
agent-browser open https://app.example.com
agent-browser snapshot -i

# Output:
# - heading "Dashboard" [ref=e1]
# - button "Create New" [ref=e2]
# - link "Settings" [ref=e3]

# Interact using refs (no selector needed)
agent-browser click @e2
agent-browser screenshot dashboard-create-modal.png

# Close when done
agent-browser close
```

### Integration Options

**Option A: Replace Playwright MCP**
```yaml
# values.yaml
mcpServers:
  agent-browser:
    name: "Agent Browser"
    description: "AI-optimized browser automation"
    transport: "stdio"
    command: "agent-browser"
    args: ["daemon"]  # Or wrap in MCP server
```

**Option B: Supplement (Use Both)**
- agent-browser for Blaze visual verification
- Playwright for Tess E2E testing (existing test suites)

### Blaze Template Updates

```markdown
## Visual Verification (MANDATORY)

Use agent-browser for screenshot capture:

\```bash
# 1. Open the deployed preview
agent-browser open $PREVIEW_URL

# 2. Capture required viewports
agent-browser screenshot --viewport 375x812 mobile.png
agent-browser screenshot --viewport 1920x1080 desktop.png

# 3. Capture dark mode
agent-browser execute "document.documentElement.classList.add('dark')"
agent-browser screenshot dark-mode.png

# 4. Close browser
agent-browser close
\```
```

## Success Criteria

- [ ] agent-browser installed in agent runtime image
- [ ] MCP server wrapper or direct CLI integration
- [ ] Blaze template updated with agent-browser instructions
- [ ] Visual verification workflow 50%+ faster than Playwright
- [ ] Reduced token usage from accessibility tree vs DOM

## Effort Estimate

**Medium (2 weeks)**
- Week 1: Build MCP wrapper, add to runtime image
- Week 2: Update Blaze/Tess templates, validate workflows

## Technical Notes

### Installation

```bash
npm install -g agent-browser
```

### Auth State Persistence

agent-browser supports session persistence for authenticated testing:
```bash
agent-browser --state ~/.agent-browser/app-auth.json open https://app.example.com
```

### Comparison Benchmark (Recommended)

Before full migration, benchmark:
```bash
# Playwright MCP
time playwright-mcp screenshot test.png

# agent-browser  
time agent-browser open example.com && agent-browser screenshot test.png && agent-browser close
```

## References

- Research entries: `2013553716549558451`, `2011522613991129265`, `2012952318103187967`
- agent-browser docs: https://agent-browser.dev
- GitHub: https://github.com/nicholasoxford/agent-browser

## Approval

- [ ] Approved for implementation
