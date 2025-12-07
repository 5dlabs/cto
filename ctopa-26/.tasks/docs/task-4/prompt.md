# Task 4: Implement JWT authentication system

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 4.

## Goal

Create JWT-based authentication with access and refresh tokens, including token validation middleware

## Requirements

1. Add jsonwebtoken and bcrypt dependencies
2. Create JWT service with token generation/validation functions
3. Implement auth middleware to extract and validate JWT from Authorization header
4. Create refresh token rotation mechanism using Redis
5. Add password hashing utilities with bcrypt
6. Implement login/logout endpoints with proper token management

## Acceptance Criteria

Test token generation/validation, verify middleware blocks unauthorized requests, validate refresh token flow

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-4): Implement JWT authentication system`
