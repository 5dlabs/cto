Implement subtask 6006: Implement draft management endpoints and AI-powered draft creation

## Objective
Build the draft CRUD endpoints that tie together photo uploads and AI caption generation to create social media drafts ready for approval.

## Steps
1. Create `src/routes/drafts.ts` with the following Elysia routes: GET /api/v1/social/drafts (list all drafts with filtering by status, platform), GET /api/v1/social/drafts/:id (get single draft with photo and caption details), POST /api/v1/social/drafts (create draft: accepts photo_id and optional platform, triggers AI caption generation), PUT /api/v1/social/drafts/:id (update draft caption/platform manually), DELETE /api/v1/social/drafts/:id (soft-delete draft). 2. Define Effect.Schema for each request/response. Draft list response includes pagination (limit, offset, total). 3. POST /api/v1/social/drafts flow: validate photo_id exists → call AICurationService.analyzePhoto → for each suggested platform (or specified platform), call AICurationService.generateCaption → insert draft records into social_drafts with status='draft' → return created drafts. 4. Allow creating multiple drafts (one per platform) from a single photo in one request. 5. Add filtering: ?status=draft|pending_approval|approved|rejected|published, ?platform=instagram|linkedin|facebook|tiktok. 6. Add Prometheus counter for drafts created (social_drafts_created_total) with platform label.

## Validation
POST a draft with a valid photo_id and verify AI-generated captions are stored. GET /drafts and verify list pagination works with status/platform filters. GET /drafts/:id returns full draft details including photo URLs. PUT updates caption text. DELETE soft-deletes and draft no longer appears in default list.