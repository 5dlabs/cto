Implement task 6: Develop Social Media Engine (Nova - Node.js/Elysia)

## Goal
Implement the Social Media Engine for AI curation, caption generation, approval workflow, and multi-platform publishing.

## Task Context
- Agent owner: Nova
- Stack: Node.js 20+/Elysia 1.x + Effect
- Priority: medium
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Node.js project with Elysia 1.x and Effect 3.x, connect to PostgreSQL and S3/R2 using ConfigMap.", "Define endpoints: /api/v1/social/upload, /api/v1/social/drafts, /api/v1/social/drafts/:id, /api/v1/social/drafts/:id/approve, /api/v1/social/drafts/:id/reject, /api/v1/social/drafts/:id/publish, /api/v1/social/published.", "Implement AI curation pipeline using OpenAI/Claude for image scoring and caption generation.", "Integrate Instagram, LinkedIn, Facebook APIs for publishing.", "Implement approval workflow: send drafts to Morgan via Signal, handle approval/rejection.", "Sync published content to website portfolio.", "Add Effect.Schema for request/response validation.", "Write integration tests for all endpoints."]}

## Acceptance Criteria
All endpoints function as described; AI curation selects top images and generates captions; approval workflow sends and receives Signal messages; posts publish to at least two platforms in test; >80% code coverage.

## Subtasks
- Initialize Elysia/Effect project with PostgreSQL and S3/R2 connectivity: Scaffold the Node.js service with Elysia 1.x and Effect 3.x, configure PostgreSQL connection pool and S3-compatible object storage client using infra ConfigMap values.
- Implement image upload endpoint with S3/R2 storage: Build the POST /api/v1/social/upload endpoint that accepts image files, stores them in S3/R2, and returns the stored URLs.
- Implement AI curation pipeline for image scoring and caption generation: Build the AI curation pipeline that uses OpenAI/Claude to score uploaded images for social media suitability and generate platform-appropriate captions.
- Implement draft management REST endpoints with Effect.Schema validation: Build the CRUD endpoints for draft management: list drafts, get draft by ID, approve, reject — with full Effect.Schema request/response validation.
- Implement Signal-based approval workflow for Morgan: Build the approval notification system that sends curated drafts to Morgan via Signal and processes approval/rejection responses.
- Implement Instagram API publishing integration: Build the Instagram publishing module using the Instagram Graph API to publish approved images with captions.
- Implement LinkedIn API publishing integration: Build the LinkedIn publishing module using the LinkedIn Marketing API to publish approved images with professional captions to the company page.
- Implement Facebook API publishing integration: Build the Facebook publishing module using the Facebook Graph API to publish approved images with captions to the business page.
- Implement publish endpoint and portfolio sync: Build the POST /api/v1/social/drafts/:id/publish endpoint that orchestrates multi-platform publishing and the GET /api/v1/social/published endpoint, plus website portfolio synchronization.
- Write integration tests for all Social Media Engine endpoints: Create comprehensive integration tests covering the full lifecycle: upload → curate → approve → publish, including error scenarios and edge cases.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.