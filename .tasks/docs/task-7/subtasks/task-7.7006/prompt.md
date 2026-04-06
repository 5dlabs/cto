Implement subtask 7006: Configure ElevenLabs voice adapter and Twilio TwiML webhook in OpenClaw

## Objective
Set up ElevenLabs TTS voice configuration using the eleven_turbo_v2 model and define the Twilio inbound call TwiML webhook handler so Morgan can receive and respond to phone calls.

## Steps
1. In agents/morgan/agent.yaml, add voice adapter block:
   adapters:
     voice:
       provider: elevenlabs
       api_key: ${ELEVENLABS_API_KEY}
       voice_id: ${ELEVENLABS_VOICE_ID}
       model: eleven_turbo_v2
       output_format: ulaw_8000  # compatible with Twilio
2. Add Twilio webhook adapter block:
   adapters:
     twilio_voice:
       enabled: true
       inbound_webhook_path: /voice/inbound
       twiml_response: true
       sip_domain: ${TWILIO_SIP_DOMAIN}
3. The /voice/inbound endpoint must return TwiML that: (a) greets the caller with a synthesized ElevenLabs message via <Play> or <Say> with voice attribute, (b) captures speech input with <Gather>, (c) POSTs transcription back to Morgan for processing.
4. Map ELEVENLABS_API_KEY and ELEVENLABS_VOICE_ID from Kubernetes secrets in the Deployment (handled in subtask 7008).
5. Document call flow in agents/morgan/README.md under 'Voice Integration'.

## Validation
POST to Morgan's /voice/inbound webhook (via port-forward or Cloudflare Tunnel URL) with a mock Twilio inbound call payload. Response must be valid TwiML XML (Content-Type: application/xml) containing <Response> root element. ElevenLabs API key validation: `openclaw agent test-voice morgan --text 'hello'` returns an audio file.