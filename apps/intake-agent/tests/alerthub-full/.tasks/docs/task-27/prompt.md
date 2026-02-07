# Task 27: Implement UserService with RBAC

## Priority
high

## Description
Create user management service with role-based access control and preferences management

## Dependencies
- Task 26

## Implementation Details
Implement UserService gRPC methods with RBAC (owner, admin, member, viewer), user preferences, and password management.

## Acceptance Criteria
User CRUD operations work, RBAC enforces access control, user preferences persist correctly, password operations secure

## Decision Points
- **d27** [security]: Password hashing algorithm

## Subtasks
- 1. Design and implement gRPC service definition with RBAC proto [implementer]
- 2. Implement user authentication and password management logic [implementer]
- 3. Implement RBAC authorization system and user preferences [implementer]
- 4. Review UserService implementation for security and Go best practices [reviewer]
