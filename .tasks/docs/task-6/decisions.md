## Decision Points

- Which Redis-compatible engine should be used for caching, rate limiting, and session storage across services?
- Which object storage provider should be used for product images and social media photos?
- Which PostgreSQL operator should be used for managing the main database cluster?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for service-to-service and user-to-service communication?
- Which AI model provider should be primary for caption generation and image curation — OpenAI (GPT-4V) or Anthropic (Claude)?
- What is the 'Morgan' system referenced for Signal integration in the approval workflow — is it an existing internal service, a third-party tool, or does it need to be built?

## Coordination Notes

- Agent owner: nova
- Primary stack: Node.js/Elysia