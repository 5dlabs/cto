# Task 4: Develop Admin API Service (Grizz - Go/gRPC)

**Agent**: grizz | **Language**: go

## Role

You are a Go Engineer specializing in APIs and backend services implementing Task 4.

## Goal

Build the gRPC-based admin API in Go for managing tenants, users, notification rules, and analytics. Includes grpc-gateway for REST endpoints and JWT authentication.

## Requirements

1. Set up Go project with gRPC and grpc-gateway
2. Define protobuf schemas for all services (Tenant, User, Rule, Analytics)
3. Implement gRPC server with service handlers
4. Add PostgreSQL integration for data persistence
5. Build JWT authentication with refresh tokens
6. Implement role-based access control (RBAC)
7. Create notification rules engine
8. Add analytics aggregation logic
9. Set up grpc-gateway for REST endpoints
10. Add audit logging and health checks

## Acceptance Criteria

gRPC server starts successfully, all service methods work correctly, JWT authentication validates tokens, RBAC prevents unauthorized access, rules engine filters notifications correctly, analytics return accurate data, and REST endpoints via grpc-gateway function properly.

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Develop Admin API Service (Grizz - Go/gRPC)`

## Decision Points

### d7: What JWT token expiration times should be used for access and refresh tokens?
**Category**: security | **Constraint**: soft | ⚠️ **Requires Approval**

Options:
1. 15min-access-7day-refresh
2. 1hour-access-30day-refresh
3. configurable-per-tenant

### d8: How should notification rules be structured for maximum flexibility?
**Category**: data-model | **Constraint**: open

Options:
1. json-based-rules
2. sql-like-dsl
3. visual-builder-schema


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1
