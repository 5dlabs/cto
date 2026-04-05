Implement subtask 7007: Implement social media skills: social-curate and social-publish

## Objective
Develop skills for Morgan to curate and publish social media content using the sigma1_social_curate and sigma1_social_publish MCP tools.

## Steps
1. Implement the social-curate skill: accept content curation requests (topic, event, equipment showcase), invoke sigma1_social_curate to generate content suggestions (captions, image selections, hashtags), and present options for approval.
2. Implement the social-publish skill: after content approval, invoke sigma1_social_publish with finalized content, target platforms, and scheduling parameters. Confirm publication status.
3. Handle multi-platform publishing (e.g., Instagram, Facebook) if supported by the backend.
4. Implement approval workflow: curated content is presented for human approval before publishing.
5. Add logging for all social media operations (content type, platform, publish status, scheduling time).

## Validation
Social-curate skill invokes sigma1_social_curate and returns content suggestions; social-publish skill invokes sigma1_social_publish and confirms publication; approval workflow correctly gates publishing; logs capture all social media operations with correct metadata.