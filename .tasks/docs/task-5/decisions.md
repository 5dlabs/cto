## Decision Points

- What is the expected payload format for discord-bridge-http? Does it support Discord rich embeds natively, or does it expect a simplified schema that the bridge transforms internally? This must be discovered from the bridge's source or docs before implementing.
- What is the expected payload format for linear-bridge? Does it accept free-form notification text, or does it require structured fields (e.g., project ID, issue references)? This must be discovered from the bridge's source or docs before implementing.

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia