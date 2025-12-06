# Acceptance Criteria: Task 17

- [ ] Create PostgreSQL schema for users, teams, team_members, tasks, and invite_links tables with proper indexes and constraints. Implement soft delete pattern for tasks.
- [ ] Run sqlx migrate run and verify all tables exist with correct schema using psql. Test foreign key constraints by attempting invalid inserts. Verify indexes exist with EXPLAIN ANALYZE queries.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 17.1: Create users table migration with authentication fields
- [ ] 17.2: Create teams table with ownership relationships
- [ ] 17.3: Create team_members junction table with role enum and constraints
- [ ] 17.4: Create tasks table with soft delete pattern and status enum
- [ ] 17.5: Create invite_links table with expiration logic
- [ ] 17.6: Validate schema integrity and run all migrations
