# Acceptance Criteria: Task 18

- [ ] Build JWT-based authentication system with access tokens (15min expiry) and refresh tokens (7 days), stored in Redis. Include token validation middleware.
- [ ] Unit tests for token generation/validation with valid/expired/invalid tokens. Integration tests: register user, login, use access token, refresh token after expiry, verify old refresh token invalidated. Test middleware rejects requests without valid token.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 18.1: Implement JWT token generation and validation utilities
- [ ] 18.2: Implement refresh token storage and retrieval in Redis
- [ ] 18.3: Create user registration endpoint with password hashing
- [ ] 18.4: Create login endpoint with credential validation
- [ ] 18.5: Create refresh token endpoint with rotation logic
- [ ] 18.6: Implement Axum authentication middleware with Claims extraction
