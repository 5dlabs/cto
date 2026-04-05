## Decision Points

- Which object storage provider should be used for product images, event photos, and other media assets?
- Which PostgreSQL operator should be used for managing the main transactional database?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- How should API versioning be handled for public and internal APIs?
- Which AI model provider should be used for image scoring and caption generation — OpenAI (GPT-4 Vision) or Anthropic (Claude)?
- How should the Signal messaging integration for the approval workflow be implemented?

## Coordination Notes

- Agent owner: Nova
- Primary stack: Node.js 20+/Elysia 1.x + Effect