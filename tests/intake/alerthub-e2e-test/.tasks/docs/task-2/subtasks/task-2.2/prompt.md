# Subtask task-2.2: Implement /health/ready readiness endpoint

## Parent Task
Task 2

## Subagent Type
implementer

## Parallelizable
Yes - can run concurrently

## Description
Create readiness probe endpoint checking database, Redis, and Kafka connectivity

## Dependencies
None

## Implementation Details
Implement GET /health/ready that checks PostgreSQL connection, Redis ping, and Kafka broker connectivity, returns 503 if any dependency is unavailable

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
