# Task 2: User Authentication Service (Grizz - Go/gRPC)

## Overview
Implement core authentication and authorization service with JWT tokens, user registration, login, and session management

## Details
- gRPC authentication service with protobuf definitions
- JWT token generation and validation
- Email verification with token-based confirmation
- Password reset with secure token flow
- Admin user management with RBAC
- Session management with refresh token rotation

## Decision Points

### 1. JWT token expiration strategy

- **Category:** security
- **Constraint Type:** soft
- **Requires Approval:** No
- **Options:** Short-lived (15min) with refresh, Medium-lived (2hrs), Long-lived (24hrs)

### 2. User profile data structure

- **Category:** data-model
- **Constraint Type:** escalation
- **Requires Approval:** Yes
- **Options:** Fixed schema with migrations, JSON column for flexible fields, Separate profile service

## Testing Strategy
Auth service passes when:
- Users can register with email verification
- Login returns valid JWT tokens
- Protected endpoints reject invalid tokens
- Refresh tokens work correctly
- Password reset emails are sent

## Metadata
- **ID:** 2
- **Priority:** high
- **Status:** pending
- **Dependencies:** [1]
- **Subtasks:** 6 (see subtasks/ directory)
