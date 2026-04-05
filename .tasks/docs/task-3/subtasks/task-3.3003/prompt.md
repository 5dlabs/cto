Implement subtask 3003: Implement PostgreSQL schema migrations and repository layer

## Objective
Create SQL migration files for all RMS domain tables and implement a Go repository layer for database access using the PostgreSQL connection from ConfigMap.

## Steps
1. Choose a migration tool (golang-migrate or goose). 2. Create migration files for tables: opportunities, projects, quotes, inventory_items, inventory_transactions, crew_members, crew_assignments, deliveries, delivery_schedules. Include proper indexes, foreign keys, and enums for statuses. 3. Implement a db package that initializes a connection pool (pgxpool) using the connection string from the ConfigMap. 4. Implement repository interfaces and concrete implementations for each domain: OpportunityRepo, ProjectRepo, InventoryRepo, CrewRepo, DeliveryRepo. 5. Each repo should support the CRUD operations needed by the gRPC services. 6. Add a migration command or auto-migrate on startup.

## Validation
Migrations run successfully against a test PostgreSQL instance; all tables are created with correct columns, types, and constraints; repository methods perform CRUD operations correctly verified by integration tests.