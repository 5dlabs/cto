Implement subtask 7008: Implement voice integration pipeline (ElevenLabs Conversational AI + Twilio SIP)

## Objective
Configure the voice call pipeline: Twilio receives PSTN calls, forwards to ElevenLabs for speech-to-text, Morgan processes the text and generates a response, ElevenLabs synthesizes speech, and Twilio plays it back to the caller.

## Steps
1. Twilio configuration:
   a. Configure Twilio SIP trunk with a phone number for Morgan.
   b. Set up Twilio webhook URL pointing to the ElevenLabs Conversational AI endpoint (or an intermediary if needed).
   c. Configure TwiML or Twilio Studio flow to handle call initiation and forwarding.
2. ElevenLabs Conversational AI setup:
   a. Configure an ElevenLabs Conversational AI agent that connects to Morgan's text processing.
   b. Select appropriate voice (professional, clear) for Morgan's persona.
   c. Configure the speech-to-text → Morgan text API → text-to-speech pipeline.
   d. Set latency optimization parameters for real-time conversation.
3. Morgan voice adapter:
   a. Expose an HTTP endpoint (or use existing WebSocket) that ElevenLabs can call with transcribed text.
   b. Process the text through Morgan's normal agent pipeline (skills, tools, etc.).
   c. Return text response for ElevenLabs to synthesize.
   d. Handle voice-specific responses: keep answers concise for voice, offer to send details via Signal/text.
4. Call flow management:
   a. Handle call start: greeting, ask how Morgan can help.
   b. Handle mid-call tool usage: when Morgan needs to call tools, use filler phrases ('Let me check that for you...').
   c. Handle call end: summarize what was discussed, offer follow-up via Signal.
   d. Handle escalation: transfer call to Mike's direct number via Twilio.
5. Store Twilio credentials (account SID, auth token) and ElevenLabs API key from Kubernetes secrets.

## Validation
Make a test call to the Twilio number, verify the call connects and ElevenLabs greeting is played. Speak a test query ('Do you have LED panels available next Saturday?'), verify Morgan processes it and a spoken response is returned within 10 seconds. Verify call transfer to Mike's number works when escalation is triggered.