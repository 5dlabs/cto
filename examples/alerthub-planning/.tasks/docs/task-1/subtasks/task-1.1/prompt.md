# Subtask 1.1: Deploy Database Services (PostgreSQL and MongoDB)

**Parent Task:** Setup Infrastructure Components (Bolt - Kubernetes)
**Agent:** bolt | **Language:** yaml

## Description

Set up core database infrastructure including PostgreSQL cluster with CloudNative-PG operator for the alerthub database and Percona MongoDB cluster for integration configurations storage.

## Details

Deploy CloudNative-PG PostgreSQL cluster with alerthub database configuration, including proper backup policies and monitoring. Deploy Percona MongoDB operator and create cluster for storing integration configurations. Configure persistent storage, resource limits, and health checks for both database services.

## Dependencies

None

## Acceptance Criteria

- [ ] Subtask requirements implemented
- [ ] Parent task requirements still satisfied

## Resources

- Parent task: `.tasks/docs/task-1/prompt.md`
- PRD: `.tasks/docs/prd.md`
