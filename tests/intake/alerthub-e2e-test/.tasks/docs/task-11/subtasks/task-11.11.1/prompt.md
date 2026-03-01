# Subtask 11.11.1: Implement /health/live liveness endpoint

## Parent Task
Task 11

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Create liveness probe endpoint that checks service is running

## Dependencies
None

## Implementation Details
Implement GET /health/live returning 200 OK when service process is healthy, used by Kubernetes liveness probe

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
