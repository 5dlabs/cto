# Acceptance Criteria: Task 9

- [ ] Create REST API endpoints for task CRUD operations with filtering and soft delete
- [ ] Integration tests for CRUD operations, filtering, and soft delete functionality with cleanup verification
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 9.1: Create tasks.rs handler module and basic structure
- [ ] 9.2: Implement POST /api/teams/:team_id/tasks endpoint
- [ ] 9.3: Implement GET /api/teams/:team_id/tasks with filtering
- [ ] 9.4: Implement GET, PATCH, DELETE /api/tasks/:id endpoints
- [ ] 9.5: Implement task status enum validation
- [ ] 9.6: Implement soft delete with deleted_at timestamp
- [ ] 9.7: Implement 30-day cleanup job for deleted tasks
