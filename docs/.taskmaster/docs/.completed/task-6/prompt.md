# Autonomous Agent Prompt: QA Enforces Documentation Updates Before Approval

## Mission
Enhance the QA stage so that before PR approval, the QA (Cleo) agent verifies that any implementation changes are reflected in project documentation, or that a rationale is provided when docs are not needed.

## Constraints


- Do not create a new GitHub App. Use the existing QA agent context (Cleo).


- Integrate into the QA stage right before Tess approval.


- Avoid blocking legitimate refactors that require no docs; allow rationale template.



## Objectives


1. Detect code changes that likely require documentation updates.


2. Determine whether appropriate docs have been updated in `docs/`.


3. If missing, post a structured PR comment listing required doc updates and provide a rationale template.


4. Fail the QA step until docs are updated or a rationale checkbox is checked in the PR comment.

## Technical Approach

### Inputs


- PR diff (changed files, added/removed paths)


- Repo tree under `docs/` for cross-referencing


- CODEOWNERS (optional) for required reviewers



### Steps
1. Classify change areas by paths:


   - `controller/**` → engineering and controller references


   - `infra/**` (charts/CRDs/templates) → infra docs and references


   - `docs/**` → may satisfy docs requirement


   - `mcp/**` → CLI/tooling docs
2. Build expected docs list per change area:


   - engineering reports or design notes under `docs/engineering/`


   - references under `docs/references/`


   - examples under `docs/examples/`


   - README/usage guides under `docs/`


3. Check if PR adds/updates any expected docs paths.
4. If none found:
   - Post/update a PR comment with:


     - Summary of change areas


     - Required docs checklist with file suggestions


     - Rationale template block
     - Instruction: “Check ‘Docs not required’ with rationale to proceed, or push docs updates.”


5. Expose a GitHub Check output summary mirroring the comment.


6. Exit non-zero to block QA approval when docs missing and rationale not confirmed.

### PR Comment Template






```
QA Docs Check

Detected changes suggest docs updates are needed:
- Areas: {{areas}}
- Suggested docs:


  - {{path 1}}


  - {{path 2}}

Actions:


- [ ] I updated the docs listed above
- [ ] Docs not required; rationale:
  > {{provide brief reasoning}}

Notes:


- Align with docs under docs/references/** and docs/engineering/** as applicable.






```

### Non-Blocking Cases


- Comment-only changes


- Pure rename/move without semantic changes


- Tests-only changes

## Integration Points


- Triggered in QA stage (Argo Workflows node before `waiting-pr-approved` resume)


- Use GitHub App auth of QA agent (Cleo) to read diff and post comments

## Verification
- Unit: path classification, doc suggestion mapping
- Integration: end-to-end on test PRs with/without docs
- Operational: avoids duplicate comments, updates existing comment



## Success Criteria


- Documents are updated or rationale provided for 95% of behavior-changing PRs


- Minimal false positives


- <10s analysis time for typical PR sizes