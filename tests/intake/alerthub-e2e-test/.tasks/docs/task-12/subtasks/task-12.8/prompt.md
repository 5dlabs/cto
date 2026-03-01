# Subtask task-12.8: Configure grpc-gateway REST mappings for tenant endpoints

## Parent Task
Task 12

## Subagent Type
implementer

## Parallelizable
No - must wait for dependencies

## Description
Setup grpc-gateway annotations for HTTP/REST access to tenant gRPC methods

## Dependencies
- Subtask 18.18.3

## Implementation Details
Add grpc-gateway annotations to proto definitions, configure HTTP rules for POST /api/v1/tenants, GET /api/v1/tenants/:id, PUT /api/v1/tenants/:id, GET /api/v1/tenants

## Test Strategy
See parent task acceptance criteria.

---
*Project: alerthub*
