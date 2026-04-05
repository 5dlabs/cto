Implement subtask 7003: Implement ElevenLabs voice integration for voice channel

## Objective
Integrate ElevenLabs TTS/STT with the Morgan agent to enable voice-based customer interactions, including speech-to-text input and text-to-speech output.

## Steps
1. Configure ElevenLabs API credentials from Kubernetes secrets.
2. Implement the voice input pipeline: receive audio → ElevenLabs STT → text → forward to agent conversation loop.
3. Implement the voice output pipeline: agent text response → ElevenLabs TTS → audio stream → deliver to caller.
4. Select and configure the appropriate ElevenLabs voice ID for the Morgan persona.
5. Implement voice session management (start call, maintain context during call, end call).
6. Handle telephony integration point (SIP/WebRTC/PSTN gateway) for actual voice calls.
7. Set up proper audio format handling (sample rate, codec) for streaming.

## Validation
Initiate a test voice call; verify STT correctly transcribes spoken input; verify TTS generates natural-sounding audio response; round-trip latency from speech input to audio response is under 10 seconds; voice session maintains conversation context across multiple turns.