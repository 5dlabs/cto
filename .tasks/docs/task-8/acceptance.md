## Acceptance Criteria

- [ ] 1. Component test: SnapshotPR rendered with a valid PR result displays the PR title, a clickable URL linking to `github.com/5dlabs/sigma-1/pull/N`, and an 'Open' badge in green. 2. Component test: SnapshotPR rendered with status='merged' displays a 'Merged' badge in purple. 3. Component test: SnapshotPR rendered with `pr_result=null` displays 'No snapshot PR created'. 4. Component test: PR URL link has `target='_blank'` and `rel='noopener noreferrer'` attributes. 5. Accessibility test: PR link is announced as an external link by screen readers.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.