Implement subtask 7010: Configure Twilio SIP trunk and phone number routing to ElevenLabs

## Objective
Set up Twilio phone number provisioning, SIP trunk configuration pointing to ElevenLabs voice endpoint, and fallback webhook routing to Morgan's text endpoint.

## Steps
1. Configure Twilio phone number:
   - Use existing Perception Events business number or provision new one via Twilio API/console
   - Store phone number SID and auth token as Kubernetes secrets
2. Configure Twilio SIP trunk:
   - Create SIP trunk in Twilio console/API
   - Set origination URI to ElevenLabs SIP endpoint (provided by ElevenLabs Conversational AI setup)
   - Configure codec preferences: PCMU, PCMA, opus
   - Set authentication credentials for SIP trunk
3. Configure call routing:
   - Incoming calls to Twilio number → SIP trunk → ElevenLabs → Morgan voice pipeline
   - Set TwiML fallback: if ElevenLabs is unavailable, redirect to Twilio webhook → Morgan text endpoint
4. Implement Twilio webhook fallback endpoint in Morgan:
   - `/api/twilio/fallback` — receives Twilio webhook on voice failure
   - Responds with TwiML: play a message ('Our voice system is temporarily unavailable, please send us a text message at this number')
   - Or: forward to SMS-based conversation
5. Configure Twilio SMS webhook:
   - Incoming SMS → POST to Morgan's `/api/twilio/sms` endpoint
   - Morgan responds via Twilio SMS API
6. Store all Twilio credentials (Account SID, Auth Token, Phone Number SID) in sigma1-external-secrets.

## Validation
Verify Twilio phone number is configured with correct SIP trunk. Make a test call and verify it routes through to ElevenLabs (or logs the attempt). Test fallback: disable ElevenLabs endpoint and verify Twilio falls back to webhook. Send test SMS and verify Morgan receives it via /api/twilio/sms webhook.