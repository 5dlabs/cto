## Decision Points

- LLM model selection: gpt-5.4-pro is specified but may not exist yet — what is the actual model to use (gpt-4o, gpt-4-turbo, Claude 3.5 Sonnet)? Should there be a fallback model?
- OpenClaw runtime version and deployment model: is OpenClaw a specific OSS framework or a custom internal runtime? Need to confirm the exact runtime image, configuration format, and tool-server protocol (MCP stdio vs HTTP SSE vs streamable HTTP).
- Signal device registration strategy: Signal requires a phone number for device registration — will Morgan use a dedicated phone number separate from Twilio? How is initial device linking handled (QR code scan, primary device registration)?
- ElevenLabs voice profile: which ElevenLabs voice ID or cloned voice should Morgan use? Is there a specific voice that matches the Perception Events brand?
- Twilio phone number: provision a new number or use an existing Perception Events business number? Same number for both voice (SIP trunk) and SMS fallback?
- Morgan system prompt ownership: who authors and maintains Morgan's persona prompt, skill routing instructions, and brand voice guidelines — is this checked into code or managed via a CMS/admin UI?
- MCP tool authentication: JWT service token issued by which identity provider? Self-signed with a shared secret, or issued by an OAuth2/OIDC provider that backend services validate?

## Coordination Notes

- Agent owner: angie
- Primary stack: OpenClaw/MCP