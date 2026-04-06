Implement task 6: Develop Social Media Engine (Nova - Node.js/Elysia)

## Goal
Implement the Social Media Engine for AI curation, caption generation, approval workflow, and multi-platform publishing. Integrate with PostgreSQL, S3/R2, and social APIs.

## Task Context
- Agent owner: Nova
- Stack: Node.js 20+/Elysia 1.x + Effect
- Priority: medium
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Node.js project with Elysia 1.x and Effect 3.x.", "Define endpoints: /api/v1/social/upload, /api/v1/social/drafts, /api/v1/social/drafts/:id, /api/v1/social/drafts/:id/approve, /api/v1/social/drafts/:id/reject, /api/v1/social/drafts/:id/publish, /api/v1/social/published.", "Implement AI curation pipeline using OpenAI/Claude for captioning.", "Integrate with Instagram, LinkedIn, Facebook APIs for publishing.", "Store drafts and published posts in PostgreSQL.", "Use S3/R2 for photo storage.", "Implement approval workflow via Signal (Morgan integration).", "Connect to infra via envFrom: sigma1-infra-endpoints ConfigMap.", "Validate all request/response schemas with Effect.Schema."]}

## Acceptance Criteria
All endpoints function as specified; AI curation selects top images; captions are generated; approval workflow triggers Signal notifications; posts are published to at least two platforms; portfolio sync updates website.

## Subtasks
- Scaffold Elysia/Effect project with PostgreSQL models and migrations: Initialize the Social Media Engine Node.js project with Elysia 1.x, Effect 3.x, and PostgreSQL integration. Define draft and published post data models and create database migrations.
- Implement S3/R2 photo upload and storage integration: Build the photo upload endpoint and S3/R2 storage integration for the /api/v1/social/upload endpoint, handling multipart file uploads and returning stored image URLs.
- Implement AI curation pipeline for image selection and caption generation: Build the AI curation module that uses OpenAI or Claude to analyze uploaded images, select the best ones, and generate platform-appropriate captions.
- Implement draft management endpoints and approval workflow: Build the draft CRUD endpoints and the approval/rejection workflow including Signal notification integration with Morgan.
- Implement Instagram API publishing integration: Build the Instagram publishing client that posts approved content to Instagram via the Instagram Graph API.
- Implement LinkedIn API publishing integration: Build the LinkedIn publishing client that posts approved content to a LinkedIn company page via the LinkedIn API.
- Implement Facebook API publishing integration: Build the Facebook publishing client that posts approved content to a Facebook page via the Graph API.
- Wire up publish endpoint, published posts listing, metrics, and schema validation: Implement the publish and published-posts endpoints that orchestrate multi-platform publishing, record results, and add Prometheus metrics and health probes.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.