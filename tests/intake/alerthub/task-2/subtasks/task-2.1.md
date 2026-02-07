# Subtask 2.1: Design User Schema and Migrations

## Parent Task
Task 2

## Agent
db-designer

## Parallelizable
Yes

## Description
Design and implement PostgreSQL schema for user authentication and authorization.

## Details
- Create users table with email, password_hash, status, roles
- Design sessions table for JWT token refresh_tokens table with storage
- Create rotation tracking
- Design profile_data JSONB column for extensibility
- Create indexes for email lookups and session queries
- Implement migrations with go-migrate

## Deliverables
- `migrations/` directory with SQL migration files
- `schema.sql` - Complete schema definition
- `models.go` - GORM/Ent models

## Acceptance Criteria
- [ ] Users table created with all required fields
- [ ] Indexes support email lookups
- [ ] Migrations can be applied/reverted
- [ ] Refresh token rotation supported

## Testing Strategy
- Apply migrations to test database
- Verify schema constraints
- Test rollback procedure
