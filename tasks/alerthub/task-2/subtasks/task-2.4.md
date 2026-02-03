# Subtask 2.4: Implement Password Reset

## Parent Task
Task 2

## Agent
auth-implementer

## Parallelizable
Yes

## Description
Build password reset flow with secure token-based confirmation.

## Details
- Create password reset request endpoint
- Generate secure reset tokens with expiration
- Send reset email with link
- Validate tokens and update password
- Invalidate all existing sessions

## Deliverables
- `password_reset.go` - Reset flow logic
- `password_reset_handler.go` - Endpoints
- `reset_email.tpl` - Email template

## Acceptance Criteria
- [ ] Reset links expire after 1 hour
- [ ] New password must meet complexity requirements
- [ ] All sessions invalidated after reset
- [ ] Rate limiting prevents abuse

## Testing Strategy
- Test token generation
- Verify password complexity enforcement
- Test session invalidation
