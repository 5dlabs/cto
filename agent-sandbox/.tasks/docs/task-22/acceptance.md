# Acceptance Criteria: Task 22

- [ ] Create endpoint to generate time-limited invite links for teams with 7-day expiration and single-use token validation.
- [ ] Integration tests: admin creates invite, verify token stored with correct expiration. Non-member accepts invite, verify added as member. Test expired token returns 404. Test already-member returns 400. Verify token deleted after acceptance. Test viewer cannot create invite.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 22.1: Create invite link domain model and repository with secure token generation
- [ ] 22.2: Implement POST /api/teams/:id/invite endpoint with admin authorization
- [ ] 22.3: Implement POST /api/invite/:token/accept endpoint with validation
- [ ] 22.4: Add atomic single-use token enforcement with race condition handling
- [ ] 22.5: Create scheduled cleanup job for expired invite tokens
