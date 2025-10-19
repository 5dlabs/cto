# Using Blaze Frontend Agent

This guide explains how to use Blaze for frontend implementation tasks in the CTO platform.

## Quick Start

### 1. Configure Your Project for Blaze

In your `cto-config.json`, set Blaze as the default implementation agent for frontend projects:

```json
{
  "defaults": {
    "play": {
      "implementationAgent": "5DLabs-Blaze",
      "qualityAgent": "5DLabs-Cleo",
      "testingAgent": "5DLabs-Tess",
      "cli": "codex",
      "model": "gpt-5-codex"
    }
  },
  "agents": {
    "blaze": {
      "githubApp": "5DLabs-Blaze",
      "cli": "codex",
      "model": "gpt-5-codex",
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "context7_get_library_docs"
        ]
      }
    }
  }
}
```

### 2. Run a Frontend Task

```bash
# Using MCP tool
cto play --task_id=5

# Or with explicit agent specification
cto play --task_id=5 --implementation_agent=5DLabs-Blaze
```

## Configuration Options

### Per-Project Configuration

For frontend-heavy projects, set Blaze as default in `cto-config.json`:

```json
{
  "defaults": {
    "play": {
      "implementationAgent": "5DLabs-Blaze"
    }
  }
}
```

### Mixed Projects (Frontend + Backend)

For projects with both frontend and backend tasks:

**Option 1: Manual Override Per Task**
```bash
# Backend task - use Rex
cto play --task_id=3 --implementation_agent=5DLabs-Rex

# Frontend task - use Blaze  
cto play --task_id=4 --implementation_agent=5DLabs-Blaze
```

**Option 2: Multiple Config Files**
Create separate config files for different workflows:

```bash
# cto-config-backend.json - Rex default
# cto-config-frontend.json - Blaze default

# Use specific config
CTO_CONFIG=cto-config-frontend.json cto play --task_id=4
```

### CLI-Specific Configuration

Blaze works with all supported CLIs. Configure per your preference:

```json
{
  "agents": {
    "blaze": {
      "githubApp": "5DLabs-Blaze",
      "cli": "codex",           // Recommended for Blaze
      "model": "gpt-5-codex",
      // Or use Claude Code:
      // "cli": "claude",
      // "model": "claude-sonnet-4-20250514"
    }
  }
}
```

## Task Requirements for Blaze

For optimal results with Blaze, structure your tasks to include:

### 1. Task Description Format

```markdown
# Task 5: Implement User Dashboard

## Description
Create a responsive user dashboard with analytics widgets.

## Technology Stack
- React 19 + Next.js 15
- TypeScript 5 (strict mode)
- Tailwind CSS + shadcn/ui
- Responsive design (mobile/tablet/desktop)

## Design Requirements
- **Colors**: Blue (#3B82F6) primary, purple (#8B5CF6) accent
- **Typography**: Inter font family
- **Style**: Modern, minimal, generous spacing
- **Accessibility**: WCAG AA compliant

## Acceptance Criteria
- [ ] Mobile-responsive (375px, 768px, 1920px)
- [ ] Keyboard navigation works
- [ ] All components properly typed (TypeScript strict)
- [ ] Production build succeeds
- [ ] Live preview URL provided in PR
```

### 2. Optional: Design System

Create `.blaze/design-system.json` in your repository for consistent theming:

```json
{
  "name": "My Project Design System",
  "colors": {
    "primary": "#3B82F6",
    "secondary": "#8B5CF6",
    "accent": "#10B981",
    "neutral": "#64748B",
    "background": "#FFFFFF",
    "foreground": "#0F172A"
  },
  "typography": {
    "fontFamily": "Inter, system-ui, sans-serif",
    "headingFamily": "Inter, system-ui, sans-serif"
  },
  "borderRadius": {
    "default": "0.5rem",
    "lg": "0.75rem"
  }
}
```

## Workflow: Blaze → Cleo → Tess

Blaze integrates into the standard play workflow:

```
1. Blaze (Implementation)
   - Reads task requirements
   - Generates Next.js + React components
   - Creates responsive, accessible UI
   - Deploys to K8s staging namespace
   - Sets up Ngrok live preview URL
   - Runs Playwright E2E tests
   - Creates PR with screenshots + live URL

2. Cleo (Quality Review)
   - Reviews TypeScript types and code quality
   - Checks responsive design
   - Validates accessibility (WCAG AA)
   - Reviews component structure
   - Provides feedback via PR comments

3. Tess (Testing & Validation)
   - Runs additional E2E tests
   - Validates live preview
   - Checks performance metrics
   - Approves PR or requests changes
```

## Expected Outputs

When Blaze completes a task, you'll see:

### Pull Request with:
- ✅ Production-ready React/Next.js code
- ✅ TypeScript strict mode (no errors)
- ✅ Responsive design (mobile/tablet/desktop)
- ✅ WCAG AA accessible components
- ✅ Live preview URL (via Ngrok)
- ✅ Screenshots (via Playwright)
- ✅ E2E test results
- ✅ Component documentation

### Example PR Description:
```markdown
## 🎨 Frontend Implementation

### Components Created
- `DashboardLayout`: Main layout with responsive sidebar
- `AnalyticsWidget`: Real-time metrics display
- `UserProfile`: User info card with avatar

### Technology Stack
- React 19 + Next.js 15
- TypeScript 5 (strict)
- Tailwind CSS 4
- shadcn/ui: button, card, dialog, avatar

### Live Preview
🌐 https://abc123.ngrok.io
Available for 24 hours for review

### Screenshots
[Mobile] [Tablet] [Desktop]

### Quality Checks
✅ TypeScript: 0 errors
✅ ESLint: Passed
✅ Build: Success
✅ Responsive: 375px/768px/1920px
✅ Accessibility: WCAG AA
✅ E2E Tests: 15/15 passed
```

## Troubleshooting

### Blaze Not Available

If you get "Agent 5DLabs-Blaze not found":
1. Check `cto-config.json` has `blaze` in `agents` section
2. Verify GitHub App is installed in your organization
3. Check Helm values include Blaze configuration

### Wrong Agent Used

If Rex runs instead of Blaze:
1. Check `defaults.play.implementationAgent` in config
2. Use explicit override: `--implementation_agent=5DLabs-Blaze`
3. Verify you're using the correct config file

### Build Failures

Blaze expects Node.js 20+ and pnpm in the runtime image:
1. Check `infra/images/codex/Dockerfile` has Node.js 20
2. Verify pnpm is installed globally
3. Runtime image should include: node, pnpm, playwright

## Advanced Usage

### Custom Tooling

Add frontend-specific tools to Blaze's configuration:

```json
{
  "agents": {
    "blaze": {
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "context7_get_library_docs",
          "figma_api",              // Design imports
          "lighthouse_audit"         // Performance testing
        ]
      }
    }
  }
}
```

### Model Rotation

Enable model rotation for better reliability:

```json
{
  "agents": {
    "blaze": {
      "modelRotation": {
        "enabled": true,
        "models": [
          "gpt-5-codex",
          "claude-sonnet-4-20250514",
          "claude-opus-4-1-20250805"
        ]
      }
    }
  }
}
```

### Multiple Frontend Agents

For large teams, create specialized frontend agents:

```json
{
  "agents": {
    "blaze-react": {
      "githubApp": "5DLabs-Blaze",
      "cli": "codex",
      "model": "gpt-5-codex"
    },
    "blaze-vue": {
      "githubApp": "5DLabs-Blaze-Vue",
      "cli": "codex", 
      "model": "gpt-5-codex"
    }
  }
}
```

## Future Enhancements

Planned improvements for automatic agent selection:

### Label-Based Auto-Selection (Coming Soon)
```bash
# GitHub issue has label "frontend" → auto-selects Blaze
# GitHub issue has label "backend" → auto-selects Rex
# GitHub issue has label "agent-blaze" → explicit Blaze selection

cto play --task_id=5  # Auto-detects from labels
```

### Task Type Detection (Coming Soon)
```bash
# Analyzes task description for keywords:
# - "React", "Next.js", "frontend", "UI" → Blaze
# - "Rust", "backend", "API", "database" → Rex

cto play --task_id=5  # Auto-detects from description
```

## See Also

- [Architecture Documentation](./architecture.md) - Technical details
- [Theming Strategy](./THEMING_STRATEGY.md) - Design system configuration
- [README](./README.md) - General overview
- [cto-config.json](../../cto-config.json) - Configuration reference

