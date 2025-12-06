# Task 4: Implement webhook signature verification

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 4.

## Goal

Add security layer to verify incoming webhook authenticity using HMAC signatures

## Requirements

1. Create middleware/webhookAuth.ts
2. Implement HMAC-SHA256 signature verification
3. Compare computed signature with webhook header
4. Return 401 for invalid signatures
5. Add timing-safe comparison to prevent timing attacks

## Acceptance Criteria

Test with valid and invalid signatures, verify rejection of tampered payloads

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Implement webhook signature verification`
