Implement subtask 3003: Define data models and integrate with PostgreSQL

## Objective
Define Go structs for `Opportunity`, `Project`, and `InventoryTransaction` and integrate `sqlx` for PostgreSQL interaction, using the `sigma1-infra-endpoints` ConfigMap.

## Steps
1. Define Go structs for `Opportunity`, `Project`, and `InventoryTransaction` with `sqlx` tags.2. Set up `sqlx` connection pool using the PostgreSQL URL from the `sigma1-infra-endpoints` ConfigMap.3. Implement basic CRUD operations for these models.

## Validation
1. Write unit tests for `sqlx` connection and basic CRUD operations.2. Deploy the service and verify successful connection to PostgreSQL.3. Insert and retrieve sample data to confirm persistence.