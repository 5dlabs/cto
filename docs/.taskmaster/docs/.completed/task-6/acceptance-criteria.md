# Acceptance Criteria: QA Enforces Documentation Updates Before Approval

## Functional Requirements


- [ ] Detect implementation changes affecting docs (API, config, workflows, controller)


- [ ] Verify related docs updated in `docs/` (guides, references, examples)


- [ ] Require PR to include docs changes or an explicit rationale note
- [ ] Post a checklist comment if missing: files/sections to update


- [ ] Block PR approval until docs check passes

## Integration Requirements


- [ ] Runs in QA stage before Tess approval (waiting-pr-approved)


- [ ] Operates as part of Cleo QA agent (no new GitHub App)


- [ ] Reports status via PR comment and GitHub Check output summary


- [ ] Respects repository `CODEOWNERS` for required docs approval if present

## Detection Logic
- [ ] Parse git diff for changes in: `controller/**`, `infra/**`, `docs/**`, `mcp/**`
- [ ] Map changed areas to expected docs:


  - controller changes → `docs/engineering/*` and `docs/references/*`


  - charts/CRDs changes → `infra/charts/**` docs and `docs/references/*`


  - workflows/sensors → `docs/references/argo-events/*`


  - public APIs/config → `docs/README.md`, `docs/examples/*`


- [ ] If no docs changes found, compute “likely impacted docs” list

## Output Requirements
- [ ] Structured PR comment with sections:


  - Summary of detected change areas


  - Required docs to update (paths)


  - Suggested headings/anchors to modify


  - Rationale if docs not needed (template provided)


- [ ] Exit non-zero in QA step if docs missing and no rationale provided



## Test Cases


- [ ] PR modifies controller logic, no docs → comment lists engineering + references


- [ ] PR modifies Argo Events sensors → comment cites argo-events references


- [ ] PR modifies Helm chart values → comment cites chart docs and values


- [ ] PR only refactors comments → allows passing without docs


- [ ] PR includes docs + code → passes with success status

## Performance & Reliability


- [ ] Analysis completes < 10s for typical PRs


- [ ] Handles large diffs gracefully


- [ ] Idempotent comments (updates existing instead of duplicates)

## Security & Governance


- [ ] No secrets exposed in comments


- [ ] Follows repo governance (CODEOWNERS) if enforced


- [ ] Log links to changed files only



## Success Metrics


1. 95% of PRs that change behavior include docs updates or rationale


2. <1% false positives requiring docs when not needed


3. <10s analysis time median