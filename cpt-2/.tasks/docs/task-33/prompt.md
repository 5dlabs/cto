# Task 33: Implement account lockout protection

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 33.

## Goal

Add account lockout after multiple failed login attempts

## Requirements

1. Track failed login attempts per user
2. Lock account after 5 failed attempts
3. Implement exponential backoff for lockout duration
4. Send email notification on account lockout
5. Provide account unlock mechanism

## Acceptance Criteria

Test account lockout triggers and unlock procedures work correctly

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-33): Implement account lockout protection`
