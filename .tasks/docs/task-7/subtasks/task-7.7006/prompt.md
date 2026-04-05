Implement subtask 7006: Implement MCP Tool Server — Social Engine tools (social_curate, social_publish)

## Objective
Define and implement MCP tool definitions for the two Social Engine tools: sigma1_social_curate and sigma1_social_publish.

## Steps
1. Create MCP tool definition for `sigma1_social_curate`:
   - HTTP: POST /api/v1/social/upload
   - Input schema: { media_url?: string, media_base64?: string, caption?: string, platform: 'instagram'|'tiktok'|'facebook'|'all', tags?: string[] }
   - Output schema: { draft_id: string, preview_url: string, status: 'draft', platforms: string[], suggested_caption?: string }
   - Description: 'Upload media content and create a social media draft for review before publishing'
2. Create MCP tool definition for `sigma1_social_publish`:
   - HTTP: POST /api/v1/social/drafts/:id/publish
   - Input schema: { draft_id: string, schedule_time?: string (ISO8601), platforms?: string[] }
   - Output schema: { draft_id: string, status: 'published'|'scheduled', published_urls: [{ platform, url }], published_at?: string }
   - Description: 'Publish or schedule a previously created social media draft to specified platforms'
3. Include Authorization header: `Bearer ${MORGAN_SERVICE_JWT}`
4. Implement URL path parameter substitution for draft_id in social_publish.
5. For social_curate, handle multipart/form-data if media_base64 is provided.
6. Error handling: invalid media format → user-friendly error, draft not found for publish → clear message.

## Validation
Invoke sigma1_social_curate with test media URL and verify correct HTTP POST body. Mock Social Engine and verify draft creation response parsing. Invoke sigma1_social_publish with test draft_id and verify URL path substitution and response parsing. Test with non-existent draft_id and verify 404 handling.