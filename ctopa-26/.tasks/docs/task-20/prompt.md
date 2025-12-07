# Task 20: Implement GDPR compliance features

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 20.

## Goal

Add data export and deletion capabilities for GDPR compliance

## Requirements

1. Create GDPR endpoints: GET /api/users/export, DELETE /api/users/delete
2. Implement data export functionality returning user data in JSON format
3. Add hard delete functionality that removes all user data and anonymizes references
4. Create audit log for data deletion requests
5. Add data retention policy enforcement (30-day soft delete cleanup)
6. Implement consent management for notification preferences
7. Add privacy policy acceptance tracking

## Acceptance Criteria

Integration tests for data export completeness and deletion verification with anonymization checks

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-20): Implement GDPR compliance features`
