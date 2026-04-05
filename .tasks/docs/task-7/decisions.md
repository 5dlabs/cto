## Decision Points

- Model selection: gpt-5.4-pro vs gpt-4o — availability, cost, latency trade-offs. Is gpt-5.4-pro actually available in the OpenAI API? Fallback strategy if model is deprecated or rate-limited.
- ElevenLabs Conversational AI integration architecture: Does ElevenLabs handle the full voice-to-text-to-voice loop natively, or does Morgan need to receive transcribed text and send back text for synthesis separately? This affects whether the voice pipeline is a direct ElevenLabs↔Twilio integration or requires Morgan as a real-time intermediary.
- Signal-CLI message ingestion: polling vs webhook — polling is simpler but adds latency; webhook requires Signal-CLI to support outbound HTTP callbacks. Which mode does the Signal-CLI REST API version in use support?
- Signal account rotation: How should failover between primary and secondary Signal numbers work? Automatic (risk of split-brain conversations) vs manual (Mike switches via admin command)? How are ongoing conversations migrated to the new number?
- Conversation state persistence: File-based on PVC vs a lightweight database (SQLite on PVC, or external PostgreSQL). File-based is simpler but harder to query for analytics. Does OpenClaw have a native conversation store?

## Coordination Notes

- Agent owner: angie
- Primary stack: OpenClaw/MCP