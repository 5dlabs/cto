# Task 7: SeaweedFS Object Storage Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 7.

## Goal

Deploy SeaweedFS for S3-compatible object storage of attachments and exports

## Requirements

1. Deploy SeaweedFS master, volume, and filer components\n2. Configure S3 gateway for compatibility\n3. Create buckets for attachments, exports, media\n4. Set up access policies and credentials\n5. Configure backup retention

## Acceptance Criteria

SeaweedFS is running, S3 API responds, can create/delete objects in test bucket using AWS CLI

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-7): SeaweedFS Object Storage Setup (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
