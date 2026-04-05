Implement subtask 7003: Integrate ElevenLabs voice synthesis for voice responses

## Objective
Implement ElevenLabs TTS integration so Morgan can generate spoken audio responses for voice channel interactions.

## Steps
1. Configure ElevenLabs API credentials and select a voice ID for Morgan's persona.
2. Implement a voice synthesis module that accepts text responses from the agent and calls the ElevenLabs TTS API to generate audio.
3. Handle streaming audio output if supported, or buffer and return complete audio files.
4. Implement audio format conversion if needed (e.g., WAV to MP3/OGG for Twilio compatibility).
5. Add error handling for API rate limits, timeouts, and fallback to text-only responses.
6. Log all voice synthesis requests (text length, audio duration, latency, API cost estimate).

## Validation
Submit a text string to the voice synthesis module and receive valid audio output; audio plays correctly in a standard player; latency from text submission to audio availability is <3s for short responses; error fallback triggers on simulated API failure.