## Develop Social Media Engine (Nova - Node.js/Elysia)

### Objective
Implement the social media backend for AI curation, caption generation, approval workflow, and multi-platform publishing using Elysia and Effect.

### Ownership
- Agent: nova
- Stack: Node.js/Elysia
- Priority: medium
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps":["Initialize Node.js 20+ project with Elysia 1.x and Effect 3.x, using POSTGRES_URL and S3_URL from ConfigMap.","Define endpoints for photo upload, draft management, approval, publishing, and listing published posts.","Integrate OpenAI/Claude for caption generation.","Implement Effect.Service patterns for Instagram, LinkedIn, TikTok, Facebook APIs.","Implement AI curation pipeline to select top images.","Implement approval workflow (Signal integration via Morgan).","Sync published content to website portfolio via webhook or API.","Add request/response validation with Effect.Schema."]}

### Subtasks
- [ ] Initialize Elysia/Effect project with database schema and validation setup: Scaffold the Node.js 20+ project with Elysia 1.x and Effect 3.x, configure PostgreSQL and S3 connectivity from ConfigMap, create database migrations for social media models, and set up Effect.Schema validation.
- [ ] Implement photo upload endpoint and S3 storage: Build the photo upload endpoint that accepts image files, stores them in S3-compatible object storage, and persists metadata in PostgreSQL.
- [ ] Implement AI curation pipeline for image selection: Build the AI-powered curation pipeline that analyzes uploaded photos and selects the top images for social media posting using OpenAI/Claude vision capabilities.
- [ ] Implement AI caption generation service: Build the AI-powered caption generation service that creates platform-specific captions for curated images using OpenAI/Claude.
- [ ] Implement multi-platform publishing services using Effect.Service: Build Effect.Service implementations for publishing content to Instagram, LinkedIn, TikTok, and Facebook, each as a separate service with platform-specific API integration.
- [ ] Implement approval workflow with Signal integration via Morgan: Build the approval workflow that sends draft content for review via Signal (through Morgan integration) and processes approval/rejection decisions.
- [ ] Implement website portfolio sync via webhook/API: Build the webhook/API integration that syncs published social media content to the company website portfolio.
- [ ] Add Prometheus metrics and health endpoints: Implement Prometheus metrics exposition and health/readiness probe endpoints for the social media engine.
- [ ] End-to-end social media pipeline integration tests: Write comprehensive integration tests validating the full social media pipeline: upload → curation → caption → approval → publish → sync.