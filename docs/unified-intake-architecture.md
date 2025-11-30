# Unified Intake Architecture

## Overview

The unified intake system combines PRD (Product Requirements Document) parsing,
task generation, context enrichment, and documentation generation into a single
operation. This replaces the previous two-step process (`intake_prd` + `docs`).

## Key Benefits

1. **Single Operation**: One MCP tool call (`intake`) handles everything
2. **Reduced API Calls**: Single GitHub clone, single PR creation
3. **Consistent Context**: Tasks and documentation share the same context window
4. **Firecrawl Integration**: Automatic URL extraction and context enrichment
5. **Opus 4.5 Support**: Leverages Claude Opus 4.5 for superior task analysis

## Architecture

```text
┌─────────────────────────────────────────────────────────────────────┐
│                      UNIFIED INTAKE FLOW                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐     ┌──────────────┐     ┌────────────────┐       │
│  │   MCP       │     │    Argo      │     │   Agent        │       │
│  │   Server    │────▶│   Workflow   │────▶│   Container    │       │
│  │ (cto-mcp)   │     │   Template   │     │ (unified.sh)   │       │
│  └─────────────┘     └──────────────┘     └────────────────┘       │
│        │                                          │                 │
│        ▼                                          ▼                 │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    UNIFIED INTAKE SCRIPT                     │   │
│  ├─────────────────────────────────────────────────────────────┤   │
│  │ Phase 1: Configuration & Auth                               │   │
│  │   - Load ConfigMap (PRD, architecture, config)              │   │
│  │   - Generate GitHub App token                               │   │
│  │   - Clone repository                                        │   │
│  │                                                             │   │
│  │ Phase 2: TaskMaster Setup                                   │   │
│  │   - Initialize .taskmaster structure                        │   │
│  │   - Configure models (Opus 4.5)                             │   │
│  │   - Parse PRD → tasks.json                                  │   │
│  │   - Analyze complexity                                       │   │
│  │   - Expand tasks with subtasks                              │   │
│  │   - Add agent routing hints                                 │   │
│  │                                                             │   │
│  │ Phase 3: Context Enrichment (Firecrawl)                     │   │
│  │   - Extract URLs from PRD                                   │   │
│  │   - Scrape referenced documentation                         │   │
│  │   - Create enriched-context.md                              │   │
│  │                                                             │   │
│  │ Phase 4: Documentation Generation                           │   │
│  │   - Generate task.md per task                               │   │
│  │   - Generate prompt.md per task                             │   │
│  │   - Generate acceptance-criteria.md per task                │   │
│  │   - Generate task.xml per task                              │   │
│  │                                                             │   │
│  │ Phase 5: PR Creation                                        │   │
│  │   - Single commit with all changes                          │   │
│  │   - Create PR with comprehensive body                       │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## MCP Tool Schema

```json
{
  "name": "intake",
  "description": "Process a PRD to generate TaskMaster tasks and documentation",
  "inputSchema": {
    "type": "object",
    "properties": {
      "project_name": {
        "type": "string",
        "description": "Name of the project subdirectory"
      },
      "prd_content": {
        "type": "string",
        "description": "PRD content (optional, reads from file if not provided)"
      },
      "architecture_content": {
        "type": "string",
        "description": "Architecture document content (optional)"
      },
      "enrich_context": {
        "type": "boolean",
        "description": "Auto-scrape URLs found in PRD via Firecrawl",
        "default": true
      },
      "model": {
        "type": "string",
        "description": "Model override (defaults to claude-opus-4-5-20250929)"
      },
      "include_codebase": {
        "type": "boolean",
        "description": "Include existing codebase as markdown context",
        "default": false
      },
      "cli": {
        "type": "string",
        "enum": ["claude", "cursor", "codex"],
        "description": "CLI for documentation generation (defaults to claude)"
      }
    },
    "required": ["project_name"]
  }
}
```

## Multi-CLI Support

The unified intake supports multiple AI CLIs for documentation generation:

| CLI | Description |
|-----|-------------|
| `claude` | Claude Code CLI (default) |
| `cursor` | Cursor CLI |
| `codex` | Codex CLI |

Template files for each CLI are located at:

- `intake/claude/container.sh.hbs`
- `intake/cursor/container.sh.hbs`
- `intake/codex/container.sh.hbs`

## CodeRun Integration

Documentation runs use the CodeRun CRD with `runType: "documentation"`:

```yaml
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: docs-run-example
spec:
  runType: documentation  # Key field for docs runs
  service: my-project
  repositoryUrl: https://github.com/org/repo
  docsRepositoryUrl: https://github.com/org/repo
  model: claude-opus-4-5-20250929
  githubApp: 5DLabs-Morgan
```

This replaces the deprecated DocsRun CRD.

## Configuration

### Default Model Configuration (cto-config.json)

```json
{
  "defaults": {
    "intake": {
      "primary": {
        "model": "claude-opus-4-5-20250929",
        "provider": "anthropic"
      },
      "research": {
        "model": "claude-opus-4-5-20250929",
        "provider": "anthropic"
      },
      "fallback": {
        "model": "claude-sonnet-4-5-20250929",
        "provider": "anthropic"
      }
    }
  }
}
```

## Generated Directory Structure

```text
project-name/
├── .taskmaster/
│   ├── config.json              # TaskMaster configuration
│   ├── docs/
│   │   ├── prd.txt              # Original PRD
│   │   ├── architecture.md      # Original architecture (if provided)
│   │   ├── enriched-context.md  # Firecrawl-scraped context (if enabled)
│   │   └── task-{id}/           # Per-task documentation
│   │       ├── task.md          # Task overview and implementation guide
│   │       ├── prompt.md        # Agent execution prompt
│   │       ├── acceptance-criteria.md
│   │       └── task.xml         # XML-structured prompt for LLMs
│   ├── tasks/
│   │   └── tasks.json           # TaskMaster task definitions
│   └── reports/
│       └── complexity-report.json
└── README.md
```

## Migration from Separate Workflows

### Before (Deprecated)

```bash
# Step 1: Parse PRD and generate tasks
cto intake_prd --project_name my-project

# Step 2: Generate documentation (separate operation)
cto docs --working_directory my-project
```

### After (Unified)

```bash
# Single operation handles everything
cto intake --project_name my-project
```

## Deprecation Notice

The following are deprecated and will be removed in a future release:

- `intake_prd` MCP tool → Use `intake` instead
- `docs` MCP tool → Use `intake` instead (docs are generated automatically)
- `DocsRun` CRD → Use `CodeRun` with `runType: "documentation"` instead
- `intake/intake.sh` script → Use `intake/unified-intake.sh.hbs`
- `docs/claude/container.sh.hbs` → Functionality merged into unified intake
- `docsrun-template.yaml` → Use project-intake or CodeRun directly

## Context Enrichment (Firecrawl)

When `enrich_context` is enabled (default: true), the unified intake:

1. Extracts URLs from the PRD using regex
2. Filters to documentation-relevant URLs
3. Uses Firecrawl to scrape content
4. Creates `enriched-context.md` with scraped content
5. Makes this context available during task generation

This enables agents to generate more accurate tasks by understanding:

- Referenced APIs and libraries
- External documentation
- Best practices from linked resources

## Future Enhancements (Phase 3+)

### Living PRD Context

The unified intake lays groundwork for the "Living PRD" concept where:

1. `prd-context.json` stores execution state and learnings
2. Agents reference and update context during execution
3. Drift detection prevents tasks from diverging from PRD intent
4. Continuous alignment replaces one-time generation

See the Phase 3-4 plan for implementation details.
