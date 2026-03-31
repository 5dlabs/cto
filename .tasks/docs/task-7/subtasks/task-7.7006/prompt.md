Implement subtask 7006: Implement MCP tool-server plugins for social media and RMS tools

## Objective
Build MCP tool-server plugins for: social_curate, social_publish, and all RMS tools (rms-*). Each plugin integrates with the Social Media and RMS backend service APIs respectively.

## Steps
1. Implement the `social_curate` MCP tool plugin:
   - Input: content_type (post/story/reel), topic or equipment_id, optional tone/style.
   - Calls the Social Media curation backend service.
   - Returns suggested content (text, hashtags, image suggestions).
2. Implement the `social_publish` MCP tool plugin:
   - Input: platform (Instagram, Facebook, etc.), content text, media URLs, schedule datetime.
   - Calls the Social Media publishing backend service.
   - Returns publish confirmation with post_id and URL.
3. Identify all RMS tools (rms-*) from the spec (e.g., rms-create-reservation, rms-update-reservation, rms-cancel-reservation, rms-list-reservations, rms-check-status).
4. Implement each RMS tool as a separate MCP plugin:
   - Each takes appropriate inputs (reservation_id, customer_id, equipment_id, dates, etc.).
   - Each calls the corresponding RMS backend endpoint.
   - Each returns structured reservation data.
5. Register all social media and RMS tools with the MCP tool-server.
6. Add input validation and error handling.

## Validation
Invoke social_curate with a sample topic and verify curated content is returned. Invoke social_publish with test content and verify a publish confirmation (or sandbox success). For each RMS tool, invoke with valid test data and verify correct CRUD operations against the RMS service. Test error cases: publishing to an unsupported platform, cancelling a nonexistent reservation. All tools appear in the agent's tool registry.