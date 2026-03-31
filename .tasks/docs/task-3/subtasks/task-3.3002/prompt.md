Implement subtask 3002: Configure PostgreSQL database connectivity and migration framework

## Objective
Establish PostgreSQL connection pooling using ConfigMap endpoints, set up a migration framework (golang-migrate or goose), and create the initial RMS schema migrations for all domain tables.

## Steps
1. Read PostgreSQL connection details from environment variables sourced via `envFrom` referencing the infra-endpoints ConfigMap.
2. Use `pgxpool` for connection pooling with configurable pool size, timeouts, and health checks.
3. Set up `golang-migrate` with a `/migrations` directory.
4. Create initial migration files for the RMS schema:
   - `opportunities` table (id, customer_id, title, description, status enum [lead/quoted/won/lost], estimated_value, created_at, updated_at)
   - `projects` table (id, opportunity_id FK, name, status enum [pending/active/completed/cancelled], start_date, end_date, site_address, created_at, updated_at)
   - `inventory_items` table (id, barcode, name, category, status enum [available/rented/maintenance/retired], daily_rate, location, created_at, updated_at)
   - `bookings` table (id, project_id FK, inventory_item_id FK, start_date, end_date, status, created_at) with unique constraint to prevent overlapping bookings per item
   - `crew_members` table (id, name, email, phone, role, calendar_id, created_at)
   - `crew_assignments` table (id, project_id FK, crew_member_id FK, start_date, end_date, status)
   - `deliveries` table (id, project_id FK, type enum [delivery/pickup], scheduled_date, status enum [scheduled/in_transit/completed], driver_id, notes, created_at)
5. Add a migration CLI command or Makefile target: `make migrate-up`, `make migrate-down`.
6. Write a `db.go` package in `/internal/db/` that exposes the pool and a health check function.

## Validation
Migrations run successfully against a local PostgreSQL instance. All tables are created with correct columns, types, and constraints. Pool connects and responds to a ping. `make migrate-down` cleanly reverses all migrations.