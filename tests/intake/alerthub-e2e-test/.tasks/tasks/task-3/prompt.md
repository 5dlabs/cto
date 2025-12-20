# Task 3: Admin API Service (Go/gRPC)

## Agent: Grizz
## Priority: High
## Language: Go 1.22+
## Framework: gRPC with grpc-gateway

## Objective
Build a management API for tenants, users, rules, and analytics.

## gRPC Services
- TenantService: CreateTenant, GetTenant, UpdateTenant, ListTenants
- UserService: CreateUser, GetUser, UpdateUser, ListUsers, UpdatePreferences
- RuleService: CreateRule, GetRule, UpdateRule, ListRules, DeleteRule
- AnalyticsService: GetNotificationStats, GetDeliveryMetrics

## Dependencies
- PostgreSQL: Tenants, users, rules, audit logs
- Redis: Session cache, analytics aggregation

## Project Structure
```
services/admin-api/
├── go.mod
├── go.sum
├── cmd/server/main.go
├── internal/
│   ├── service/
│   ├── repository/
│   └── config/
├── api/
│   └── proto/
└── tests/
```

## Acceptance Criteria
- [ ] gRPC services work correctly
- [ ] REST gateway accessible via HTTP
- [ ] Tests pass with `go test ./...`
- [ ] golangci-lint passes

