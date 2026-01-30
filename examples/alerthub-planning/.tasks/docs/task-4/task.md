# Task 4: Admin API gRPC Service (Grizz - Go/gRPC)

## Status
pending

## Priority
high

## Dependencies
task-1

## Description
Implement the management API for tenants, users, rules, and analytics using Go gRPC with grpc-gateway for REST compatibility.

## Details
Generate Go code from protobuf definitions for TenantService, UserService, RuleService, and AnalyticsService. Implement JWT authentication, RBAC, PostgreSQL integration, Redis caching, and audit logging. Add grpc-gateway for REST API compatibility.

## Test Strategy
All gRPC services respond correctly, REST endpoints work via grpc-gateway, JWT authentication validates tokens, RBAC enforces permissions, CRUD operations persist to PostgreSQL, and audit logs are created for sensitive operations

## Decision Points

### d7: JWT token expiration and refresh strategy
- **Category**: security
- **Constraint**: soft
- **Requires Approval**: No
- **Options**:
  - 15 minute access tokens with refresh tokens
  - 1 hour access tokens with sliding refresh
  - configurable token lifetime per tenant

### d8: Analytics data aggregation frequency
- **Category**: data-model
- **Constraint**: open
- **Requires Approval**: No
- **Options**:
  - real-time aggregation
  - hourly batch aggregation
  - daily aggregation with real-time approximations

