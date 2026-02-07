# Lobster Intake Migration with 5-Model Voting Committee

## Overview

Replace the current `intake-agent` TypeScript binary with a Lobster workflow + slim `intake-util` CLI. All LLM calls move to `llm-task` plugin steps in Lobster. All deterministic template rendering stays in a compiled `intake-util` binary. A 5-model voting committee replaces the existing Advocate-Adversary-Arbiter debate pattern.

## Architecture

### Current System

```
PRD text
  → intake-agent binary (Bun/TypeScript)
    → parse_prd (LLM call via Claude Agent SDK)
    → analyze_complexity (LLM call)
    → expand_task (LLM call)
    → generate_with_critic / generate_with_debate (LLM calls)
    → generate_docs (deterministic templates)
    → generate_prompts (deterministic templates)
  → JSON response on stdout
```

Everything runs inside one monolithic binary that mixes LLM orchestration with deterministic code.

### Target System

```
PRD text
  → intake.lobster.yaml (Lobster workflow)
    → llm-task: parse PRD → GeneratedTask[] JSON (schema-validated)
    → llm-task: analyze complexity → ComplexityAnalysis[] JSON
    → approval gate (human reviews task breakdown)
    → llm-task: expand tasks → GeneratedSubtask[] JSON
    → voting.lobster.yaml (sub-workflow, 5 models vote)
    → intake-util generate-docs (deterministic CLI)
    → intake-util generate-prompts (deterministic CLI)
  → files written to .tasks/docs/
```

LLM orchestration lives in Lobster YAML. Deterministic code lives in `intake-util`.

## Data Flow Contract

The JSON Schema for `GeneratedTask[]` is the critical contract between the LLM steps and the deterministic CLI. It sits at the boundary:

- **LLM side**: `llm-task` steps use the schema in their `schema:` parameter to validate model output
- **CLI side**: `intake-util` validates its stdin/file input against the same schema

### GeneratedTask Schema (from types.ts)

```typescript
interface GeneratedTask {
  id: number;
  title: string;             // includes "(AgentName - Stack)" hint
  description: string;
  status?: "pending" | "in_progress" | "done" | "blocked" | "deferred";
  dependencies: number[];
  priority?: "high" | "medium" | "low";
  details?: string;
  test_strategy?: string;    // aka testStrategy (camelCase variant)
  subtasks?: GeneratedSubtask[];
  decision_points?: DecisionPoint[];  // aka decisionPoints
}

interface GeneratedSubtask {
  id: number;
  title: string;
  description: string;
  status?: string;
  dependencies: number[];
  details?: string;
  test_strategy?: string;    // aka testStrategy
  subagent_type?: "implementer" | "reviewer" | "tester" | "researcher" | "documenter";
  parallelizable?: boolean;
}

interface DecisionPoint {
  id: string;
  category: "architecture" | "error-handling" | "data-model" | "api-design" | "ux-behavior" | "performance" | "security";
  description: string;
  options: string[];
  requires_approval: boolean;   // aka requiresApproval
  constraint_type: "hard" | "soft" | "open" | "escalation";  // aka constraintType
}
```

**Important**: The existing code handles both `snake_case` and `camelCase` variants (e.g., `task.decision_points || task.decisionPoints`). The JSON Schema should normalize to `snake_case` and the `intake-util` CLI should only accept `snake_case`. The `llm-task` schema enforces this at the LLM output boundary.

## Component 1: JSON Schema Files

Create formal JSON Schema files that serve as the single source of truth.

### Files to create

- `intake/schemas/generated-task.schema.json` -- the `GeneratedTask` type as JSON Schema
- `intake/schemas/generated-subtask.schema.json` -- the `GeneratedSubtask` type
- `intake/schemas/decision-point.schema.json` -- the `DecisionPoint` type
- `intake/schemas/complexity-analysis.schema.json` -- the `TaskComplexityAnalysis` type
- `intake/schemas/vote-ballot.schema.json` -- new, for voting committee output

### Source

Translate directly from `infra-operator-catalog/apps/intake-agent/src/types.ts`:
- Lines 219-230: `GeneratedTask`
- Lines 237-250: `GeneratedSubtask`
- Lines 207-214: `DecisionPoint`
- Lines 269-276: `TaskComplexityAnalysis`

## Component 2: LLM Prompt Files

Extract all prompt strings from existing TypeScript into standalone `.md` files. These become the `prompt:` parameter values for `llm-task` steps.

### Files to create

- `intake/prompts/parse-prd-system.md` -- from `parse-prd.ts` `getSystemPrompt()` (lines 57-128)
- `intake/prompts/parse-prd-user.md` -- from `parse-prd.ts` `getUserPrompt()` (lines 134-153) -- this is a template with `{{prd_content}}`, `{{num_tasks}}`, `{{next_id}}` placeholders
- `intake/prompts/expand-task-system.md` -- from `expand-task.ts` system prompt function
- `intake/prompts/expand-task-user.md` -- from `expand-task.ts` user prompt function
- `intake/prompts/analyze-complexity-system.md` -- from `analyze.ts` system prompt
- `intake/prompts/analyze-complexity-user.md` -- from `analyze.ts` user prompt
- `intake/prompts/vote-system.md` -- new, instructs a model to evaluate and score task quality
- `intake/prompts/vote-user.md` -- new, presents tasks for evaluation

### Source

- `infra-operator-catalog/apps/intake-agent/src/operations/parse-prd.ts` lines 57-153
- `infra-operator-catalog/apps/intake-agent/src/operations/expand-task.ts` (system/user prompt functions)
- `infra-operator-catalog/apps/intake-agent/src/operations/analyze.ts` (system/user prompt functions)

## Component 3: intake-util CLI

A slim compiled binary with **only** deterministic operations. No LLM calls, no provider code, no Claude SDK.

### Subcommands

1. **`generate-docs`** -- Takes `GeneratedTask[]` JSON, writes task documentation files
2. **`generate-prompts`** -- Takes `GeneratedTask[]` JSON, writes prompt/acceptance files
3. **`tally`** -- Takes vote ballot JSON from 5 models, computes majority decision (new)

### Methods to include (verbatim copy from existing code)

**From `generate-docs.ts` (all of these, unchanged):**
- `generateTaskMarkdown(task: GeneratedTask): string`
- `generatePromptMarkdown(task: GeneratedTask): string`
- `generateAcceptanceMarkdown(task: GeneratedTask): string`
- `generateDecisionsMarkdown(task: GeneratedTask): string`
- `generatePromptXml(task: GeneratedTask): string`
- `generateTaskDocs(task, basePath): Promise<GeneratedDoc>`
- `generateDocs(payload): Promise<{success, generated_docs?, error?}>`

**From `generate-prompts.ts` (all of these, unchanged):**
- `generateTaskPromptMd(task: GeneratedTask, projectName: string): string`
- `generateTaskPromptXml(task: GeneratedTask, projectName: string): string`
- `generateAcceptanceMd(task: GeneratedTask): string`
- `generateSubtaskPromptMd(subtask, parentTaskId, projectName): string`
- `generatePrompts(payload): Promise<AgentResponse<GeneratePromptsData>>`
- `formatDecisionPoints(points?: DecisionPoint[]): string`
- `formatDecisionPointsXml(points?: DecisionPoint[]): string`
- `formatDecisionPointsForAcceptance(points?: DecisionPoint[]): string`
- `escapeXml(str: string): string`
- `extractAgent(title: string): { agent: string; stack: string }`

**New (tally subcommand):**
- `tallyVotes(ballots: VoteBallot[]): TallyResult` -- majority vote across 5 model responses

### What gets deleted (NOT included in intake-util)

- All of `parse-prd.ts` -- replaced by `llm-task`
- All of `parse-prd-iterative.ts` -- replaced by `llm-task`
- All of `expand-task.ts` -- replaced by `llm-task`
- All of `analyze.ts` -- replaced by `llm-task`
- All of `generate.ts` -- replaced by `llm-task`
- All of `generate-with-critic.ts` -- replaced by voting workflow
- All of `generate-with-debate.ts` -- replaced by voting workflow
- All of `research.ts` -- replaced by `llm-task`
- All of `providers/*.ts` (claude.ts, minimax.ts, codex.ts, anthropic.ts) -- no LLM calls needed
- `index.ts` router -- replaced by Lobster workflow orchestration
- Claude Agent SDK dependency -- not needed
- `cli-finder.ts` -- not needed
- `utils/timeout.ts` -- not needed (no LLM calls to timeout)

### CLI interface

```bash
# Read task JSON from file, write docs to output directory
intake-util generate-docs --task-json tasks.json --base-path .tasks/docs

# Read task JSON from stdin
cat tasks.json | intake-util generate-docs --base-path .tasks/docs

# Generate prompts
intake-util generate-prompts --task-json tasks.json --output-dir .tasks/docs --project-name myproject

# Tally votes from 5 model ballots
intake-util tally --ballots-json votes.json
```

### Build

Compile with Bun to a single binary (no runtime dependency):

```bash
bun build src/index.ts --compile --outfile intake-util
```

## Component 4: Voting Sub-Workflow (voting.lobster.yaml)

A Lobster sub-workflow that sends the same prompt to 5 different models and tallies votes.

### 5-Model Committee

| Voter | Provider | Model |
|-------|----------|-------|
| voter-1 | anthropic | claude-sonnet-4-20250514 |
| voter-2 | openai | gpt-4.1 |
| voter-3 | anthropic | claude-haiku-4-20250514 |
| voter-4 | minimax | minimax-01 |
| voter-5 | openai | o3-mini |

(Models are configurable -- these are starting defaults)

### Workflow structure

```yaml
name: voting
description: "5-model voting committee for task quality evaluation"
inputs:
  - name: content_to_evaluate
    type: string
  - name: evaluation_criteria
    type: string

steps:
  - name: voter-1
    command: >
      openclaw.invoke --tool llm-task --action json --args-json '{
        "prompt": "{{prompts/vote-system.md}}",
        "input": { "content": "{{inputs.content_to_evaluate}}", "criteria": "{{inputs.evaluation_criteria}}" },
        "schema": "{{schemas/vote-ballot.schema.json}}",
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514"
      }'

  - name: voter-2
    command: >
      openclaw.invoke --tool llm-task --action json --args-json '{
        "prompt": "{{prompts/vote-system.md}}",
        "input": { "content": "{{inputs.content_to_evaluate}}", "criteria": "{{inputs.evaluation_criteria}}" },
        "schema": "{{schemas/vote-ballot.schema.json}}",
        "provider": "openai",
        "model": "gpt-4.1"
      }'

  # ... voter-3, voter-4, voter-5 similarly

  - name: tally
    command: >
      intake-util tally --ballots-json <(echo '[
        ${{ steps.voter-1.output }},
        ${{ steps.voter-2.output }},
        ${{ steps.voter-3.output }},
        ${{ steps.voter-4.output }},
        ${{ steps.voter-5.output }}
      ]')
```

Note: The 5 voter steps can run in parallel since they have no dependencies on each other.

## Component 5: Main Intake Workflow (intake.lobster.yaml)

The top-level pipeline that replaces `intake-agent`.

### Workflow structure

```yaml
name: intake
description: "PRD intake pipeline with voting committee"
inputs:
  - name: prd_content
    type: string
  - name: prd_path
    type: string
  - name: num_tasks
    type: number
    default: 10
  - name: project_name
    type: string
    default: "project"

steps:
  - name: parse-prd
    command: >
      openclaw.invoke --tool llm-task --action json --args-json '{
        "prompt": "{{prompts/parse-prd-system.md}}\n\n{{prompts/parse-prd-user.md}}",
        "input": {
          "prd_content": "{{inputs.prd_content}}",
          "num_tasks": {{inputs.num_tasks}},
          "next_id": 1
        },
        "schema": "{{schemas/generated-task.schema.json}}",
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514"
      }'

  - name: analyze-complexity
    command: >
      openclaw.invoke --tool llm-task --action json --args-json '{
        "prompt": "{{prompts/analyze-complexity-system.md}}",
        "input": { "tasks": ${{ steps.parse-prd.output }} },
        "schema": "{{schemas/complexity-analysis.schema.json}}",
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514"
      }'

  - name: review-tasks
    approval:
      message: "Review generated tasks and complexity analysis before expanding"
      show:
        - tasks: ${{ steps.parse-prd.output }}
        - complexity: ${{ steps.analyze-complexity.output }}

  - name: expand-tasks
    command: >
      openclaw.invoke --tool llm-task --action json --args-json '{
        "prompt": "{{prompts/expand-task-system.md}}",
        "input": {
          "tasks": ${{ steps.parse-prd.output }},
          "complexity": ${{ steps.analyze-complexity.output }}
        },
        "schema": "{{schemas/generated-task.schema.json}}",
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514"
      }'

  - name: vote-on-quality
    workflow: voting.lobster.yaml
    inputs:
      content_to_evaluate: ${{ steps.expand-tasks.output }}
      evaluation_criteria: "task decomposition quality, dependency ordering, decision point coverage"

  - name: generate-docs
    command: >
      echo '${{ steps.expand-tasks.output }}' | intake-util generate-docs --base-path .tasks/docs

  - name: generate-prompts
    command: >
      echo '${{ steps.expand-tasks.output }}' | intake-util generate-prompts --output-dir .tasks/docs --project-name {{inputs.project_name}}
```

## Component 6: OpenClaw Configuration

Enable the `llm-task` plugin with all 5 voting models allowed.

```json
{
  "plugins": {
    "entries": {
      "llm-task": {
        "enabled": true,
        "config": {
          "defaultProvider": "anthropic",
          "defaultModel": "claude-sonnet-4-20250514",
          "allowedModels": [
            "anthropic/claude-sonnet-4-20250514",
            "anthropic/claude-haiku-4-20250514",
            "openai/gpt-4.1",
            "openai/o3-mini",
            "minimax/minimax-01"
          ],
          "maxTokens": 16000,
          "timeoutMs": 120000
        }
      },
      "lobster": { "enabled": true }
    }
  },
  "agents": {
    "list": [
      {
        "id": "intake",
        "tools": { "allow": ["llm-task", "lobster"] }
      }
    ]
  }
}
```

## Testing Plan

### Phase 1: Deterministic Output Parity

Verify that `intake-util` produces identical output to the existing `intake-agent` for the same task JSON input.

**Method:**
1. Capture 3 representative `GeneratedTask[]` JSON payloads from existing intake runs (small PRD, medium PRD, large PRD)
2. Run each through existing system:
   ```bash
   echo '{"operation":"generate_docs","payload":{"tasks":TASKS_JSON,"base_path":"/tmp/old"}}' | intake-agent
   echo '{"operation":"generate_prompts","payload":{"tasks":TASKS_JSON,"output_dir":"/tmp/old-prompts"}}' | intake-agent
   ```
3. Run each through new system:
   ```bash
   intake-util generate-docs --task-json tasks.json --base-path /tmp/new
   intake-util generate-prompts --task-json tasks.json --output-dir /tmp/new-prompts
   ```
4. Compare:
   ```bash
   diff -r /tmp/old /tmp/new        # should be empty (byte-identical)
   diff -r /tmp/old-prompts /tmp/new-prompts  # should be empty
   ```

**Pass criteria:** Zero diff. Byte-identical output.

### Phase 2: LLM Output Quality Comparison

Compare task quality between old `intake-agent` LLM calls and new `llm-task` steps.

**Method:**
1. Select 3 representative PRDs of varying complexity
2. Run each through old system: `echo '{"operation":"parse_prd",...}' | intake-agent`
3. Run each through new Lobster workflow: `lobster run intake.lobster.yaml`
4. Score both outputs on:
   - **Task count accuracy** (did it generate the requested number?)
   - **Dependency ordering** (are dependencies valid? no forward refs?)
   - **Agent assignment** (correct agent hints in titles?)
   - **Decision point coverage** (did it identify ambiguous areas?)
   - **Test strategy quality** (are acceptance criteria actionable?)
   - **JSON validity** (did schema validation catch issues?)
   - **Latency** (end-to-end time)
   - **Token usage** (total tokens consumed)

**Pass criteria:** New system scores >= old system on quality metrics. Latency within 2x.

### Phase 3: Voting Committee Evaluation

Test that the 5-model voting committee produces better decisions than the single-model approach.

**Method:**
1. Run 3 PRDs through old debate pattern (`generate_with_debate` with Advocate-Adversary-Arbiter)
2. Run same 3 PRDs through new voting workflow (5 models)
3. Have a human evaluator score both on:
   - Decision quality (did the voting catch issues the single arbiter missed?)
   - Diversity of perspectives (did different models surface different concerns?)
   - Consensus reliability (did the majority vote align with human judgment?)
4. Record per-model vote breakdowns to identify any consistently unhelpful voters

**Pass criteria:** Voting committee matches or exceeds human judgment alignment vs. single arbiter.

### Phase 4: End-to-End Integration

Verify the full Lobster workflow feeds correctly into the downstream Play workflow.

**Method:**
1. Run a PRD through the complete new Lobster intake pipeline
2. Feed the output into the existing Play workflow
3. Verify Play can read and process all generated files (task.md, prompt.md, prompt.xml, acceptance.md, decisions.md)
4. Verify no downstream errors from schema mismatches

**Pass criteria:** Play workflow starts successfully with no parse errors on intake output.

## Implementation Order

1. **Create JSON Schema files** -- `intake/schemas/*.schema.json`
2. **Extract prompt files** -- `intake/prompts/*.md`
3. **Build intake-util CLI** -- copy template methods verbatim, add tally subcommand, compile
4. **Run Phase 1 tests** -- verify deterministic parity before proceeding
5. **Write voting.lobster.yaml** -- 5-model voting sub-workflow
6. **Write intake.lobster.yaml** -- main intake pipeline
7. **Configure OpenClaw** -- enable llm-task plugin with allowed models
8. **Run Phase 2 tests** -- compare LLM output quality
9. **Run Phase 3 tests** -- evaluate voting committee
10. **Run Phase 4 tests** -- end-to-end integration with Play

## File Inventory

### New files to create

```
intake/
  schemas/
    generated-task.schema.json
    generated-subtask.schema.json
    decision-point.schema.json
    complexity-analysis.schema.json
    vote-ballot.schema.json
  prompts/
    parse-prd-system.md
    parse-prd-user.md
    expand-task-system.md
    expand-task-user.md
    analyze-complexity-system.md
    analyze-complexity-user.md
    vote-system.md
    vote-user.md
  workflows/
    intake.lobster.yaml
    voting.lobster.yaml
  util/
    src/
      index.ts              (CLI entry point with subcommand routing)
      generate-docs.ts       (verbatim from intake-agent)
      generate-prompts.ts    (verbatim from intake-agent)
      tally.ts               (new: vote tallying)
      types.ts               (subset of intake-agent types, snake_case only)
    package.json
    tsconfig.json
  config/
    openclaw-llm-task.json   (plugin configuration)
  tests/
    fixtures/
      small-prd.md
      medium-prd.md
      large-prd.md
      tasks-small.json       (captured from existing system)
      tasks-medium.json
      tasks-large.json
    parity-test.sh           (Phase 1: diff old vs new)
    quality-test.sh          (Phase 2: LLM comparison)
    voting-test.sh           (Phase 3: committee evaluation)
    integration-test.sh      (Phase 4: Play compatibility)
```

### Existing files referenced

```
infra-operator-catalog/apps/intake-agent/src/
  types.ts                    (GeneratedTask, GeneratedSubtask, DecisionPoint interfaces)
  operations/
    generate-docs.ts          (template methods to copy verbatim)
    generate-prompts.ts       (template methods to copy verbatim)
    parse-prd.ts              (prompt strings to extract, then discard)
    expand-task.ts            (prompt strings to extract, then discard)
    analyze.ts                (prompt strings to extract, then discard)
    generate-with-debate.ts   (debate pattern to study for voting design)
  providers/
    types.ts                  (ProviderName type for reference)
```

### Files to eventually retire

The entire `infra-operator-catalog/apps/intake-agent/` directory once migration is validated. Keep it around during testing for side-by-side comparison, then archive.
