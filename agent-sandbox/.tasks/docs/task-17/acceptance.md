# Acceptance Criteria: Task 17

- [ ] Create PostgreSQL schema for teams, users, tasks, invites, and audit tables with proper indexes and constraints
- [ ] Run sqlx migrate run, verify schema with \d commands, test constraint violations, verify indexes exist
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 17.1: Create core entity tables migration (users, teams, team_members)
- [ ] 17.2: Implement tasks table with status enums and constraints
- [ ] 17.3: Create invites table and audit/supporting tables
- [ ] 17.4: Add performance indexes and validate foreign key relationships
