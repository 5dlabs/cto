# Task 2: PostgreSQL Database Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 2.

## Goal

Deploy CloudNative-PG operator and PostgreSQL cluster for structured data storage

## Requirements

1. Install CloudNative-PG operator\n2. Create PostgreSQL cluster with 1 instance\n3. Configure database schemas for users, tenants, notifications\n4. Set up connection pooling\n5. Create service accounts and secrets

## Acceptance Criteria

PostgreSQL cluster is running, accepts connections, alerthub database exists, connection string works from test pod

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-2): PostgreSQL Database Setup (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
