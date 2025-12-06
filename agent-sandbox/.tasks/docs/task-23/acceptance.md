# Acceptance Criteria: Task 23

- [ ] Create background job to permanently delete tasks older than 30 days from soft deletion
- [ ] Unit test with mock database, integration test with test data, verify tasks deleted after 30 days, verify tasks within 30 days retained
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 23.1: Add tokio-cron-scheduler dependency to Cargo.toml
- [ ] 23.2: Create infra/jobs directory structure
- [ ] 23.3: Implement cleanup_deleted_tasks function
- [ ] 23.4: Add logging infrastructure for cleanup job
- [ ] 23.5: Implement metrics counter for deleted tasks
- [ ] 23.6: Create background task scheduler in main.rs
- [ ] 23.7: Add configuration for cleanup interval
- [ ] 23.8: Create integration tests for cleanup job
- [ ] 23.9: Add graceful shutdown handling for cleanup job
- [ ] 23.10: Document cleanup job behavior and configuration
