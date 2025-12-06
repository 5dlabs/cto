# Acceptance Criteria: Task 22

- [ ] Create comprehensive task management endpoints with status transitions, assignment, filtering, and soft delete functionality
- [ ] Integration tests for CRUD operations, verify filtering combinations, test soft delete and 30-day retention, verify authorization checks, test assignee validation
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 22.1: Create Task domain model and repository trait
- [ ] 22.2: Implement TaskRepository with sqlx and soft delete support
- [ ] 22.3: Implement task creation endpoint with validation and authorization
- [ ] 22.4: Implement task retrieval endpoints with filtering
- [ ] 22.5: Implement task update endpoint with status transitions and validation
- [ ] 22.6: Implement soft delete and Redis pub/sub event publishing
