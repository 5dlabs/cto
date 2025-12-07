# Task 9: Implement team invitation system

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 9.

## Goal

Create team invitation links with expiration and acceptance flow

## Requirements

1. POST /api/teams/:id/invite - Generate invite token with 7-day expiration
2. Store invite tokens in Redis with team_id and role information
3. GET /api/invites/:token - Validate and display invite details
4. POST /api/invites/:token/accept - Accept invitation and add user to team
5. DELETE /api/teams/:id/invites/:token - Revoke invitation (admin+ only)

## Acceptance Criteria

Test invite generation, verify expiration handling, validate acceptance flow and duplicate prevention

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-9): Implement team invitation system`
