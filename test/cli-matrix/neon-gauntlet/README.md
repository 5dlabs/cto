# Neon Gauntlet ⚡🧤

Multi-CLI E2E test suite for the CTO controller pipeline.

## What it tests

- All 8 CLI types spawn pods with correct config
- Remote skills (23 per agent) are fetched, extracted, and available
- Discord rate limiting is mitigated via stagger + selective disable
- Datadog telemetry attributes are properly extracted
- Pod isolation (`.task-env`, workspace dirs) works across concurrent runs

## CLI Matrix

| CLI | Provider | Model | Discord |
|-----|----------|-------|---------|
| Claude | Anthropic | claude-sonnet-4-6 | yes |
| Codex | OpenAI | codex-mini-latest | no |
| OpenCode | Fireworks | qwen3-235b-a22b | no |
| Factory | ZhipuAI | qwen3-235b-a22b | no |
| Cursor | Google | gemini-2.5-flash | no |
| Gemini | Google | gemini-2.5-flash | no |
| Copilot | Anthropic | claude-sonnet-4-6 | no |
| Kimi | Moonshot | kimi-k2p5-turbo | no |

## Usage

```bash
# Apply all test CRDs
kubectl apply -f test/cli-matrix/neon-gauntlet/

# Watch pods
kubectl get pods -n cto -l component=code-runner -w
```
