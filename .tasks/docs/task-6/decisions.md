## Decision Points

- AI model selection for image curation: OpenAI Vision API (gpt-4o) vs. a cheaper model (gpt-4o-mini) — cost vs. quality trade-off for scoring image composition, lighting, subject clarity
- AI model selection for caption generation: OpenAI (GPT-4o) vs. Anthropic Claude — different tone/style characteristics, pricing, and API availability
- ORM choice: drizzle-orm vs. kysely for database migrations and query building — both are mentioned as options in the details
- Draft status state machine: should 'partially_published' be a distinct status separate from 'published' and 'failed', or should per-platform statuses be tracked only at the published_posts level?
- Instagram publishing approach: Instagram Graph API requires a Facebook Business account and media container flow — confirm whether the client already has Business accounts set up for all 4 platforms with appropriate API access tokens

## Coordination Notes

- Agent owner: nova
- Primary stack: Node.js 20+/Elysia + Effect