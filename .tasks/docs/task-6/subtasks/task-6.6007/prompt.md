Implement subtask 6007: Implement caption generation via OpenAI chat API

## Objective
Implement the caption generation step that assembles event context and calls the OpenAI chat API to generate a caption and hashtags, storing results on the social_drafts row.

## Steps
Create src/pipelines/caption.ts. Function generateCaption(draft_id, event_id): fetch event context from the catalog service (GET {CATALOG_SERVICE_URL}/api/v1/events/{event_id} — use CATALOG_SERVICE_URL env var). Assemble prompt: 'Write a social media caption for a photography event. Event name: {name}. Venue: {venue}. Equipment used: {equipment}. Generate a caption (max 200 words) and 10 relevant hashtags. Return JSON {caption: string, hashtags: string[]}.' Call openai.chat.completions.create with model='gpt-4o', temperature=0.7. Parse response JSON. UPDATE social_drafts SET caption=$1, hashtags=$2 WHERE id=$3. Call generateCaption at end of runCurationPipeline before setting status='ready_for_approval'. On catalog service fetch failure, use fallback context {name:'Photography Event', venue:'', equipment:[]}.

## Validation
Unit test with mocked OpenAI and mocked catalog service: verify caption and hashtags are non-empty strings after pipeline run. Verify caption is stored in the social_drafts row. Test catalog service failure fallback: still produces a caption using fallback context. Verify hashtags array contains exactly 10 items.