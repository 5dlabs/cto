## Decision Points

- Which object storage provider should be used for product images and social media photos?
- How should service-to-service communication be handled between backend services (e.g., Morgan agent, RMS, Catalog, Finance, Vetting, Social Engine)?
- Should all services share a single PostgreSQL cluster with multiple schemas, or should each service have its own isolated database instance?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- Which AI model should be used for caption generation and image curation — OpenAI (GPT-4o/DALL-E) or Anthropic Claude? Each has different strengths for creative text generation and image understanding.
- How should the approval workflow notify Morgan via Signal? Direct Signal API integration, or through an intermediary bot framework?

## Coordination Notes

- Agent owner: nova
- Primary stack: Node.js 20+/Elysia + Effect