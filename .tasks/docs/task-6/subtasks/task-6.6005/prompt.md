Implement subtask 6005: Implement draft management CRUD endpoints

## Objective
Build the draft management endpoints: GET /api/v1/social/drafts, GET /api/v1/social/drafts/:id, and draft creation flow that combines uploads with AI-generated captions into reviewable drafts.

## Steps
1. In `src/routes/drafts.ts`, implement: a) `POST /api/v1/social/drafts` — accepts upload_ids and optional manual caption, triggers AI caption generation for each target platform, creates a Draft record with status PENDING_REVIEW, returns 201 with draft details including AI captions. b) `GET /api/v1/social/drafts` — lists all drafts with pagination and optional status filter, returns 200 with array. c) `GET /api/v1/social/drafts/:id` — returns single draft with full details including uploads and captions, returns 200 or 404. 2. Validate all requests/responses with Effect.Schema. 3. Draft creation should call AICaptionService.generateCaption for each target platform. 4. Include presigned URLs for images in draft responses so reviewers can preview.

## Validation
POST /drafts with valid upload_ids creates draft with AI-generated captions and PENDING_REVIEW status; GET /drafts returns paginated list; GET /drafts/:id returns correct draft; 404 for non-existent draft; Effect.Schema validates all inputs/outputs.