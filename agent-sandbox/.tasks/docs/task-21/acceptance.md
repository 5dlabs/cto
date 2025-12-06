# Acceptance Criteria: Task 21

- [ ] Create team management endpoints with role-based access control enforcing owner/admin/member/viewer permissions.
- [ ] Integration tests: create team as user A, verify user A is owner. User B cannot access team. Add user B as member, verify they can GET but not PATCH. Test admin can PATCH. Verify member counts are accurate.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 21.1: Create domain models for Team, TeamMember, and TeamRole with permission logic
- [ ] 21.2: Implement team repository with sqlx queries for CRUD operations
- [ ] 21.3: Implement team creation endpoint with automatic owner assignment
- [ ] 21.4: Implement team retrieval endpoint with member count aggregation
- [ ] 21.5: Implement team update endpoint with admin authorization
- [ ] 21.6: Create RequireTeamRole extractor middleware for role-based access control
