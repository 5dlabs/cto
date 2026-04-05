## Decision Points

- AI model selection: OpenAI Vision API vs Claude for photo scoring and caption generation — cost, latency, and quality trade-offs need evaluation before implementation
- Postgres client library choice: `@effect/sql-pg` (tighter Effect integration but less mature) vs `postgres` (postgresjs — battle-tested but requires manual Effect wrapping)
- Instagram publishing: Instagram Graph API requires a Meta App Review process and business account — confirm whether OAuth tokens and app approval are already in place or if a sandbox/mock strategy is needed for v1
- TikTok publishing: TikTok Content Posting API has restrictive access requirements — confirm whether API access is approved or if TikTok support should be deferred to a later iteration

## Coordination Notes

- Agent owner: nova
- Primary stack: Node.js 20+/Elysia 1.x + Effect 3.x