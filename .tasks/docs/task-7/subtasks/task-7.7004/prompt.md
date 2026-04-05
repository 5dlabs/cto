Implement subtask 7004: Integrate Twilio for phone/SIP voice channel

## Objective
Set up Twilio integration for inbound/outbound voice calls and SIP, routing voice interactions through Morgan's agent runtime with ElevenLabs TTS output.

## Steps
1. Configure a Twilio phone number and set up webhook endpoints for incoming calls.
2. Implement a Twilio voice webhook handler that receives inbound call events and initiates a conversation with the OpenClaw agent.
3. Implement speech-to-text processing for caller audio (using Twilio's built-in STT or a separate service).
4. Route transcribed text to the agent runtime, receive text responses, and pipe them through the ElevenLabs voice synthesis module (7003).
5. Stream or play synthesized audio back to the caller via Twilio's TwiML or Media Streams API.
6. Handle call lifecycle events: ringing, connected, on-hold, transfer, hangup.
7. Implement SIP endpoint configuration if required for business phone system integration.
8. Add logging for call metadata (duration, caller ID, transcript summary, latency).

## Validation
Place a test call to the Twilio number; Morgan answers, transcribes speech, generates a response, and plays audio back; round-trip latency for a simple query is <10s; call lifecycle events are logged; SIP endpoint is reachable if configured.