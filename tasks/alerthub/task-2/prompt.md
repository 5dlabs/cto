# Implementation Prompt: Task 2

**Agent:** Grizz
**Tech Stack:** Go, gRPC, PostgreSQL, JWT
**Status:** pending

## What to Build
User authentication and authorization service with:
- gRPC API for auth operations
- JWT token management with refresh rotation
- Email verification flow
- Password reset functionality
- Admin user management

## Implementation Details
Execute subtasks:
1. Design user schema and migrations
2. Implement gRPC auth service
3. Implement email verification
4. Implement password reset
5. Implement admin management
6. Security review

## Dependencies
Task 1 (Infrastructure) must be complete

## Testing Requirements
Auth service ready when:
- Users can register with email verification
- Login returns valid JWT tokens
- Protected endpoints reject invalid tokens
- Password reset emails are sent
