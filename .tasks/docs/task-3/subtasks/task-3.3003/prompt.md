Implement subtask 3003: Create PostgreSQL schema migrations for the RMS schema

## Objective
Write SQL migration files for all RMS tables in the `rms` PostgreSQL schema, including opportunities, projects, inventory_items, crew_members, deliveries, and join tables.

## Steps
1. Use golang-migrate or goose for migration management. 2. Create migration 001: `CREATE SCHEMA IF NOT EXISTS rms;` 3. Migration 002: Create `rms.opportunities` table with columns matching the Opportunity proto message, plus tenant_id, timestamps, and soft delete. 4. Migration 003: Create `rms.projects` table with FK to opportunities, calendar_event_id, budget fields, timestamps. 5. Migration 004: Create `rms.inventory_items` table with SKU uniqueness constraint, quantity fields, location. 6. Migration 005: Create `rms.crew_members` table with skills as JSONB or text array, calendar_id, availability_status. 7. Migration 006: Create `rms.deliveries` table with FK to projects, status enum, scheduled/actual dates. 8. Migration 007: Create `rms.delivery_items` join table (delivery_id, inventory_item_id, quantity). 9. Migration 008: Create `rms.project_crew` join table (project_id, crew_member_id, role, assigned_at). 10. Add indexes on foreign keys, status columns, and tenant_id. 11. Integrate migration runner into the server startup or as a separate CLI command.

## Validation
Migrations run successfully against a clean PostgreSQL instance; all tables are created in the `rms` schema; rollback migrations work without errors; schema matches the protobuf message definitions.