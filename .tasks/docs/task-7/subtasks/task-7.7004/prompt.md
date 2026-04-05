Implement subtask 7004: Integrate Twilio for phone and SIP connectivity

## Objective
Configure Twilio for inbound/outbound phone calls and SIP trunking, connecting voice calls to the Morgan agent with ElevenLabs voice output.

## Steps
1. Configure a Twilio phone number and set its webhook URL to point to the Morgan agent's voice endpoint (via ingress or NodePort).
2. Implement TwiML response handler: on inbound call, stream audio to Morgan, get text response, synthesize via ElevenLabs, stream audio back.
3. Configure SIP trunk in Twilio for business phone system integration.
4. Handle call events: answer, hangup, DTMF input, call transfer.
5. Implement outbound calling capability for Morgan to initiate calls (e.g., follow-up with leads).
6. Store Twilio Account SID and Auth Token in Kubernetes Secret.

## Validation
Place a test call to the Twilio number; verify Morgan answers and responds with voice. Test SIP trunk connectivity. Verify DTMF handling and call transfer. Check outbound call initiation via API.