# Acceptance Criteria: Task 6

- [ ] Implement task management endpoints with create, read, update, delete operations, status/assignee/date filtering, and 30-day soft delete retention
- [ ] Integration tests: create task, verify in list. Update status, verify change. Soft delete, verify not in default list but exists in DB. Test all filter combinations. Verify only team members can access tasks
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 6.1: Implement task creation endpoint with team membership validation
- [ ] 6.2: Build task listing endpoint with multi-parameter filtering
- [ ] 6.3: Create task retrieval and update endpoints with authorization
- [ ] 6.4: Implement soft delete functionality for tasks
- [ ] 6.5: Add background job for permanent deletion of expired soft-deleted tasks
