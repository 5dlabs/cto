# Task 3: Admin API Service (Grizz - Go/gRPC)

**Agent**: grizz | **Language**: go

## Role

You are a Senior Go Engineer with expertise in concurrent systems and microservices implementing Task 3.

## Goal

Build Go gRPC service with grpc-gateway for tenant, user, rule management and analytics

## Requirements

Implement TenantService, UserService, RuleService, AnalyticsService with JWT auth, RBAC (owner/admin/member/viewer), notification rules engine, and audit logging.

## Acceptance Criteria

Unit tests for rule engine, integration tests for gRPC, verify RBAC restrictions

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-3): Admin API Service (Grizz - Go/gRPC)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
