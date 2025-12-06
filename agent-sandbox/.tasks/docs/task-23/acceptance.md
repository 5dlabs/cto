# Acceptance Criteria: Task 23

- [ ] Create task management endpoints with status/assignee/date range filtering and soft delete with 30-day retention.
- [ ] Integration tests: create task, verify in list. Filter by status, assignee, date range. Update task status. Soft delete task, verify not in list but exists in DB. Test hard delete after 30 days. Verify member can create, only admin can delete. Test assignee validation.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 23.1: Create Task domain models with status enum and filter struct
- [ ] 23.2: Implement task repository with complex filtering queries
- [ ] 23.3: Create task creation endpoint with assignee validation
- [ ] 23.4: Implement task listing endpoint with multi-parameter filtering
- [ ] 23.5: Create task update endpoint with authorization checks
- [ ] 23.6: Implement soft delete with deleted_at timestamp
- [ ] 23.7: Create background job for hard deletion after 30-day retention
