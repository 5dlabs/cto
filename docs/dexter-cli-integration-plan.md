# Dexter CLI Integration Plan

**Status:** Draft
**Date:** December 7, 2025
**Repository:** https://github.com/virattt/dexter

## Executive Summary

Dexter is an autonomous financial research agent built by [@virattt](https://twitter.com/virattt) that uses a multi-agent architecture with task planning, self-reflection, and real-time market data. The goal is to integrate Dexter as an additional CLI option in the CTO platform, following the existing multi-CLI integration patterns.

## Dexter Overview

### What Is Dexter?

- **Type:** Python-based autonomous agent for financial research
- **Architecture:** Multi-agent system with Planning, Action, Validation, and Answer agents
- **Primary Use Case:** Deep financial research (SEC filings, financial statements, stock analysis)
- **Unique Features:**
  - Intelligent task planning with automatic query decomposition
  - Self-validation and iteration until tasks are complete
  - Loop detection and step limits for safety
  - Multi-model support (OpenAI, Anthropic, Google Gemini)

### Key Dependencies

```toml
# From pyproject.toml
dependencies = [
    "googlenewsdecoder>=0.1.7",
    "langchain>=0.3.27",
    "langchain-openai>=0.3.35",
    "langchain-anthropic>=0.3.0",
    "langchain-google-genai>=2.0.0",
    "langsmith>=0.3.37",
    "openai>=2.2.0",
    "prompt-toolkit>=3.0.0",
    "pydantic>=2.11.10",
    "python-dotenv>=1.1.1",
    "requests>=2.32.5",
]
```

### Required Environment Variables

```bash
# LLM API Keys (at least one required)
OPENAI_API_KEY=<key>
ANTHROPIC_API_KEY=<key>
GOOGLE_API_KEY=<key>

# Financial Data API (required for financial research)
FINANCIAL_DATASETS_API_KEY=<key>

# Optional: LangSmith for tracing
LANGSMITH_API_KEY=<key>
LANGSMITH_ENDPOINT=https://api.smith.langchain.com
LANGSMITH_PROJECT=dexter
LANGSMITH_TRACING=true
```

### Default Model

```python
# From model.py
DEFAULT_MODEL = "gpt-4.1"  # OpenAI GPT-4.1 (configurable)
```

## Architecture Analysis

### Dexter's Multi-Agent Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│                     User Query                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   PLANNING AGENT                             │
│  - Analyzes query complexity                                 │
│  - Creates structured task list                              │
│  - Maps tasks to available tools                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    ACTION AGENT                              │
│  - Selects appropriate tools                                 │
│  - Optimizes tool arguments                                  │
│  - Executes research steps                                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  VALIDATION AGENT                            │
│  - Verifies task completion                                  │
│  - Checks data sufficiency                                   │
│  - Meta-validates overall goal achievement                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    ANSWER AGENT                              │
│  - Synthesizes collected data                                │
│  - Generates comprehensive response                          │
│  - Streams output to user                                    │
└─────────────────────────────────────────────────────────────┘
```

### Model Support

Dexter supports multiple LLM providers via LangChain:

| Provider  | Model Prefix | Models                          |
|-----------|--------------|----------------------------------|
| OpenAI    | `gpt-*`      | gpt-4.1, gpt-4, gpt-4-turbo, etc.|
| Anthropic | `claude-*`   | claude-3-opus, claude-3-sonnet   |
| Google    | `gemini-*`   | gemini-pro, gemini-1.5-pro       |

### Key Differences from Other CLIs

| Aspect | Claude Code | Codex | Dexter |
|--------|-------------|-------|--------|
| **Language** | Node.js | Rust binary | Python |
| **Package Manager** | npm | npm (rust bin) | pip/uv |
| **Config Format** | CLAUDE.md + mcp.json | TOML | .env + Python |
| **Memory Mechanism** | CLAUDE.md | AGENTS.md | Context Manager (file-based) |
| **Tool Integration** | Native MCP | STDIO MCP | LangChain Tools |
| **Streaming** | Native | STDIO | LangChain streaming |

## Implementation Plan

### Phase 1: Docker Image Creation (Week 1)

#### 1.1 Create Dockerfile

Create `infra/images/dexter/Dockerfile`:

```dockerfile
ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

# Setup workspace
RUN mkdir -p /workspace /home/node/.cache/pip
WORKDIR /workspace

# Install uv (fast Python package manager)
USER root
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:$PATH"

# Install Dexter and dependencies
ARG DEXTER_VERSION=latest
RUN if [ "$DEXTER_VERSION" = "latest" ]; then \
      pip install --break-system-packages git+https://github.com/virattt/dexter.git; \
    else \
      pip install --break-system-packages git+https://github.com/virattt/dexter.git@v${DEXTER_VERSION}; \
    fi

# Create dexter config directory
RUN mkdir -p /home/node/.dexter && chown -R 1000:1000 /home/node/.dexter

# Configure git identity
USER node
RUN git config --global user.name "5D Labs Agent" && \
    git config --global user.email "agent@5dlabs.com"

USER node
```

#### 1.2 Create README

Create `infra/images/dexter/README.md`:

```markdown
# Dexter Agent Image

Dexter is an autonomous financial research agent that performs analysis
using task planning, self-reflection, and real-time market data.

## Build

```bash
docker build -t ghcr.io/5dlabs/dexter:latest .
```

## Run Locally

```bash
docker run -it \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  -e FINANCIAL_DATASETS_API_KEY=$FINANCIAL_DATASETS_API_KEY \
  ghcr.io/5dlabs/dexter:latest \
  dexter-agent
```

## Environment Variables

- `OPENAI_API_KEY` - OpenAI API key (or ANTHROPIC_API_KEY, GOOGLE_API_KEY)
- `FINANCIAL_DATASETS_API_KEY` - Financial Datasets API key for market data
- `LANGSMITH_API_KEY` (optional) - LangSmith for tracing
```

### Phase 2: CLI Invoke Template (Week 1)

#### 2.1 Create Invoke Script Template

Create `templates/clis/dexter/invoke.sh.hbs`:

```bash
# =========================================================================
# Dexter Agent CLI Invocation Partial
#
# This partial handles ONLY the Dexter CLI command building and execution.
# Dexter is a Python-based autonomous financial research agent.
#
# Required context variables:
#   - DEXTER_WORK_DIR: Working directory for Dexter
#   - PROMPT_CONTENT: The prompt/query to send to Dexter
#
# Optional context variables:
#   - model: Model to use (defaults to gpt-4.1)
#   - max_steps: Maximum global steps (default: 20)
#   - max_steps_per_task: Maximum steps per task (default: 5)
# =========================================================================

echo "═══════════════════════════════════════════════════════════════"
echo "║               DEXTER AGENT CLI INVOCATION                    ║"
echo "═══════════════════════════════════════════════════════════════"

# Set working directory
DEXTER_WORK_DIR="${DEXTER_WORK_DIR:-/workspace}"
cd "$DEXTER_WORK_DIR"

# Configure model (defaults to gpt-4.1)
{{#if model}}
export DEXTER_MODEL="{{model}}"
echo "✓ Using model: {{model}}"
{{else}}
export DEXTER_MODEL="${DEXTER_MODEL:-gpt-4.1}"
echo "✓ Using default model: $DEXTER_MODEL"
{{/if}}

# Configure step limits for safety
{{#if max_steps}}
export DEXTER_MAX_STEPS="{{max_steps}}"
{{else}}
export DEXTER_MAX_STEPS="${DEXTER_MAX_STEPS:-20}"
{{/if}}

{{#if max_steps_per_task}}
export DEXTER_MAX_STEPS_PER_TASK="{{max_steps_per_task}}"
{{else}}
export DEXTER_MAX_STEPS_PER_TASK="${DEXTER_MAX_STEPS_PER_TASK:-5}"
{{/if}}

echo "✓ Max steps: $DEXTER_MAX_STEPS, Max per task: $DEXTER_MAX_STEPS_PER_TASK"

# Verify API keys
if [ -z "$OPENAI_API_KEY" ] && [ -z "$ANTHROPIC_API_KEY" ] && [ -z "$GOOGLE_API_KEY" ]; then
    echo "❌ ERROR: No LLM API key found. Set OPENAI_API_KEY, ANTHROPIC_API_KEY, or GOOGLE_API_KEY"
    exit 1
fi

if [ -z "$FINANCIAL_DATASETS_API_KEY" ]; then
    echo "⚠️ WARNING: FINANCIAL_DATASETS_API_KEY not set. Financial research tools may not work."
fi

# Determine prompt source
if [ -n "${PROMPT_CONTENT:-}" ]; then
    USER_QUERY="$PROMPT_CONTENT"
elif [ -f "$DEXTER_WORK_DIR/task/prompt.md" ]; then
    USER_QUERY=$(cat "$DEXTER_WORK_DIR/task/prompt.md")
    echo "✓ Using prompt from task/prompt.md"
else
    echo "❌ ERROR: No prompt content available"
    exit 1
fi

echo ""
echo "🔍 Query: ${USER_QUERY:0:100}..."
echo ""

# Create Python script to run Dexter programmatically
cat > /tmp/run_dexter.py << 'DEXTER_SCRIPT'
import os
import sys
from dexter.agent import Agent

def main():
    model = os.environ.get('DEXTER_MODEL', 'gpt-4.1')
    max_steps = int(os.environ.get('DEXTER_MAX_STEPS', '20'))
    max_steps_per_task = int(os.environ.get('DEXTER_MAX_STEPS_PER_TASK', '5'))
    
    query = sys.argv[1] if len(sys.argv) > 1 else sys.stdin.read().strip()
    
    if not query:
        print("Error: No query provided", file=sys.stderr)
        sys.exit(1)
    
    agent = Agent(
        max_steps=max_steps,
        max_steps_per_task=max_steps_per_task,
        model=model
    )
    
    try:
        answer = agent.run(query)
        print("\n" + "="*60)
        print("FINAL ANSWER")
        print("="*60)
        print(answer)
    except KeyboardInterrupt:
        print("\nOperation cancelled.")
        sys.exit(130)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
DEXTER_SCRIPT

# Execute Dexter
echo "🚀 Starting Dexter agent..."
python3 /tmp/run_dexter.py "$USER_QUERY"
DEXTER_EXIT_CODE=$?

echo ""
echo "✓ Dexter execution completed (exit code: $DEXTER_EXIT_CODE)"
exit $DEXTER_EXIT_CODE
```

### Phase 3: Helm Integration (Week 2)

#### 3.1 Update Controller Values

Add to `infra/charts/controller/values.yaml`:

```yaml
agent:
  cliImages:
    # ... existing entries ...
    dexter: ghcr.io/5dlabs/dexter:latest

  cliVersions:
    # ... existing entries ...
    dexter: "1.0.1"  # From pyproject.toml

  cliDefaults:
    dexter:
      model: "gpt-4.1"
      maxSteps: 20
      maxStepsPerTask: 5
      temperature: 0.7
```

#### 3.2 Add Dexter-Specific Templates

Create `templates/agents/dexter-system-prompt.md.hbs`:

```markdown
# Dexter Financial Research Agent

You are Dexter, an autonomous financial research agent deployed within the 5D Labs platform.
Your primary objective is to conduct deep and thorough research on stocks and companies.

## Available Tools

You have access to:
- SEC filing retrieval (10-K, 10-Q, 8-K filings)
- Financial statement data (income statements, balance sheets, cash flow)
- Stock price data and historical quotes
- News and market sentiment analysis

## Task Context

{{#if task_description}}
Current Task: {{task_description}}
{{/if}}

{{#if additional_context}}
Additional Context:
{{additional_context}}
{{/if}}

## Guidelines

1. Break complex questions into manageable research steps
2. Use available tools strategically to gather data
3. Verify findings before synthesizing answers
4. Provide specific numbers with proper context
5. Cite data sources when multiple sources are used
```

### Phase 4: Local Testing (Week 2)

#### 4.1 Local Docker Build Script

Create `scripts/build-dexter-image.sh`:

```bash
#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔨 Building Dexter agent image..."

docker build \
  -t ghcr.io/5dlabs/dexter:local \
  -f "$REPO_ROOT/infra/images/dexter/Dockerfile" \
  "$REPO_ROOT/infra/images/dexter"

echo "✅ Dexter image built: ghcr.io/5dlabs/dexter:local"
```

#### 4.2 Local Test Script

Create `scripts/test-dexter-local.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Ensure required env vars
if [ -z "${OPENAI_API_KEY:-}" ]; then
    echo "❌ OPENAI_API_KEY is required"
    exit 1
fi

if [ -z "${FINANCIAL_DATASETS_API_KEY:-}" ]; then
    echo "⚠️ FINANCIAL_DATASETS_API_KEY not set - financial tools may not work"
fi

# Test query
TEST_QUERY="${1:-What was Apple's revenue growth over the last 4 quarters?}"

echo "🧪 Testing Dexter with query: $TEST_QUERY"
echo ""

docker run -it --rm \
  -e OPENAI_API_KEY="$OPENAI_API_KEY" \
  -e FINANCIAL_DATASETS_API_KEY="${FINANCIAL_DATASETS_API_KEY:-}" \
  -e ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-}" \
  ghcr.io/5dlabs/dexter:local \
  bash -c "echo '$TEST_QUERY' | python3 -c '
import sys
from dexter.agent import Agent
agent = Agent(max_steps=10, max_steps_per_task=3)
query = sys.stdin.read().strip()
print(agent.run(query))
'"
```

### Phase 5: CLI Adapter (Week 3)

#### 5.1 Add Dexter to CLI Factory

Update the CLI factory pattern to include Dexter:

```rust
// In controller/src/cli/factory.rs (conceptual)

pub enum CLIType {
    Claude,
    Codex,
    Opencode,
    Gemini,
    Dexter,  // NEW
}

impl CliFactory {
    pub fn create_adapter(cli_type: &str) -> Result<Box<dyn CliAdapter>> {
        match cli_type {
            "claude" => Ok(Box::new(ClaudeAdapter::new())),
            "codex" => Ok(Box::new(CodexAdapter::new())),
            "opencode" => Ok(Box::new(OpencodeAdapter::new())),
            "dexter" => Ok(Box::new(DexterAdapter::new())),
            _ => Err(anyhow!("Unsupported CLI type: {}", cli_type))
        }
    }
}
```

#### 5.2 Dexter Adapter Implementation

```rust
// Conceptual DexterAdapter

pub struct DexterAdapter;

impl CliAdapter for DexterAdapter {
    fn validate_model(&self, model: &str) -> Result<()> {
        // Dexter supports OpenAI, Anthropic, and Google models
        if model.starts_with("gpt-") 
            || model.starts_with("claude-")
            || model.starts_with("gemini-") {
            Ok(())
        } else {
            Err(anyhow!("Invalid Dexter model: {}. Use gpt-*, claude-*, or gemini-*", model))
        }
    }

    fn get_memory_filename(&self) -> &str {
        // Dexter uses file-based context management, not a memory file
        "dexter_context"
    }

    fn get_executable_name(&self) -> &str {
        "dexter-agent"
    }
}
```

### Phase 6: Secret Management (Week 3)

#### 6.1 Vault Secret Configuration

Create `infra/vault/secrets/dexter.yaml`:

```yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: dexter-secrets
  namespace: cto
spec:
  provider: vault
  parameters:
    vaultAddress: "http://vault.vault:8200"
    roleName: "cto-secrets-reader"
    objects: |
      - objectName: "OPENAI_API_KEY"
        secretPath: "secret/data/cto/dexter"
        secretKey: "openai_api_key"
      - objectName: "ANTHROPIC_API_KEY"
        secretPath: "secret/data/cto/dexter"
        secretKey: "anthropic_api_key"
      - objectName: "GOOGLE_API_KEY"
        secretPath: "secret/data/cto/dexter"
        secretKey: "google_api_key"
      - objectName: "FINANCIAL_DATASETS_API_KEY"
        secretPath: "secret/data/cto/dexter"
        secretKey: "financial_datasets_api_key"
      - objectName: "LANGSMITH_API_KEY"
        secretPath: "secret/data/cto/dexter"
        secretKey: "langsmith_api_key"
  secretObjects:
    - data:
        - key: OPENAI_API_KEY
          objectName: OPENAI_API_KEY
        - key: ANTHROPIC_API_KEY
          objectName: ANTHROPIC_API_KEY
        - key: GOOGLE_API_KEY
          objectName: GOOGLE_API_KEY
        - key: FINANCIAL_DATASETS_API_KEY
          objectName: FINANCIAL_DATASETS_API_KEY
        - key: LANGSMITH_API_KEY
          objectName: LANGSMITH_API_KEY
      secretName: dexter-api-keys
      type: Opaque
```

## Implementation Timeline

| Phase | Task | Duration | Dependencies |
|-------|------|----------|--------------|
| 1.1 | Create Dockerfile | 1 day | - |
| 1.2 | Create README | 0.5 day | 1.1 |
| 2.1 | Create invoke.sh.hbs template | 1 day | 1.1 |
| 3.1 | Update controller values.yaml | 0.5 day | 2.1 |
| 3.2 | Create system prompt template | 0.5 day | 2.1 |
| 4.1 | Local build script | 0.5 day | 1.1 |
| 4.2 | Local test script | 0.5 day | 4.1 |
| 5.1 | CLI factory update | 1 day | 4.2 |
| 5.2 | Dexter adapter | 1 day | 5.1 |
| 6.1 | Vault secrets | 0.5 day | 5.2 |

**Total Estimated Time:** ~3 weeks

## Testing Strategy

### Unit Tests

1. **Model Validation:** Test all supported model prefixes
2. **Adapter Interface:** Verify DexterAdapter implements all required methods
3. **Template Rendering:** Test invoke script template with various contexts

### Integration Tests

1. **Local Docker:** Build and run image locally with test queries
2. **API Integration:** Verify Financial Datasets API connectivity
3. **Multi-Model:** Test with OpenAI, Anthropic, and Google models

### E2E Tests

1. **Simple Query:** "What is Apple's current stock price?"
2. **Complex Analysis:** "Compare Microsoft and Google's operating margins for 2023"
3. **Multi-Step Research:** "Analyze Tesla's cash flow trends and provide investment insights"

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Financial Datasets API rate limits | Medium | High | Implement caching and rate limiting |
| LangChain compatibility issues | Low | Medium | Pin versions, test thoroughly |
| Large context window usage | Medium | Medium | Implement context summarization |
| Python dependency conflicts | Low | Low | Use isolated virtualenv |

### Operational Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| API key exposure | Low | Critical | Use Vault secrets, never log keys |
| Runaway execution | Low | Medium | Step limits, timeout enforcement |
| Cost overruns from API usage | Medium | Medium | Usage monitoring, budget alerts |

## Future Enhancements

1. **Custom Tool Integration:** Add MCP-compatible tools for Dexter
2. **Memory Persistence:** Implement DEXTER.md for session memory
3. **Specialized Agents:** Create finance-focused agent profiles
4. **Multi-Agent Workflows:** Integrate Dexter into Play orchestration
5. **TypeScript Port:** Consider using dexter-ts for consistency

## References

- [Dexter Repository](https://github.com/virattt/dexter)
- [Financial Datasets API](https://financialdatasets.ai/)
- [LangChain Documentation](https://python.langchain.com/)
- [Multi-CLI Integration Design](./multi-cli-integration-design.md)

