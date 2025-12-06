# Acceptance Criteria: Task 8

- [ ] Implement granular permission system for owner, admin, member, and viewer roles with route-level authorization guards
- [ ] Unit test role hierarchy comparison. Integration tests: create team as owner, add member with viewer role, verify viewer cannot create tasks but can read. Test admin can manage members but not delete team
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 8.1: Define role hierarchy and permission matrix with documentation
- [ ] 8.2: Implement authorization middleware with role checking logic
- [ ] 8.3: Create helper functions for team access validation
- [ ] 8.4: Apply authorization to routes with comprehensive permission testing
