# Cost Reporting System

## Overview

The Cost Reporting System provides comprehensive tracking and analysis of AI model token usage and associated costs across the CTO platform. This enables per-PR cost tracking, agent-specific cost analysis, and iteration-based cost visibility.

## Key Features

- **Token Usage Tracking**: Monitor token consumption per agent and model
- **Cost Calculation**: Dynamic pricing with support for multiple AI providers
- **PR-Level Analysis**: Calculate total cost per pull request with iteration breakdown
- **Agent Efficiency**: Analyze cost efficiency by agent
- **Reporting**: CLI commands and potential web dashboard for cost visibility
- **Threshold Alerts**: Configure warnings and errors for cost overruns

## Documentation

- **[prd.txt](prd.txt)**: Complete Product Requirements Document with technical specifications
- **[architecture.md](architecture.md)**: Detailed system architecture and implementation design

## Use Cases

### For Developers
- Understand the cost impact of their PRs
- See which agents and iterations consume the most tokens
- Optimize workflow choices based on cost data

### For Platform Engineers
- Monitor overall platform costs
- Identify optimization opportunities
- Configure model defaults for cost efficiency

### For Finance Teams
- Track AI spending for budgeting
- Export cost data for accounting
- Allocate costs across teams or projects

## CLI Commands (Planned)

```bash
# Show costs for a specific PR
cto-cli costs show --pr 123

# Show detailed breakdown with iterations
cto-cli costs show --pr 123 --detailed

# Show costs for a specific agent
cto-cli costs agent --name rex --since 2025-01-01

# Compare costs across multiple PRs
cto-cli costs compare --prs 120,121,122

# Export cost data
cto-cli costs export --output costs.json --since 2025-01-01

# Show current month summary
cto-cli costs summary --month current
```

## Data Model

### Token Usage Record
Each API call is tracked with:
- PR number
- Agent name
- Model used (e.g., claude-sonnet, gpt-4o)
- Input/output tokens
- Workflow stage (intake, code, docs, qa, security)
- Iteration number
- Calculated cost in USD

### PR Cost Summary
Aggregated view per PR including:
- Total cost
- Cost by agent
- Cost by stage
- Number of iterations per agent
- Detailed breakdown

## Pricing

The system maintains pricing information for all supported models:

| Model | Provider | Input (per 1M) | Output (per 1M) |
|-------|----------|---------------|-----------------|
| Claude Sonnet | Anthropic | $3.00 | $15.00 |
| Claude Opus | Anthropic | $15.00 | $75.00 |
| GPT-4o | OpenAI | $2.50 | $10.00 |
| Sonar Pro | Perplexity | $3.00 | $15.00 |

Pricing is configurable via ConfigMap and can be updated without code changes.

## Implementation Status

This feature is currently in the design phase. The PRD and architecture documents outline the complete implementation plan.

## Development Roadmap

1. **Phase 1 (Days 1-3)**: Core infrastructure - data models, storage, cost collector
2. **Phase 2 (Days 4-5)**: CLI integration - commands, query engine
3. **Phase 3 (Days 6-8)**: Workflow integration - agent instrumentation
4. **Phase 4 (Days 9-10)**: Reporting - PR comments, MCP tool
5. **Phase 5 (Days 11-12)**: Advanced features - analytics, trends, export

## Integration Points

### Agent Workflows
Each agent workflow will report token usage to the cost collector after API calls.

### GitHub PR Comments
Automated cost summaries will be posted to PR comments when enabled.

### CLI
New `costs` subcommand provides querying and reporting capabilities.

### MCP Server
New `mcp_cto_costs` tool enables cost queries from AI agents.

## Configuration

Cost tracking is configured via:
- ConfigMap for model pricing
- Cost config for thresholds and settings
- Storage backend configuration (SQLite, PostgreSQL)

## Storage

Cost data is persisted in a SQLite database (upgradable to PostgreSQL) with:
- Indexed queries for fast retrieval
- Configurable data retention
- Cleanup jobs for old records

## Monitoring

Prometheus metrics track:
- Cost recording duration
- Total costs by PR/agent/stage
- Token usage totals
- Threshold violations

## Related Documentation

- [Main README](../../README.md)
- [Agent Documentation](../../AGENTS.md)
- [CLI Documentation](../cto-cli-config-reference.md)

