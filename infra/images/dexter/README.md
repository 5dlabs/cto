# Dexter Agent Image

Dexter is an autonomous financial research agent that performs analysis using task planning, self-reflection, and real-time market data.

**Repository:** https://github.com/virattt/dexter

> **Note:** Dexter was rewritten from Python to TypeScript/Bun in December 2025.
> This image uses the Bun runtime to execute the TypeScript implementation.

## Features

- **Intelligent Task Planning:** Automatically decomposes complex queries into structured research steps
- **Autonomous Execution:** Selects and executes the right tools to gather financial data
- **Self-Validation:** Checks its own work and iterates until tasks are complete
- **Real-Time Financial Data:** Access to income statements, balance sheets, and cash flow statements
- **Web Search:** Optional Tavily integration for web search capabilities
- **Multi-Model Support:** Works with OpenAI (gpt-4.1), Anthropic (claude-*), and Google (gemini-*)

## Build

```bash
# Build with latest Dexter version
docker build -t ghcr.io/5dlabs/dexter:latest .

# Build with specific version (if tags exist)
docker build --build-arg DEXTER_VERSION=1.0.1 -t ghcr.io/5dlabs/dexter:1.0.1 .
```

## Run Locally

### Interactive Mode

```bash
docker run -it --rm \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  -e FINANCIAL_DATASETS_API_KEY=$FINANCIAL_DATASETS_API_KEY \
  ghcr.io/5dlabs/dexter:latest \
  bash -c "cd /opt/dexter && bun start"
```

### Development Mode

```bash
docker run -it --rm \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  -e FINANCIAL_DATASETS_API_KEY=$FINANCIAL_DATASETS_API_KEY \
  ghcr.io/5dlabs/dexter:latest \
  bash -c "cd /opt/dexter && bun dev"
```

## Environment Variables

### Required (at least one LLM provider)

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key for gpt-* models |
| `ANTHROPIC_API_KEY` | Anthropic API key for claude-* models |
| `GOOGLE_API_KEY` | Google API key for gemini-* models |

### Financial Data

| Variable | Description |
|----------|-------------|
| `FINANCIAL_DATASETS_API_KEY` | Financial Datasets API key ([Get one here](https://financialdatasets.ai/)) |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `DEXTER_MODEL` | `claude-sonnet-4-20250514` | Default model to use |
| `DEXTER_MAX_STEPS` | `20` | Maximum global execution steps |
| `DEXTER_MAX_STEPS_PER_TASK` | `5` | Maximum steps per individual task |
| `TAVILY_API_KEY` | - | Tavily API key for web search ([Get one here](https://tavily.com/)) |

## Supported Models

Dexter supports multiple LLM providers via LangChain.js:

| Provider | Model Examples | Prefix |
|----------|----------------|--------|
| OpenAI | gpt-4.1, gpt-4, gpt-4-turbo | `gpt-*` |
| Anthropic | claude-sonnet-4-20250514, claude-3-opus | `claude-*` |
| Google | gemini-pro, gemini-1.5-pro | `gemini-*` |

Use `/model` in the interactive CLI to switch between models.

## Example Queries

```
"What was Apple's revenue growth over the last 4 quarters?"
"Compare Microsoft and Google's operating margins for 2023"
"Analyze Tesla's cash flow trends over the past year"
"What is Amazon's debt-to-equity ratio based on recent financials?"
```

## Architecture

Dexter uses a multi-agent architecture with specialized components:

1. **Planning Agent:** Analyzes queries and creates structured task lists
2. **Action Agent:** Selects appropriate tools and executes research steps
3. **Validation Agent:** Verifies task completion and data sufficiency
4. **Answer Agent:** Synthesizes findings into comprehensive responses

### Tech Stack

- **Runtime:** [Bun](https://bun.sh/)
- **UI Framework:** [React](https://react.dev/) + [Ink](https://github.com/vadimdemedes/ink) (terminal UI)
- **LLM Integration:** [LangChain.js](https://js.langchain.com/) with multi-provider support
- **Schema Validation:** [Zod](https://zod.dev/)
- **Language:** TypeScript

## Integration with CTO Platform

This image is designed to work with the CTO platform's multi-CLI architecture. Configure via:

```yaml
# In cto-config.json
{
  "agents": {
    "dexter": {
      "cli": "dexter",
      "cliConfig": {
        "model": "claude-sonnet-4-20250514",
        "maxSteps": 20,
        "maxStepsPerTask": 5
      }
    }
  }
}
```

## Troubleshooting

### "No LLM API key found"

Ensure at least one of `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `GOOGLE_API_KEY` is set.

### "Financial tools not working"

The `FINANCIAL_DATASETS_API_KEY` is required for accessing SEC filings, financial statements, and market data. Get a key at https://financialdatasets.ai/

### "Rate limit exceeded"

Dexter makes multiple API calls during research. Consider:
- Using a model with higher rate limits
- Reducing `DEXTER_MAX_STEPS`
- Adding delays between queries

### Build fails with "No pyproject.toml found"

This error occurs if you're using an outdated Dockerfile that tries to use Python/uv. Dexter was rewritten to TypeScript/Bun in December 2025. Ensure you're using the updated Dockerfile that installs Bun instead of Python.

