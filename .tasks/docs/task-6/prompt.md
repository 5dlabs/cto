Implement task 6: Develop Social Media Engine (Nova - Node.js/Elysia)

## Goal
Implement the Social Media Engine for AI curation, caption generation, approval workflow, and multi-platform publishing, using Elysia 1.x and Effect for schema validation and retries.

## Task Context
- Agent owner: Nova
- Stack: Node.js/Elysia
- Priority: medium
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Node.js 20+ project with Elysia 1.x and Effect 3.x.", "Define endpoints: /api/v1/social/upload, /api/v1/social/drafts, /api/v1/social/drafts/:id, /api/v1/social/drafts/:id/approve, /api/v1/social/drafts/:id/reject, /api/v1/social/drafts/:id/publish, /api/v1/social/published.", "Integrate with PostgreSQL for drafts and published posts.", "Implement S3/R2 photo storage and retrieval.", "Integrate with Instagram, LinkedIn, Facebook APIs for publishing.", "Use OpenAI/Claude for caption generation.", "Implement AI curation pipeline to select top images.", "Implement approval workflow via Signal (Morgan integration).", "Add Effect.Schema for request/response validation and Effect.retry for API calls.", "Document API usage and content pipeline."]}

## Acceptance Criteria
Uploading event photos creates drafts; AI curation selects top images; caption generation produces relevant captions; approval workflow triggers Signal message; publishing posts to at least one platform succeeds; all endpoints validate requests/responses with Effect.Schema.

## Subtasks
- Initialize Elysia project with Effect, PostgreSQL integration, and data models: Set up the Node.js 20+ project with Elysia 1.x and Effect 3.x, define the database schema for social media drafts and published posts, and create the base Elysia router with Effect.Schema validation.
- Implement S3/R2 photo upload and storage service: Build the photo upload and storage module using S3-compatible API (Cloudflare R2 or AWS S3 per dp-4), handling multipart uploads, generating signed URLs for retrieval, and wiring into the /api/v1/social/upload endpoint.
- Implement AI curation pipeline for selecting top images: Build the AI image curation module that analyzes uploaded event photos using a vision-capable AI model and selects the top images based on quality, composition, and relevance.
- Implement AI caption generation service: Build the AI caption generation module that creates platform-appropriate captions for curated social media posts using OpenAI or Claude.
- Implement approval workflow with Signal/Morgan integration: Build the approval workflow that sends draft posts for human review via Signal (through Morgan integration), handles approve/reject responses, and manages the draft lifecycle endpoints.
- Implement Instagram API publishing integration: Build the Instagram publishing module to post approved content (photos + captions) to Instagram using the Instagram Graph API.
- Implement LinkedIn API publishing integration: Build the LinkedIn publishing module to post approved content to a LinkedIn company page using the LinkedIn Marketing API.
- Implement Facebook API publishing integration: Build the Facebook publishing module to post approved content to a Facebook page using the Facebook Graph API.
- Build publishing orchestrator and published posts endpoint: Implement the publishing orchestrator that dispatches approved drafts to all target platforms concurrently, handles partial failures, and exposes the publish and published-posts endpoints.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.