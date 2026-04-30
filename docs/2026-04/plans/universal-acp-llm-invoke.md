# Universal ACP LLM Invoke Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Standardize CTO intake/Lobster LLM calls on a universal ACP/ACPX backend while preserving `intake/scripts/llm-invoke.sh` as the stable workflow-facing abstraction.

**Architecture:** Lobster workflows keep calling `llm-invoke.sh --tool llm-task ...`. `llm-invoke.sh` selects a backend in order: explicit `CTO_LLM_INVOKE_CMD`, ACPX (`acpx-llm-task.py`) when enabled/available, OpenClaw gateway fallback, then direct real-model adapter only when explicitly configured. The ACPX adapter translates the existing `llm-task` payload contract into an ACPX `agent exec -f prompt.md` call and validates JSON output.

**Tech Stack:** Bash, Python stdlib, ACPX CLI, Copilot/Gemini/Claude ACP agents, Lobster workflow YAML, shell regression tests.

---

## Task 1: Add ACPX llm-task wrapper contract tests

**Objective:** Prove the desired ACPX adapter behavior before implementation.

**Files:**
- Create: `intake/tests/acpx-llm-task-tests.sh`
- Test target: `intake/scripts/acpx-llm-task.py`

**Step 1: Write failing test**

Create `intake/tests/acpx-llm-task-tests.sh` with tests that:

1. create a fake `acpx` executable in a temp `PATH` that records its argv and prints JSON
2. call `intake/scripts/acpx-llm-task.py --tool llm-task --action json --args-file payload.json`
3. assert stdout is valid JSON from fake ACPX
4. assert fake ACPX argv includes `--format text`, `--model gpt-5.5`, `copilot exec -f <prompt-file>` for `provider=github-copilot`
5. assert prompt file includes task prompt, input JSON, and schema text
6. assert `--tool provider-capabilities --action json` reports providers without secret values
7. assert invalid ACPX JSON exits nonzero

**Step 2: Run test to verify failure**

Run:

```bash
cd /opt/data/workspace/cto
chmod +x intake/tests/acpx-llm-task-tests.sh
intake/tests/acpx-llm-task-tests.sh
```

Expected: FAIL because `intake/scripts/acpx-llm-task.py` does not exist.

---

## Task 2: Implement `acpx-llm-task.py`

**Objective:** Add the universal ACPX backend for the existing OpenClaw-compatible `llm-task` argv shape.

**Files:**
- Create: `intake/scripts/acpx-llm-task.py`

**Implementation requirements:**

- Parse:
  - `--tool llm-task`
  - `--tool provider-capabilities`
  - `--action json|text`
  - `--args-json`
  - `--args-file`
- Load payload fields:
  - `provider`
  - `model`
  - `prompt`
  - `input`
  - `schema`
- Provider-to-agent mapping:
  - `github-copilot`, `copilot`, `github` → `copilot`
  - `gemini`, `google`, `google-gemini` → `gemini`
  - `anthropic`, `claude` → `claude`
  - `openai`, `codex`, `gpt` → `codex`
  - `opencode`, `fireworks` → `opencode`
  - `factory` → `droid`
  - `cursor` → `cursor`
- Env overrides:
  - `ACPX_LLM_AGENT`
  - `ACPX_LLM_MODEL`
  - `ACPX_LLM_TIMEOUT`, default `300`
  - `ACPX_LLM_CWD`, default `${WORKSPACE:-$PWD}`
  - `ACPX_LLM_BIN`, default `acpx`
- Build prompt equivalent to `real-llm-invoke.py`.
- Write prompt to temp file.
- Run:

```bash
acpx --cwd "$cwd" --non-interactive-permissions deny --auth-policy skip --timeout "$timeout" --format text --model "$model" "$agent" exec -f "$prompt_file"
```

- For `cursor`, omit `--model`.
- For `--action json`, extract first valid JSON object/array from ACPX stdout and print compact JSON.
- For invalid JSON, print a concise stderr message and return nonzero.
- For `provider-capabilities`, return non-secret JSON based on `command -v acpx` and provider CLI presence.

**Step 2: Run test to verify pass**

Run:

```bash
intake/tests/acpx-llm-task-tests.sh
python3 -m py_compile intake/scripts/acpx-llm-task.py
```

Expected: PASS.

---

## Task 3: Add `llm-invoke.sh` backend-selection tests

**Objective:** Verify `llm-invoke.sh` prefers ACPX in auto mode without breaking explicit override semantics.

**Files:**
- Create: `intake/tests/llm-invoke-backend-tests.sh`
- Modify: `intake/scripts/llm-invoke.sh`

**Test cases:**

1. If `CTO_LLM_INVOKE_CMD` is set, `llm-invoke.sh` delegates exactly there.
2. If `CTO_LLM_INVOKE_BACKEND=acp`, it uses `intake/scripts/acpx-llm-task.py`.
3. If `CTO_LLM_INVOKE_BACKEND=auto`, `acpx` is on PATH, and no explicit command is set, it uses `acpx-llm-task.py`.
4. If `CTO_LLM_INVOKE_BACKEND=direct`, it uses `real-llm-invoke.py` only when requested.
5. If no backend is available, error message explains `CTO_LLM_INVOKE_BACKEND`, `CTO_LLM_INVOKE_CMD`, and OpenClaw fallback.

Use fake scripts in temp dirs so tests do not require real ACPX/Copilot.

**Step 2: Run test to verify failure**

Run:

```bash
chmod +x intake/tests/llm-invoke-backend-tests.sh
intake/tests/llm-invoke-backend-tests.sh
```

Expected: FAIL until `llm-invoke.sh` is updated.

---

## Task 4: Update `llm-invoke.sh` backend selection

**Objective:** Make ACPX the standardized preferred backend while retaining compatibility/fallbacks.

**Files:**
- Modify: `intake/scripts/llm-invoke.sh`

**Backend order:**

1. `CTO_LLM_INVOKE_CMD` explicit override — existing behavior unchanged.
2. `CTO_LLM_INVOKE_BACKEND=acp` — require/use `intake/scripts/acpx-llm-task.py`.
3. `CTO_LLM_INVOKE_BACKEND=auto` or unset — if `acpx` exists and `acpx-llm-task.py` is executable, use it.
4. `CTO_LLM_INVOKE_BACKEND=openclaw` or `CTO_LLM_INVOKE_FALLBACK_OPENCLAW` — use `openclaw-invoke-retry.sh` if `openclaw.invoke` exists.
5. `CTO_LLM_INVOKE_BACKEND=direct` — use `real-llm-invoke.py`.
6. Fail with explicit setup instructions.

**Step 2: Run tests**

Run:

```bash
intake/tests/llm-invoke-backend-tests.sh
intake/tests/acpx-llm-task-tests.sh
intake/tests/provider-routing-tests.sh
intake/tests/provider-capability-workflow-tests.sh
```

Expected: PASS.

---

## Task 5: Wire capability reporting to understand ACP backend

**Objective:** Ensure pipeline `load-config` captures ACP backend provider capabilities via existing `llm-invoke.sh`/`CTO_LLM_INVOKE_CMD` path without leaking secrets.

**Files:**
- Modify if needed: `intake/workflows/pipeline.lobster.yaml`
- Modify: `intake/tests/provider-capability-workflow-tests.sh`

**Requirements:**

- `provider-capabilities` should work when `CTO_LLM_INVOKE_CMD=./intake/scripts/acpx-llm-task.py`.
- Capability output includes `invoke: acpx` for ACP-backed providers.
- No token/env values are printed.

**Verification:**

```bash
intake/tests/provider-capability-workflow-tests.sh
```

---

## Task 6: Documentation and final verification

**Objective:** Document the standard path and verify no regressions.

**Files:**
- Create or update: `docs/2026-04/acp-llm-invoke-standard.md`

**Documentation points:**

- Workflows call `llm-invoke.sh`, not ACPX directly.
- ACPX is the preferred backend.
- OpenClaw gateway and direct API are fallback/compatibility paths.
- Required runtime tools for true Copilot: `acpx` + `copilot` + Copilot auth/OAuth.
- Example Sigma run env:

```bash
export CTO_LLM_INVOKE_BACKEND=acp
export INTAKE_DELIBERATION_VIDEO=0
```

**Final verification:**

```bash
python3 -m py_compile intake/scripts/acpx-llm-task.py intake/scripts/real-llm-invoke.py
intake/tests/acpx-llm-task-tests.sh
intake/tests/llm-invoke-backend-tests.sh
intake/tests/provider-routing-tests.sh
intake/tests/provider-capability-workflow-tests.sh
intake/tests/workflow-output-file-tests.sh
intake/tests/task-scope-tests.sh
intake/tests/design-provider-auto-tests.sh
intake/tests/deliberation-audio-tests.sh
git diff --check
```

---

## Rollout

Open a PR with the ACP standardization. After merge, run Sigma from an environment with `acpx` + `copilot` installed using:

```bash
export CTO_LLM_INVOKE_BACKEND=acp
export INTAKE_DELIBERATION_VIDEO=0
```

If ACPX/Copilot is not present, `llm-invoke.sh` should fail clearly instead of using the wrong provider silently.
