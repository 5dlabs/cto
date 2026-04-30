# ACP LLM Invoke Standard

CTO intake workflows standardize on `intake/scripts/llm-invoke.sh` as the workflow-facing contract and ACP/ACPX as the preferred execution backend.

## Contract

Lobster workflow steps should continue calling:

```bash
intake/scripts/llm-invoke.sh \
  --tool llm-task \
  --action json \
  --args-json '{...}'
```

or:

```bash
intake/scripts/llm-invoke.sh \
  --tool llm-task \
  --action json \
  --args-file /path/to/payload.json
```

Do not call ACPX directly from each workflow step. Keeping `llm-invoke.sh` as the stable boundary lets the workflow run in both agency/OpenClaw environments and local/Hermes smoke environments.

## Backend order

`llm-invoke.sh` chooses a backend in this order:

1. `CTO_LLM_INVOKE_CMD` explicit override.
2. `CTO_LLM_INVOKE_BACKEND=acp` / `acpx`: use `intake/scripts/acpx-llm-task.py`.
3. `CTO_LLM_INVOKE_BACKEND=auto` or unset: prefer ACPX when `acpx` is available.
4. `CTO_LLM_INVOKE_BACKEND=openclaw` or legacy `CTO_LLM_INVOKE_FALLBACK_OPENCLAW`: use OpenClaw gateway compatibility.
5. `CTO_LLM_INVOKE_BACKEND=direct`: use the direct provider adapter.
6. Otherwise fail closed with setup guidance.

## ACPX adapter

`intake/scripts/acpx-llm-task.py` accepts the same OpenClaw-compatible argv shape as existing `llm-task` adapters. It maps provider/model payloads to ACPX agents and runs:

```bash
acpx \
  --cwd "$WORKSPACE" \
  --non-interactive-permissions deny \
  --auth-policy skip \
  --timeout 300 \
  --format text \
  --model "$MODEL" \
  "$AGENT" exec -f "$PROMPT_FILE"
```

For Cursor, the adapter omits `--model` because the existing ACPX path treats Cursor model selection specially.

Provider-to-agent defaults:

| Provider | ACPX agent |
| --- | --- |
| `github-copilot`, `copilot`, `github` | `copilot` |
| `gemini`, `google`, `google-gemini` | `gemini` |
| `anthropic`, `claude` | `claude` |
| `openai`, `codex`, `gpt` | `codex` |
| `opencode`, `fireworks` | `opencode` |
| `factory` | `droid` |
| `cursor` | `cursor` |

Overrides:

```bash
export ACPX_LLM_AGENT=copilot
export ACPX_LLM_MODEL=gpt-5.5
export ACPX_LLM_TIMEOUT=600
export ACPX_LLM_CWD=/workspace/repo
export ACPX_LLM_BIN=acpx
```

## True Copilot/GPT-5.5 runtime requirements

For a real Copilot-backed Sigma/FleetAxis run, the pod/image must provide:

- `acpx`
- `copilot`
- valid Copilot OAuth/auth state
- `CTO_LLM_INVOKE_BACKEND=acp`

The adapter reports non-secret capability booleans via:

```bash
intake/scripts/llm-invoke.sh --tool provider-capabilities --action json
```

Capability output must never include token or secret values.

## Sigma MP3-first/no-video smoke

Use:

```bash
export CTO_LLM_INVOKE_BACKEND=acp
export INTAKE_DELIBERATION_VIDEO=0
export INTAKE_AUTO_MIN_TASKS=12
```

If ACPX/Copilot is unavailable, the run should fail clearly instead of silently routing to Gemini or another direct API provider.
