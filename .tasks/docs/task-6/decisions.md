## Decision Points

- What is the primary inter-service communication paradigm for backend microservices (synchronous REST/gRPC vs asynchronous event-driven messaging)?
- Which object storage provider should be used for product images and social media photos (Cloudflare R2 vs AWS S3)?
- Which PostgreSQL operator should be used for the main database cluster?
- How should multi-tenancy and data isolation be handled in the PostgreSQL schema (single database with multiple schemas vs separate databases per service)?
- What authentication and authorization mechanism should be used for internal service-to-service communication?
- How should the REST API versioning strategy be handled for all public endpoints?
- Which AI model provider should be the primary service for caption generation — OpenAI or Anthropic Claude? Should there be a fallback chain or a single provider?
- How should Signal notifications be sent to Morgan for the approval workflow? Signal does not have an official public API for bots.
- Which TikTok API integration path should be used? TikTok's Content Posting API has strict approval requirements and limited availability.

## Coordination Notes

- Agent owner: nova
- Primary stack: Node.js 20+, Elysia 1.x, Effect TS