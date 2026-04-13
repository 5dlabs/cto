<identity>
You are nova working on subtask 6005 of task 6.
</identity>

<context>
<scope>
Implement the async curation pipeline that scores each uploaded photo using OpenAI gpt-4o vision API, selects the top 5-10 photos, and updates the draft's selected_photo_urls.
</scope>
</context>

<implementation_plan>
Create src/pipelines/curation.ts. Function runCurationPipeline(draft_id, source_urls): for each source URL, call OpenAI chat.completions.create with model='gpt-4o', messages containing image_url content blocks and a scoring prompt: 'Score this photo on composition (0-10), lighting (0-10), and subject clarity (0-10). Return JSON {composition, lighting, clarity}.' Parse response JSON. Compute total = composition+lighting+clarity. Sort photos by total descending. Select top min(10, all) where total >= 15 (threshold for quality gate); ensure at least 1 photo is selected regardless of threshold. Update social_drafts SET selected_photo_urls=$1, status='ready_for_approval' WHERE id=$2. Wrap entire pipeline in try/catch; on error set status='pending_curation' with error logged via pino. Use OPENAI_API_KEY from environment.
</implementation_plan>

<validation>
Integration test with mocked OpenAI API (vitest mock): supply 5 photos with mock scores [28, 22, 15, 10, 5]. Verify selected_photo_urls contains the top 3 (scores >= 15) and draft status transitions to 'ready_for_approval'. Test error path: OpenAI throws → draft status remains 'pending_curation' and error is logged.
</validation>