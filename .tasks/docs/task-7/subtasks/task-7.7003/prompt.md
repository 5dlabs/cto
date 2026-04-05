Implement subtask 7003: Integrate ElevenLabs for voice interactions

## Objective
Implement the voice channel for Morgan using ElevenLabs text-to-speech and speech-to-text APIs, enabling voice-based customer interactions with natural-sounding responses.

## Steps
Step 1: Configure ElevenLabs API credentials and select a voice profile for Morgan. Step 2: Implement the speech-to-text pipeline — receive audio input (from phone/VoIP integration or web), send to ElevenLabs STT, receive transcript. Step 3: Implement the text-to-speech pipeline — take agent text responses, send to ElevenLabs TTS, receive audio stream. Step 4: Implement audio streaming for low-latency voice responses (chunked transfer). Step 5: Handle voice-specific conversation flow: greeting, interruption handling, silence detection, end-of-conversation. Step 6: Wire the voice channel into the OpenClaw agent runtime as an input/output channel alongside Signal and web chat. Step 7: Add latency tracking for voice round-trips to ensure <10s total response time.

## Validation
Speak a test query to the voice endpoint and receive an audible response from Morgan; verify transcript accuracy matches intent; measure round-trip latency is under 10 seconds for simple queries; voice profile sounds natural and consistent.