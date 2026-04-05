Implement task 6: Develop Social Media Engine (Nova - Node.js/Elysia)

## Goal
Implement the Social Media Engine for AI-powered photo curation, caption generation, approval workflow, and multi-platform publishing. Enables automated event content publishing and portfolio sync.

## Task Context
- Agent owner: nova
- Stack: Node.js/Elysia
- Priority: medium
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Node.js 20+ project with Elysia 1.x and Effect TypeScript.", "Define Effect.Services for Instagram, LinkedIn, TikTok, Facebook.", "Implement endpoints: /api/v1/social/upload, /api/v1/social/drafts, /api/v1/social/drafts/:id, /api/v1/social/drafts/:id/approve, /api/v1/social/drafts/:id/reject, /api/v1/social/drafts/:id/publish, /api/v1/social/published.", "Integrate OpenAI/Claude for caption generation.", "Implement AI curation pipeline for image selection and cropping.", "Integrate with S3/R2 for photo storage and serve via CDN.", "Implement approval workflow via Signal integration.", "Reference connection strings and API keys from 'sigma1-infra-endpoints' ConfigMap and secrets.", "Write unit and integration tests for all endpoints and pipelines."]}

## Acceptance Criteria
All endpoints function as specified; AI curation and captioning produce expected results; approval workflow triggers Signal notifications; published content appears on all platforms; tests cover at least 80% of code paths.

## Subtasks
- Scaffold Elysia/Effect TypeScript project with infrastructure wiring: Initialize the Node.js 20+ project with Elysia 1.x, Effect TypeScript, and all necessary dependencies. Configure the project structure, environment loading from 'sigma1-infra-endpoints' ConfigMap, database connection via Effect layers, and a health check endpoint.
- Define data models and database migrations for social media drafts and published content: Design and implement the data models for social media content lifecycle: uploads, drafts (with AI-generated captions), approval status, and published content records. Create PostgreSQL migrations for all tables.
- Implement S3/R2 photo storage integration service: Build an Effect.Service for uploading, retrieving, and managing photos in S3/R2 object storage. Handle image upload processing, generate storage keys, and provide CDN-ready URLs.
- Implement AI curation pipeline for image selection and quality assessment: Build an Effect.Service that uses OpenAI's vision API (or Claude) to analyze uploaded images, assess quality, select the best images for social media posting, and suggest cropping/composition improvements.
- Implement AI caption generation service: Build an Effect.Service that uses OpenAI/Claude to generate social media captions, hashtags, and platform-specific variations for curated images. Support different tones and styles per platform.
- Implement Effect.Services for Instagram publishing: Build an Effect.Service for publishing content to Instagram via the Instagram Graph API, including image upload, caption posting, and status tracking.
- Implement Effect.Services for LinkedIn publishing: Build an Effect.Service for publishing content to LinkedIn via the LinkedIn Marketing API, including image upload, article/post creation, and status tracking.
- Implement Effect.Services for TikTok publishing: Build an Effect.Service for publishing content to TikTok via the TikTok Content Posting API, including video/image upload and status tracking.
- Implement Effect.Services for Facebook publishing: Build an Effect.Service for publishing content to Facebook via the Facebook Graph API, including photo upload, post creation on a page, and status tracking.
- Implement approval workflow with Signal integration: Build the approval workflow system that sends draft content for human review via Signal messaging, handles approve/reject responses, and updates draft status accordingly.
- Implement API endpoints for upload, drafts, and publishing lifecycle: Implement all Elysia HTTP endpoints for the social media content lifecycle: upload photos, list/view/approve/reject/publish drafts, and list published content. Wire up all services and pipelines.
- Write comprehensive integration and end-to-end tests for social media engine: Create a full test suite covering all social media engine endpoints and pipelines end-to-end, with mocked external services (AI, social platforms, Signal, S3), verifying the complete content lifecycle from upload to publish.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.