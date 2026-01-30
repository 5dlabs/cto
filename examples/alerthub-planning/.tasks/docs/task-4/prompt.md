# Implementation Prompt for Task 4

## Context
You are implementing "Admin API gRPC Service (Grizz - Go/gRPC)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Implement the management API for tenants, users, rules, and analytics using Go gRPC with grpc-gateway for REST compatibility.

## Implementation Details
Generate Go code from protobuf definitions for TenantService, UserService, RuleService, and AnalyticsService. Implement JWT authentication, RBAC, PostgreSQL integration, Redis caching, and audit logging. Add grpc-gateway for REST API compatibility.

## Dependencies
This task depends on: task-1. Ensure those are complete before starting.

## Testing Requirements
All gRPC services respond correctly, REST endpoints work via grpc-gateway, JWT authentication validates tokens, RBAC enforces permissions, CRUD operations persist to PostgreSQL, and audit logs are created for sensitive operations

## Decision Points to Address

The following decisions need to be made during implementation:

### d7: JWT token expiration and refresh strategy
**Category**: security | **Constraint**: soft

Options:
1. 15 minute access tokens with refresh tokens
2. 1 hour access tokens with sliding refresh
3. configurable token lifetime per tenant

Document your choice and rationale in the implementation.

### d8: Analytics data aggregation frequency
**Category**: data-model | **Constraint**: open

Options:
1. real-time aggregation
2. hourly batch aggregation
3. daily aggregation with real-time approximations

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging
