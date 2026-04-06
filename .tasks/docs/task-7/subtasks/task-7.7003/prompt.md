Implement subtask 7003: Integrate ElevenLabs and Twilio for voice channel

## Objective
Configure the voice pipeline: Twilio receives inbound calls and streams audio, ElevenLabs handles text-to-speech for agent responses, and a speech-to-text service transcribes caller input. Wire this pipeline into the OpenClaw agent's conversation flow.

## Steps
1. Set up Twilio webhook endpoint in the agent (or a small adapter service) to receive inbound voice calls.
2. Configure Twilio to stream audio / use TwiML to gather speech input.
3. Implement speech-to-text transcription of caller audio (using Twilio's built-in STT or a separate provider).
4. Route transcribed text to the OpenClaw agent conversation endpoint.
5. Take agent text responses and send them to ElevenLabs TTS API to generate audio.
6. Stream synthesized audio back to the caller via Twilio.
7. Store Twilio Account SID, Auth Token, and ElevenLabs API key as Kubernetes Secrets; mount into the pod.
8. Configure ElevenLabs voice ID, model, and latency settings for <10s response target.
9. Handle call lifecycle: greeting, conversation turns, hangup/timeout.

## Validation
Place an inbound call to the Twilio number; verify the caller hears a greeting; speak a simple query and verify the agent responds with synthesized speech; end-to-end voice round-trip completes; Twilio webhook logs show successful call handling; ElevenLabs API calls return 200.