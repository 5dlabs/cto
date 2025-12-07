# Linear CTO Configuration

This document explains how to configure CTO platform settings (CLI, model) directly from Linear issues using labels and/or description frontmatter.

## Overview

The CTO platform supports per-issue configuration that allows you to override default settings when triggering intake or play workflows from Linear. This is useful when:

- Different projects require different AI models (e.g., Opus for complex architectural work)
- You want to use a specific CLI for certain tasks (e.g., Cursor for frontend-heavy work)
- You need to experiment with different configurations without changing environment defaults

## Configuration Methods

There are two methods to configure CTO settings in Linear issues:

### 1. Label Groups (Quick Selection)

Use labels in the Linear UI for quick configuration. The platform supports two label group formats:

#### Grouped Labels (Recommended)

Create label groups in Linear for organized configuration:

| Label Group | Labels | Description |
|-------------|--------|-------------|
| `CTO CLI/` | `claude`, `cursor`, `codex`, `dexter`, `opencode` | CLI to use for code generation |
| `CTO Model/` | `sonnet`, `opus`, `gpt-4.1`, `o3` | Model shortcut or full name |

**Example labels:**
- `CTO CLI/cursor` - Use Cursor CLI
- `CTO Model/opus` - Use Claude Opus model

#### Flat Labels

You can also use simple labels without the group prefix:

- `claude`, `cursor`, `codex`, `dexter`, `opencode` → Sets CLI
- `sonnet`, `opus`, `gpt-4.1`, `o3` → Sets model (uses shortcut mapping)

### 2. Description Frontmatter (Advanced)

For more control, add YAML frontmatter to the issue description:

```yaml
---
cto:
  cli: cursor
  model: claude-opus-4-20250514
---
## PRD: My Feature

This is the actual PRD content...
```

#### Supported Fields

| Field | Description | Example Values |
|-------|-------------|----------------|
| `cli` | CLI to use | `claude`, `cursor`, `codex`, `dexter`, `opencode` |
| `model` | Full model name | `claude-sonnet-4-20250514`, `claude-opus-4-20250514`, `gpt-4.1`, `o3` |

**Note:** When using frontmatter, specify the full model name (not the shortcut).

## Resolution Order

Configuration is resolved in the following order (later sources override earlier ones):

1. **Environment Defaults** - Server configuration (`PRIMARY_MODEL`, `INTAKE_CLI`, etc.)
2. **Labels** - Labels on the Linear issue
3. **Frontmatter** - YAML frontmatter in the issue description

This means frontmatter values will always override labels, which override environment defaults.

## Model Shortcuts

The following model shortcuts are supported in labels:

| Shortcut | Full Model Name |
|----------|-----------------|
| `sonnet` | `claude-sonnet-4-20250514` |
| `opus` | `claude-opus-4-20250514` |
| `gpt-4.1` | `gpt-4.1` |
| `o3` | `o3` |

## Examples

### Example 1: Simple Label Configuration

Add these labels to your PRD issue:
- `CTO CLI/cursor`
- `CTO Model/opus`

### Example 2: Flat Labels

Add these labels to your issue:
- `cursor`
- `opus`

### Example 3: Frontmatter Override

```yaml
---
cto:
  cli: codex
  model: gpt-4.1
---
# PRD: API Refactoring

## Objective
Refactor the REST API to use OpenAPI spec...
```

### Example 4: Mixed Configuration

You can combine labels and frontmatter. Labels provide defaults, frontmatter overrides specific values:

**Labels:**
- `CTO CLI/claude`

**Description:**
```yaml
---
cto:
  model: claude-opus-4-20250514
---
# PRD: Complex Feature

This PRD requires Opus for better reasoning...
```

Result: CLI = `claude` (from label), Model = `claude-opus-4-20250514` (from frontmatter)

## Setting Up Linear Label Groups

To create the label groups in Linear:

1. Go to **Settings** → **Labels** in your Linear workspace
2. Create a label group called `CTO CLI` with these labels:
   - `claude` (suggested color: Blue #0066FF)
   - `cursor`
   - `codex`
   - `dexter`
   - `opencode`

3. Create a label group called `CTO Model` with these labels:
   - `sonnet` (suggested color: Purple #8B5CF6)
   - `opus`
   - `gpt-4.1`
   - `o3`

## Troubleshooting

### Configuration Not Being Applied

1. **Check label spelling** - Labels are case-insensitive, but must match exactly
2. **Verify frontmatter format** - Ensure proper YAML syntax with `---` delimiters
3. **Check the cto: section** - The frontmatter must have a `cto:` key

### Frontmatter Not Parsed

Ensure:
- The frontmatter starts at the very beginning of the description
- It uses `---` as both opening and closing delimiters
- The YAML is valid (proper indentation, no syntax errors)

### Unknown CLI or Model

Only supported CLIs and models are recognized:
- **CLIs:** `claude`, `cursor`, `codex`, `dexter`, `opencode`
- **Model shortcuts:** `sonnet`, `opus`, `gpt-4.1`, `o3`
- **Full model names:** Any valid model identifier (e.g., `claude-sonnet-4-20250514`)

## API Reference

No new Linear API calls are required - labels are already included in webhook payloads and `get_issue()` responses. The configuration extraction happens automatically when processing Linear events.


