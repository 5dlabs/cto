Implement subtask 3005: Implement database migrations and repository layer for all RMS entities

## Objective
Create PostgreSQL schema migrations for opportunities, projects, inventory items, crew members, assignments, and deliveries. Implement a repository layer with CRUD operations using pgx.

## Steps
1. Use golang-migrate or goose for migration management.
2. Create migration files:
   - 001_create_opportunities.up.sql: opportunities table with all PRD fields, indexes on status and client
   - 002_create_projects.up.sql: projects table with FK to opportunities, indexes on status and schedule dates
   - 003_create_inventory_items.up.sql: inventory_items table with barcode (unique index), serial_number, status, category index
   - 004_create_crew_members.up.sql: crew_members table with skills (jsonb), availability
   - 005_create_assignments.up.sql: assignments table with FKs to crew_members and projects, unique constraint on (crew_member_id, project_id, date_range) using exclusion constraint
   - 006_create_deliveries.up.sql: deliveries table with FK to projects, status enum, items (jsonb or junction table)
3. Implement repository interfaces and pgx-based implementations for each entity with Create, Get, List (with pagination), Update, Delete methods.
4. Add corresponding .down.sql migrations for rollback.
5. Run migrations on startup or via CLI flag.

## Validation
Migrations run forward and backward without errors on a clean database; repository CRUD operations work correctly against a test PostgreSQL instance; unique constraints and foreign keys are enforced; pagination returns correct subsets.