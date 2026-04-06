Implement subtask 7003: Integrate ElevenLabs voice synthesis and Twilio phone channel

## Objective
Set up ElevenLabs for text-to-speech and Twilio for inbound/outbound phone calls so Morgan can handle voice interactions.

## Steps
1. Configure the ElevenLabs API client within the agent, using API key from Secrets, selecting an appropriate voice ID for Morgan's persona.
2. Implement a text-to-speech adapter that converts agent text responses to audio streams via ElevenLabs API.
3. Set up a Twilio phone number and configure the webhook URL to point to Morgan's voice endpoint (via ingress or NodePort).
4. Implement a Twilio webhook handler that receives inbound calls, streams audio to a speech-to-text service, and passes transcribed text to the agent.
5. Implement the response path: agent text response → ElevenLabs TTS → Twilio TwiML <Play> or <Stream> back to caller.
6. Handle call lifecycle events: call start, end, transfer, voicemail.
7. Ensure voice round-trip latency meets the 10-second SLA for simple queries.

## Validation
Place a test call to the Twilio number; verify Morgan answers and responds with synthesized voice; measure round-trip latency is under 10 seconds for a simple greeting; verify call end is handled gracefully.