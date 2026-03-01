# Subtask 18.18.1: Implement CreateTenant and GetTenant gRPC methods

## Parent Task
Task 18

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Build CreateTenant and GetTenant gRPC service methods with PostgreSQL persistence

## Dependencies
None

## Implementation Details
Implement TenantService CreateTenant (validates input, inserts to DB, returns Tenant) and GetTenant (queries by ID, returns Tenant or NOT_FOUND)

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
