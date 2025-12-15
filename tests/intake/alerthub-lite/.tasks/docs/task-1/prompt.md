# Task 1: Setup Infrastructure with PostgreSQL and Redis/Valkey

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 1.

## Goal

Provision PostgreSQL database using CloudNative-PG operator and Redis/Valkey cache to support the notification service

## Requirements

1. Create Kubernetes namespace 'alerthub'
2. Deploy CloudNative-PG cluster with 1 instance, 1Gi storage
3. Deploy Redis/Valkey using redis-operator with valkey/valkey:7.2-alpine image
4. Create database 'alerthub' with user 'alerthub_user'
5. Generate ConfigMap 'alerthub-infra-config' with DATABASE_URL and REDIS_URL
6. Verify connectivity and create initial schema

## Acceptance Criteria

Test database and Redis connectivity, verify ConfigMap contains valid connection strings, run basic CRUD operations

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-1): Setup Infrastructure with PostgreSQL and Redis/Valkey`
