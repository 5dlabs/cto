# Task 6: QA Enforces Documentation Updates Before Approval

## Overview
Add a QA-stage check (under Cleo) to ensure PRs that change behavior include appropriate documentation updates, or provide an explicit rationale when docs are not required.

## Technical Context
- Runs during QA stage prior to Tess approval (`waiting-pr-approved`)
- Uses GitHub App auth under QA agent context
- Analyzes PR diff and repository docs structure

## Implementation Guide

### Phase 1: Change Classification
1. Implement path-based classifier:
   - `controller/**` → engineering + references
   - `infra/**` → charts/CRDs docs + references
   - `mcp/**` → tooling docs
   - `docs/**` → counts as docs update
2. Compute “expected docs” list per classification.

### Phase 2: Docs Presence Check
1. Scan diff for any updated files under expected docs paths.
2. If none present, generate suggestions with concrete paths and anchors.

### Phase 3: PR Feedback & Gate
1. Post or update a structured PR comment with:
   - Summary of change areas
   - Checklist of suggested docs paths
   - Rationale template (allow pass without docs if justified)
2. Surface a GitHub Check status with pass/fail and summary.
3. Exit non-zero when docs missing and no rationale checked.

### Phase 4: Workflow Integration
1. Add a QA task in the workflow before approval step to run the check.
2. Provide repo + PR context to the QA agent container via env.
3. Ensure idempotent re-runs update the existing comment.

## Code Examples

### Pseudocode
```bash
changed=$(gh api repos/:owner/:repo/pulls/:pr/files)
areas=$(classify_paths "$changed")
expected=$(map_expected_docs "$areas")
found=$(grep_changed_docs "$changed" "$expected")

if [ -z "$found" ]; then
  upsert_pr_comment "$areas" "$expected"
  if ! rationale_checked; then
    echo "Docs check failed" >&2
    exit 1
  fi
fi

echo "Docs check passed"
```

### Workflow Hook (conceptual)
```yaml
- name: qa-docs-check
  container:
    image: your/qa-agent:tag
    env:
      - name: GITHUB_TOKEN
        valueFrom: {secretKeyRef: {name: {{qa-app-secret}}, key: token}}
      - name: PR_NUMBER
        value: "{{workflow.parameters.pr-number}}"
```

## Testing Strategy
- Unit tests for classification and mapping
- Integration test on synthetic PRs with/without docs
- Ensure idempotent comment updates

## Success Metrics
- High compliance of docs updates for behavior-changing PRs
- Low false positives
- Fast execution under typical PR sizes