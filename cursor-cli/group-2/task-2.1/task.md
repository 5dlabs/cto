# Task 2.1 – Helm Values & ConfigMap Wiring for Cursor

## Dependencies
- Task 0.1 (requirements) and Task 1.1 (CLIType).

## Parallelization Guidance
- Coordinate with Task 1.2/1.3 to avoid merge conflicts in templates/ConfigMap generation.

## Task Prompt
Expose Cursor CLI settings through Helm so the controller pods mount Cursor templates and pass configuration to jobs.

Detailed actions:
1. `infra/charts/controller/values.yaml`
   - Add Cursor agents (Rex, Blaze, etc.) with fields `cli: "Cursor"`, `model`, `maxTokens`, `temperature`, `reasoningEffort` (mirroring Codex but adjusting model slug if Cursor uses different defaults).
   - Introduce `agent.cliImages.cursor` pointing to the Cursor worker image (once built/published).
   - Document `CURSOR_API_KEY` secret expectation in comments next to the new entries.
2. `infra/charts/controller/templates/task-controller-config.yaml`
   - Ensure `agentCliConfigs` merge logic normalises legacy entries and sets `settings.reasoningEffort` without losing existing data (reuse improvements we made for Codex earlier).
   - Inject Cursor configs so rendered YAML includes entries like:
     ```yaml
     agentCliConfigs:
       5DLabs-Rex:
         cliType: Cursor
         model: gpt-5
         maxTokens: 64000
         settings:
           approvalPolicy: never
           reasoningEffort: high
     ```
   - Keep indentation yamllint-compliant (lessons learned from recent linting errors).
3. Static template ConfigMap (`infra/charts/controller/agent-templates-static.yaml` or renamed equivalent):
   - Update the generator references so Cursor templates are included but ensure this file remains auto-generated (do **not** hand-edit the generated file; update the script/Make target instead).
4. Values schema (`infra/charts/controller/values.schema.json`):
   - Extend schema to allow `Cursor` in `agents.*.cli` and to document new fields if necessary.
5. Update docs (`infra/README.md`) to mention new values after Group 4 tasks are ready (coordinate with doc team).

## Acceptance Criteria
- `helm lint infra/charts/controller` succeeds with Cursor values enabled.
- `helm template` output shows Cursor agents under both `agents:` and `agentCliConfigs:` sections with `approvalPolicy: never` nested in settings.
- No yamllint indentation errors (run `helm template ... | yamllint` to confirm).
- Codex/Claude entries unaffected (diff should be additive).

## Implementation Notes / References
- Reuse the normalisation logic we recently added for Codex reasoning effort; ensure we don’t duplicate it.
- Remember that the generated static ConfigMap is auto-built via `scripts/generate-templates-configmap.sh`; modify script/inputs rather than committing generated output.
- When referencing secrets, follow the pattern already in place for `OPENAI_API_KEY` to keep security posture consistent.
