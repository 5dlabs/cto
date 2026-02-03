# Subtask 2.3: Implement Email Verification

## Parent Task
Task 2

## Agent
email-implementer

## Parallelizable
Yes

## Description
Build email verification system with token-based confirmation.

## Details
- Generate signed verification tokens
- Create email template with confirmation link
- Implement token storage and expiration
- Handle verification callback endpoint
- Rate limit verification requests

## Deliverables
- `email_service.go` - Email sending
- `verification_token.go` - Token generation
- `email_templates/` - HTML email templates
- `verification_handler.go` - Callback handler

## Acceptance Criteria
- [ ] Verification emails sent on registration
- [ ] Links expire after 24 hours
- [ ] Cannot verify same email twice
- [ ] Rate limits prevent abuse

## Testing Strategy
- Send test emails to mailhog/mailpit
- Verify token generation and validation
- Test expiration logic
