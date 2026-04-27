# Agent Browser Skill

Use `agent-browser` for headless web automation via CLI. It provides snapshot-based
element refs and structured JSON output, purpose-built for AI agent workflows.

## When to Use

- You need deterministic web automation (not just fetching static HTML).
- You want compact accessibility snapshots with refs and JSON output.
- You need to interact with JS-heavy pages, SPAs, or login flows.
- `web_fetch` or Firecrawl are insufficient for the task.

## When NOT to Use

- Static page content → use `web_fetch` or Firecrawl instead.
- Just finding URLs → use `web_search`.
- You need a full SDK or custom JS integration.

## Core Workflow

Always follow the snapshot → act → re-snapshot loop:

```
# 1. Navigate
agent-browser open <url>

# 2. Get interactive elements with refs
agent-browser snapshot -i --json

# 3. Act using refs from the snapshot
agent-browser click @e2
agent-browser fill @e3 "text"

# 4. Re-snapshot after DOM changes
agent-browser snapshot -i --json
```

## Essential Commands

### Navigation & Lifecycle
```
agent-browser open <url>              # Navigate (aliases: goto, navigate)
agent-browser back / forward / reload # History navigation
agent-browser close                   # Close browser (aliases: quit, exit)
```

### Snapshot (primary inspection tool)
```
agent-browser snapshot                # Full accessibility tree
agent-browser snapshot -i             # Interactive elements only (recommended)
agent-browser snapshot -i -c          # Interactive + compact
agent-browser snapshot -i -c -d 5    # With depth limit
agent-browser snapshot -s "#main"    # Scope to selector
agent-browser snapshot --json        # Machine-readable output
```

### Interaction (use @refs from snapshot)
```
agent-browser click @e2              # Click by ref
agent-browser fill @e3 "value"       # Clear and fill input
agent-browser type @e3 "value"       # Type without clearing
agent-browser press Enter            # Press key
agent-browser select @e4 "option"    # Select dropdown
agent-browser check @e5              # Check checkbox
agent-browser hover @e6              # Hover element
agent-browser scroll down 500        # Scroll page
```

### Get Info
```
agent-browser get text @e1           # Text content
agent-browser get value @e3          # Input value
agent-browser get title              # Page title
agent-browser get url                # Current URL
agent-browser is visible @e2         # Visibility check
```

### Screenshots & Debug
```
agent-browser screenshot             # Save to temp dir
agent-browser screenshot page.png    # Save to path
agent-browser screenshot --full      # Full page
agent-browser console                # View console messages
agent-browser errors                 # View JS errors
```

### Wait
```
agent-browser wait @e1               # Wait for element visible
agent-browser wait 2000              # Wait milliseconds
agent-browser wait --text "Welcome"  # Wait for text
agent-browser wait --load networkidle  # Wait for network idle
```

### Tabs
```
agent-browser tab                    # List tabs
agent-browser tab new [url]          # New tab
agent-browser tab 2                  # Switch to tab
agent-browser tab close              # Close current tab
```

### Find Elements (semantic locators)
```
agent-browser find role button click --name "Submit"
agent-browser find text "Sign In" click
agent-browser find label "Email" fill "user@example.com"
```

## Sessions & Profiles

```
# Isolated sessions (separate browser instances)
agent-browser --session agent1 open site-a.com
agent-browser --session agent2 open site-b.com

# Persistent profiles (cookies/auth survive restarts)
agent-browser --profile /workspace/.browser-profiles/myapp open myapp.com
```

## Safety Rules

- Do NOT use `eval`, `--allow-file-access`, or custom `--executable-path` without explicit user approval.
- Do NOT use `network route`, `set credentials`, or cookie/storage mutations unless the task requires it.
- Treat tokens and credentials as secrets — never log them.
- Always `close` sessions when done to release resources.
- Snapshot early, act via refs, then re-snapshot after DOM changes.

## Troubleshooting

| Problem | Fix |
|---------|-----|
| Snapshot empty/missing elements | Try without `-i` flag; some elements are non-interactive |
| Click does nothing | Re-snapshot — refs are stale after DOM changes |
| Element not found | Use `wait @ref` before acting; page may still be loading |
| Timeout errors | Add `agent-browser wait --load networkidle` before snapshot |
| Login state lost | Use `--profile <path>` for persistent auth |

## Environment

In this Kubernetes pod, `agent-browser` is pre-installed globally with Playwright's
Chromium. The executable path is set via `AGENT_BROWSER_EXECUTABLE_PATH` in
`/etc/profile.d/agent-browser.sh`. Always use `--json` flag when parsing output
programmatically.
