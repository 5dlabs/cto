Implement subtask 7003: Integrate ElevenLabs voice channel

## Objective
Configure ElevenLabs TTS/STT as the voice channel for Morgan, enabling customers to interact via voice calls. Wire speech-to-text input and text-to-speech output through the OpenClaw agent.

## Steps
1. Set up ElevenLabs API credentials as a Kubernetes secret and mount into the agent pod.
2. Implement the voice channel adapter: (a) accept inbound audio streams (via telephony integration or WebRTC endpoint), (b) transcribe speech to text using ElevenLabs STT or a configured STT provider, (c) send transcribed text to the Morgan agent, (d) convert agent text responses to speech using ElevenLabs TTS, (e) stream audio back to the caller.
3. Select and configure an appropriate ElevenLabs voice ID for Morgan's persona.
4. Handle conversation turn-taking: detect end-of-speech, manage silence timeouts, and support barge-in.
5. Implement error handling for ElevenLabs API failures (fallback to text or retry).
6. Expose the voice endpoint for telephony or WebRTC connection.

## Validation
Initiate a test voice call to the voice endpoint. Speak a simple query and verify: (1) speech is transcribed correctly, (2) agent generates a relevant response, (3) response is spoken back in the configured ElevenLabs voice within 10 seconds total latency. Verify error handling by simulating an ElevenLabs API timeout.