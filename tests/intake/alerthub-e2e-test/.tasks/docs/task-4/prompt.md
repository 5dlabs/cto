# Task 4: Implement Admin API Service (Grizz - Go/gRPC)

**Agent**: grizz | **Language**: go

## Role

You are a Senior Go Engineer with expertise in concurrent systems and microservices implementing Task 4.

## Goal

Build the management API for tenants, users, rules, and analytics using gRPC with REST gateway

## Requirements

1. Initialize Go project with gRPC:
   go mod init github.com/alerthub/admin-api
   go get google.golang.org/grpc google.golang.org/protobuf github.com/grpc-ecosystem/grpc-gateway/v2 github.com/jackc/pgx/v5 github.com/redis/go-redis/v9

2. Define protobuf schemas in api/proto/admin.proto:
   service TenantService { rpc CreateTenant, GetTenant, UpdateTenant, ListTenants, DeleteTenant }
   service UserService { rpc CreateUser, GetUser, UpdateUser, ListUsers, DeleteUser, UpdatePreferences }
   service RuleService { rpc CreateRule, GetRule, UpdateRule, ListRules, DeleteRule, EvaluateRule }
   service AnalyticsService { rpc GetNotificationStats, GetDeliveryMetrics, GetChannelBreakdown }
   message Tenant { string id, string name, string plan, TenantSettings settings, google.protobuf.Timestamp created_at }
   message User { string id, string tenant_id, string email, string role, UserPreferences preferences }
   message NotificationRule { string id, string tenant_id, string name, repeated RuleCondition conditions, repeated RuleAction actions, bool enabled, int32 priority }

3. Generate Go code and gRPC gateway:
   protoc --go_out=. --go-grpc_out=. --grpc-gateway_out=. api/proto/admin.proto

4. Implement database layer with pgx:
   - Create migrations for tenants, users, rules, audit_logs tables
   - Implement repository pattern for each entity
   - Add connection pool with pgxpool

5. Implement gRPC service handlers:
   type TenantServiceServer struct { db pgxpool.Pool, redis redis.Client }
   func (s TenantServiceServer) CreateTenant(ctx context.Context, req CreateTenantRequest) (Tenant, error) {
     // Validate request, insert to DB, return tenant
   }
   // Implement all other RPC methods

6. Implement authentication middleware:
   - JWT validation using RS256
   - Extract tenant_id and user_id from token
   - Add to context for downstream handlers

7. Implement RBAC authorization:
   - Define role permissions (owner: all, admin: CRUD, member: read/create, viewer: read)
   - Check permissions in each RPC handler
   - Return PermissionDenied error if unauthorized

8. Implement notification rules engine:
   - Parse rule conditions (field, operator, value)
   - Evaluate against notification metadata
   - Support operators: eq, ne, gt, lt, contains, regex
   - Return matching actions

9. Implement analytics aggregation:
   - Query notifications table with time range filters
   - Aggregate by status, channel, priority
   - Cache results in Redis with TTL
   - Return NotificationStats and DeliveryMetrics

10. Setup grpc-gateway for REST:
    - Register HTTP handlers from generated code
    - Map gRPC endpoints to REST paths
    - Add CORS middleware

11. Add audit logging:
    - Log all write operations (create, update, delete)
    - Include user_id, tenant_id, action, timestamp, changes
    - Store in audit_logs table

12. Create Dockerfile:
   FROM golang:1.22 AS builder
   WORKDIR /app
   COPY go.mod go.sum ./
   RUN go mod download
   COPY . .
   RUN CGO_ENABLED=0 go build -o admin-api cmd/server/main.go
   FROM alpine:latest
   COPY --from=builder /app/admin-api /usr/local/bin/
   CMD ["admin-api"]

## Acceptance Criteria

1. Unit tests for each RPC handler
2. Test database operations with testcontainers
3. Test JWT authentication with valid/invalid tokens
4. Test RBAC with different roles
5. Test rule evaluation with various conditions
6. Test analytics aggregation with sample data
7. Integration tests for gRPC and REST endpoints
8. Test audit logging for all write operations
9. Load test with grpcurl
10. Verify Redis caching behavior

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-4): Implement Admin API Service (Grizz - Go/gRPC)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
