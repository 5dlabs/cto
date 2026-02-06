# Subtask 2.2: Implement gRPC Auth Service

## Parent Task
Task 2

## Agent
grpc-implementer

## Parallelizable
Yes

## Description
Implement core gRPC authentication service with protobuf definitions.

## Details
- Define auth.proto with Register, Login, Refresh, Logout RPCs
- Implement Register handler with bcrypt password hashing
- Implement Login handler with JWT generation
- Implement Refresh token rotation logic
- Add middleware for authentication interceptors
- Create protobuf Go code generation

## Deliverables
- `proto/auth.proto` - Service definitions
- `auth_server.go` - gRPC server implementation
- `auth_middleware.go` - JWT verification middleware
- `auth_client.go` - Client library for other services

## Acceptance Criteria
- [ ] gRPC server starts and listens
- [ ] Register creates user with hashed password
- [ ] Login returns JWT access + refresh tokens
- [ ] Middleware validates JWT on protected endpoints

## Testing Strategy
- Unit tests for each RPC handler
- Integration tests with test database
- Test JWT token validation
