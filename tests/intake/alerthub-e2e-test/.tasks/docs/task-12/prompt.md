# Task 12: Notification Submission API (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 12.

## Goal

Implement POST endpoints for single and batch notification submission

## Requirements

1. Create POST /api/v1/notifications endpoint\n2. Add request validation with serde\n3. Implement batch submission endpoint\n4. Add authentication middleware\n5. Return proper HTTP status codes and error responses

## Acceptance Criteria

Can submit valid notifications and receive 202 Accepted, invalid requests return 400 with error details, rate limits enforced

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-12): Notification Submission API (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 10, 11
