# Task 1: Infrastructure Foundation Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 1.

## Goal

Set up Kubernetes cluster with basic operators and namespaces for AlertHub platform

## Requirements

1. Create Kubernetes cluster\n2. Install cert-manager for TLS certificates\n3. Create namespaces: databases, kafka, messaging, alerthub\n4. Set up RBAC policies\n5. Configure network policies for service isolation

## Acceptance Criteria

Cluster is accessible via kubectl, all namespaces exist, cert-manager webhook responds, network policies allow expected traffic

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-1): Infrastructure Foundation Setup (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
