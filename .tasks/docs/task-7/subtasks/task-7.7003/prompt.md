Implement subtask 7003: Integrate ElevenLabs for voice synthesis

## Objective
Configure the Morgan agent to use ElevenLabs API for text-to-speech voice responses, including voice selection, streaming audio output, and error handling.

## Steps
1. Store ElevenLabs API key in a Kubernetes Secret and reference it in the agent's environment.
2. Implement a voice synthesis module that takes agent text responses and calls ElevenLabs text-to-speech API.
3. Select and configure the appropriate voice ID for the Morgan persona.
4. Support streaming audio output for low-latency voice responses.
5. Implement fallback behavior if ElevenLabs is unavailable (e.g., return text-only response with error flag).
6. Ensure audio format compatibility with Twilio (mulaw/8000 or appropriate codec).

## Validation
Call the voice synthesis module with sample text; verify audio bytes are returned in the expected format. Measure latency is under 3 seconds for a short sentence. Simulate ElevenLabs API failure and verify fallback behavior.