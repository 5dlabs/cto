# Task 11: Implement user registration with email

## Role

You are a Senior Security Engineer with expertise in authentication and secure coding implementing Task 11.

## Goal

Create traditional email/password registration functionality

## Requirements

1. Create POST /auth/register endpoint
2. Validate email format and password strength
3. Hash password using bcrypt with salt rounds=12
4. Check for existing users with same email
5. Create user record and send verification email
6. Apply rate limiting to registration endpoint

## Acceptance Criteria

Test user registration, password hashing, duplicate email handling, and email sending

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-11): Implement user registration with email`
