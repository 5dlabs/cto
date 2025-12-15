# AlertHub CLI/Provider Test Matrix

## Overview

This document defines the CLI and model provider rotation strategy for the AlertHub E2E test. The goal is to ensure comprehensive coverage of all supported CLIs and model providers across the full development workflow.

---

## Supported CLIs

| CLI | Description | Supported Providers | Best For |
|-----|-------------|---------------------|----------|
| **claude** | Anthropic's official Claude Code CLI | Anthropic | General coding, complex reasoning |
| **cursor** | AI-first code editor with MCP support | Anthropic, OpenAI | IDE-integrated development |
| **codex** | OpenAI's coding assistant | OpenAI | OpenAI model access |
| **factory** | Autonomous AI CLI for unattended execution | OpenAI, Anthropic | CI/CD, batch operations |
| **opencode** | Open-source CLI alternative | Anthropic, OpenAI | Community/extensibility |
| **gemini** | Google's Gemini CLI | Google | Google model access |
| **dexter** | Lightweight AI CLI | Various | Simple tasks |

---

## Supported Model Providers

| Provider | Models | API Key Env Var |
|----------|--------|-----------------|
| **Anthropic** | claude-sonnet-4-20250514, claude-opus-4-20250514 | `ANTHROPIC_API_KEY` |
| **OpenAI** | gpt-4o, gpt-4-turbo, o1-preview | `OPENAI_API_KEY` |
| **Google** | gemini-2.0-flash, gemini-1.5-pro | `GOOGLE_API_KEY` |

---

## Test Matrix

### Phase 1: Intake

| Task | Agent | CLI | Provider | Model | Rationale |
|------|-------|-----|----------|-------|-----------|
| PRD Parsing | Morgan | claude | Anthropic | claude-sonnet-4-20250514 | Best for complex document understanding |

### Phase 2: Implementation - Backend

| Task | Agent | CLI | Provider | Model | Rationale |
|------|-------|-----|----------|-------|-----------|
| Notification Router | Rex | factory | OpenAI | gpt-4o | Test Factory CLI with OpenAI |
| Integration Service | Nova | cursor | Anthropic | claude-sonnet-4-20250514 | Test Cursor with Anthropic |
| Admin API | Grizz | opencode | Anthropic | claude-sonnet-4-20250514 | Test OpenCode CLI |

### Phase 2: Implementation - Frontend

| Task | Agent | CLI | Provider | Model | Rationale |
|------|-------|-----|----------|-------|-----------|
| Web Console | Blaze | codex | OpenAI | gpt-4o | Test Codex with OpenAI |
| Mobile App | Tap | claude | Anthropic | claude-sonnet-4-20250514 | Test Claude CLI |
| Desktop Client | Spark | gemini | Google | gemini-2.0-flash | Test Gemini CLI |

### Phase 3: Quality

| Task | Agent | CLI | Provider | Model | Rationale |
|------|-------|-----|----------|-------|-----------|
| Code Review | Cleo | claude | Anthropic | claude-sonnet-4-20250514 | Quality requires strong reasoning |
| Security Scan | Cipher | cursor | Anthropic | claude-sonnet-4-20250514 | Security analysis |
| Test Suite | Tess | factory | OpenAI | gpt-4o | Automated test generation |

### Phase 4: Integration

| Task | Agent | CLI | Provider | Model | Rationale |
|------|-------|-----|----------|-------|-----------|
| PR Review | Stitch | claude | Anthropic | claude-sonnet-4-20250514 | Code review expertise |
| Merge | Atlas | claude | Anthropic | claude-sonnet-4-20250514 | Git operations |

### Phase 5: Deployment

| Task | Agent | CLI | Provider | Model | Rationale |
|------|-------|-----|----------|-------|-----------|
| Deploy | Bolt | claude | Anthropic | claude-sonnet-4-20250514 | Infrastructure expertise |

---

## CLI Coverage Summary

| CLI | Tasks Assigned | Percentage |
|-----|---------------|------------|
| claude | 6 | 46% |
| factory | 2 | 15% |
| cursor | 2 | 15% |
| opencode | 1 | 8% |
| codex | 1 | 8% |
| gemini | 1 | 8% |

---

## Provider Coverage Summary

| Provider | Tasks Assigned | Percentage |
|----------|---------------|------------|
| Anthropic | 10 | 77% |
| OpenAI | 2 | 15% |
| Google | 1 | 8% |

---

## Configuration Examples

### cto-config.json Agent Overrides

To run the test with specific CLI/provider combinations, configure your `cto-config.json`:

```json
{
  "version": "1.0",
  "agents": {
    "morgan": {
      "githubApp": "5DLabs-Morgan",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    },
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "factory",
      "model": "gpt-4o"
    },
    "nova": {
      "githubApp": "5DLabs-Nova",
      "cli": "cursor",
      "model": "claude-sonnet-4-20250514"
    },
    "grizz": {
      "githubApp": "5DLabs-Grizz",
      "cli": "opencode",
      "model": "claude-sonnet-4-20250514"
    },
    "blaze": {
      "githubApp": "5DLabs-Blaze",
      "cli": "codex",
      "model": "gpt-4o"
    },
    "tap": {
      "githubApp": "5DLabs-Tap",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    },
    "spark": {
      "githubApp": "5DLabs-Spark",
      "cli": "gemini",
      "model": "gemini-2.0-flash"
    },
    "cleo": {
      "githubApp": "5DLabs-Cleo",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    },
    "cipher": {
      "githubApp": "5DLabs-Cipher",
      "cli": "cursor",
      "model": "claude-sonnet-4-20250514"
    },
    "tess": {
      "githubApp": "5DLabs-Tess",
      "cli": "factory",
      "model": "gpt-4o"
    },
    "stitch": {
      "githubApp": "5DLabs-Stitch",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    },
    "atlas": {
      "githubApp": "5DLabs-Atlas",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    },
    "bolt": {
      "githubApp": "5DLabs-Bolt",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    }
  }
}
```

---

## MCP Tool Invocation Examples

### Intake with Specific CLI

```javascript
// Morgan uses claude CLI as configured
mcp_cto_intake({
  project_name: "alerthub-e2e-test",
  cli: "claude",  // Override if needed
  model: "claude-sonnet-4-20250514"
});
```

### Play with CLI Override

```javascript
// Override implementation agent CLI
mcp_cto_play({
  implementation_agent: "rex",
  cli: "factory",  // Force Factory CLI
  model: "gpt-4o"  // Force GPT-4o
});
```

### Check Play Status

```javascript
mcp_cto_play_status({
  repository: "5dlabs/agent-sandbox"
});
```

---

## Alternative Test Configurations

### All-Anthropic Configuration

For testing Anthropic provider coverage:

| Agent | CLI | Model |
|-------|-----|-------|
| All agents | claude | claude-sonnet-4-20250514 |

### All-OpenAI Configuration

For testing OpenAI provider coverage:

| Agent | CLI | Model |
|-------|-----|-------|
| All agents | factory | gpt-4o |

### All-Google Configuration

For testing Google provider coverage:

| Agent | CLI | Model |
|-------|-----|-------|
| All agents | gemini | gemini-2.0-flash |

### Mixed High-Performance Configuration

For production-quality output:

| Agent | CLI | Model | Notes |
|-------|-----|-------|-------|
| Morgan | claude | claude-opus-4-20250514 | Best reasoning |
| Rex | factory | o1-preview | Complex Rust code |
| Blaze | claude | claude-sonnet-4-20250514 | UI expertise |
| Bolt | claude | claude-sonnet-4-20250514 | Infrastructure |

---

## Validation Checklist

### CLI Functionality Tests

- [ ] **claude**: Agent completes task successfully
- [ ] **cursor**: Agent completes task successfully
- [ ] **codex**: Agent completes task successfully
- [ ] **factory**: Agent completes task successfully
- [ ] **opencode**: Agent completes task successfully
- [ ] **gemini**: Agent completes task successfully

### Provider Connectivity Tests

- [ ] **Anthropic**: API authentication works
- [ ] **OpenAI**: API authentication works
- [ ] **Google**: API authentication works

### Model Capability Tests

- [ ] **claude-sonnet-4**: Code generation quality acceptable
- [ ] **gpt-4o**: Code generation quality acceptable
- [ ] **gemini-2.0-flash**: Code generation quality acceptable

---

## Troubleshooting

### CLI-Specific Issues

| CLI | Common Issue | Resolution |
|-----|--------------|------------|
| claude | Rate limiting | Increase delay between tasks |
| cursor | MCP connection | Restart Cursor, verify mcp.json |
| codex | Token limits | Reduce context size |
| factory | Timeout | Increase max_retries |
| opencode | Config issues | Check .opencode config |
| gemini | Auth errors | Verify GOOGLE_API_KEY |

### Provider-Specific Issues

| Provider | Common Issue | Resolution |
|----------|--------------|------------|
| Anthropic | 429 Rate Limit | Enable model rotation |
| OpenAI | Context length | Use gpt-4-turbo for larger context |
| Google | Regional availability | Check API region settings |

---

## Metrics Collection

Track these metrics during the E2E test:

| Metric | Target | Collection Method |
|--------|--------|-------------------|
| Task completion time per CLI | < 30 min | Workflow timestamps |
| Success rate per CLI | > 95% | Task status tracking |
| Token usage per provider | Varies | API billing dashboard |
| Error rate per CLI | < 5% | Error log analysis |
| Model rotation triggers | 0 (ideally) | Rotation event logs |

---

## Notes

1. **CLI Selection**: The matrix prioritizes diversity over optimization. Production deployments should use the best-performing CLI for each agent type.

2. **Model Rotation**: If a model fails or rate limits, the platform automatically rotates to configured fallback models.

3. **Cost Optimization**: For cost-sensitive tests, use `claude-sonnet-4` and `gpt-4o` instead of Opus/o1-preview.

4. **Parallel Execution**: When `parallel_execution: true`, multiple agents may use the same CLI simultaneously. Ensure adequate API rate limits.

5. **Fallback Strategy**: Configure fallback models in `cto-config.json` for resilience:
   ```json
   {
     "modelRotation": {
       "enabled": true,
       "models": ["claude-sonnet-4-20250514", "gpt-4o", "gemini-2.0-flash"]
     }
   }
   ```

