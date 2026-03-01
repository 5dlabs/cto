# Task 3: Redis Cache Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 3.

## Goal

Deploy Redis operator and Valkey instance for caching and rate limiting

## Requirements

1. Install Redis operator\n2. Create Valkey instance with persistence\n3. Configure memory limits and eviction policies\n4. Set up monitoring and alerts\n5. Create service for internal access

## Acceptance Criteria

Valkey instance is running, accepts redis-cli connections, can set/get keys, persistence is working

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-3): Redis Cache Setup (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
