# Acceptance Criteria: Task 2

- [ ] Design and implement the database schema for teams, users, tasks, and invites with proper indexes and constraints using sqlx migrations
- [ ] Run `sqlx migrate run` against test database. Verify all tables created with `\dt` in psql. Test connection pool with simple SELECT query
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 2.1: Create core user and team tables with relationships
- [ ] 2.2: Implement tasks table with comprehensive indexing strategy
- [ ] 2.3: Create invites table with token management and expiration
- [ ] 2.4: Set up database connection pool and health check infrastructure
