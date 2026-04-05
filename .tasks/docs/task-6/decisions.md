## Decision Points

- Which object storage provider should be used for product images and social media photos?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service API calls?
- How should the public-facing API endpoints (e.g., Equipment Catalog, RMS REST, Finance) be versioned and exposed?
- Which AI model/provider should be used for caption generation and image curation — OpenAI (GPT-4V for images, GPT-4 for text) or Anthropic Claude?
- How should the Signal-based approval workflow be implemented — direct Signal API integration, or via Morgan (OpenClaw) as an intermediary?
- How should Instagram, LinkedIn, and Facebook API tokens/credentials be managed for multi-platform publishing?

## Coordination Notes

- Agent owner: Nova
- Primary stack: Node.js/Elysia