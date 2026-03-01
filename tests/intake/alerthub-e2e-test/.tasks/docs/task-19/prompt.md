# Task 19: Slack Integration Service (Nova - Bun/Elysia+Effect)

**Agent**: nova | **Language**: typescript

## Role

You are a Senior Node.js Engineer with expertise in server-side JavaScript and APIs implementing Task 19.

## Goal

Implement Slack delivery service with Effect retry logic and error handling

## Requirements

1. Create SlackService using Effect.Service pattern\n2. Implement webhook and Bot API delivery methods\n3. Add Effect.retry with exponential backoff\n4. Handle Slack-specific errors as tagged unions\n5. Add rate limiting with Effect.Semaphore

## Acceptance Criteria

Can deliver notifications to Slack webhooks, retries work on failures, rate limiting prevents API abuse

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-19): Slack Integration Service (Nova - Bun/Elysia+Effect)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 18
