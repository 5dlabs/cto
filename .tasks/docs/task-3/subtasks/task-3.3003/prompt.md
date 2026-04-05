Implement subtask 3003: Implement PostgreSQL database layer and migrations for RMS schema

## Objective
Create the database migration files and a shared repository/data-access layer for all five RMS domain tables in the rms PostgreSQL schema.

## Steps
1. Choose a migration tool (golang-migrate/migrate or pressly/goose) and add it as a dependency.
2. Create migration files for: opportunities, projects, inventory_items, crew_members, deliveries, plus join tables (project_crew, project_equipment, delivery_equipment).
3. Each table should include: UUID primary key, created_at/updated_at timestamps, soft-delete (deleted_at nullable) per GDPR decision point.
4. Add appropriate indexes: opportunities(customer_id, status), inventory_items(barcode) UNIQUE, crew_members(name), deliveries(project_id, status), projects(status).
5. Implement a db package in /internal/db with: connection pool initialization using pgx/pgxpool, migration runner on startup, and base repository patterns.
6. Implement repository interfaces and concrete implementations for each domain: OpportunityRepo, ProjectRepo, InventoryRepo, CrewRepo, DeliveryRepo with standard CRUD methods.
7. Use transactions where needed (e.g., ConvertToProject creates a project and updates opportunity status atomically).
8. Reference POSTGRES_URL from config package (envFrom sigma1-infra-endpoints).

## Validation
Migrations run successfully against a test PostgreSQL instance creating all expected tables and indexes; repository CRUD operations work in isolation with test data; transactional operations roll back correctly on failure.