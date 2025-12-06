# Acceptance Criteria: Task 7

- [ ] Add OAuth2 authentication flows for Google and GitHub providers with callback handling and user account linking
- [ ] Manual test: initiate OAuth flow, complete authorization, verify JWT returned. Unit test state validation. Test account linking when email exists. Mock provider APIs for integration tests
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 7.1: Set up OAuth2 client configuration and environment variables
- [ ] 7.2: Implement authorization redirect flow with CSRF state protection
- [ ] 7.3: Build callback handler with token exchange
- [ ] 7.4: Implement user info fetching and account creation/linking logic
- [ ] 7.5: Add database schema changes for OAuth provider tracking
