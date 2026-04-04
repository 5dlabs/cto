## Acceptance Criteria

- [ ] 1. Component test: TaskCard rendered with `delegate_id='user_123'` and `agent='nova'` displays 'Nova' badge in green and shows the avatar. 2. Component test: TaskCard rendered with `delegate_id=null` displays 'Unresolved' badge in amber. 3. Component test: TaskList with 5 tasks renders them in dependency order (no task appears before its dependencies). 4. Component test: Summary header shows correct counts — given 5 tasks with 4 assigned and 1 unresolved, displays '5 tasks, 4 assigned, 1 unresolved'. 5. Accessibility test: All interactive elements in TaskCard and TaskList are keyboard-navigable and have appropriate ARIA labels (tested via @testing-library/jest-dom axe integration).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.