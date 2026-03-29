Implement subtask 4002: Integrate PostgreSQL for data persistence

## Objective
Connect the service to PostgreSQL using `sqlx` for data persistence, utilizing the `sigma1-infra-endpoints` ConfigMap.

## Steps
1. Configure `sqlx` to connect to PostgreSQL using the URL from `sigma1-infra-endpoints`.2. Implement connection pooling for PostgreSQL.

## Validation
Deploy the service and verify successful connection to PostgreSQL. Perform a basic database operation (e.g., insert/select) to confirm persistence.