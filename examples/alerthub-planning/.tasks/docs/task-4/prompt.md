# Task 4: Admin API gRPC Service (Grizz - Go/gRPC)

**Agent**: grizz | **Language**: go

## Role

You are a Go Engineer specializing in APIs and backend services implementing Task 4.

## Goal

Implement the management API for tenants, users, rules, and analytics using Go gRPC with grpc-gateway for REST compatibility.

## Requirements

Generate Go code from protobuf definitions for TenantService, UserService, RuleService, and AnalyticsService. Implement JWT authentication, RBAC, PostgreSQL integration, Redis caching, and audit logging. Add grpc-gateway for REST API compatibility.

## Acceptance Criteria

All gRPC services respond correctly, REST endpoints work via grpc-gateway, JWT authentication validates tokens, RBAC enforces permissions, CRUD operations persist to PostgreSQL, and audit logs are created for sensitive operations

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-4): Admin API gRPC Service (Grizz - Go/gRPC)`

## Decision Points

### d7: JWT token expiration and refresh strategy
**Category**: security | **Constraint**: soft

Options:
1. 15 minute access tokens with refresh tokens
2. 1 hour access tokens with sliding refresh
3. configurable token lifetime per tenant

### d8: Analytics data aggregation frequency
**Category**: data-model | **Constraint**: open

Options:
1. real-time aggregation
2. hourly batch aggregation
3. daily aggregation with real-time approximations


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1
