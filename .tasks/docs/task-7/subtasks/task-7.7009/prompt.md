Implement subtask 7009: Implement ElevenLabs voice integration with WebSocket streaming

## Objective
Configure the ElevenLabs Conversational AI agent with Morgan's voice profile and implement the WebSocket-based real-time voice streaming pipeline for phone call interactions.

## Steps
1. Configure ElevenLabs Conversational AI agent:
   - Create or select voice profile for Morgan (professional, clear, warm tone)
   - Set up ElevenLabs agent with Morgan's system prompt (aligned with OpenClaw manifest)
   - Configure the agent to forward tool calls to Morgan's MCP tool server
2. Implement WebSocket connection to ElevenLabs streaming API:
   - Establish persistent WebSocket connection for real-time audio streaming
   - Handle audio chunks: receive PCM/opus audio from ElevenLabs, stream to caller
   - Send user audio: receive audio from Twilio, forward to ElevenLabs for transcription
3. Implement voice-to-text pipeline:
   - ElevenLabs handles STT internally → receive transcribed text
   - Forward transcribed text to Morgan's conversation handler
   - Receive Morgan's text response → send to ElevenLabs for TTS
4. Handle conversation state continuity between voice and text channels:
   - Voice calls should be able to reference prior text conversations if same phone number
   - Store voice call session in Valkey with phone number as key
5. Implement graceful handling of voice-specific scenarios:
   - Silence detection / timeout → polite prompt
   - Call disconnect → save conversation state
   - Audio quality issues → fallback to text summary
6. Store ElevenLabs API key as Kubernetes secret, reference via sigma1-external-secrets.

## Validation
Verify ElevenLabs WebSocket connection establishes successfully with valid API key. Send test audio and verify transcription is received. Send test text and verify audio response is generated. Verify conversation state is stored in Valkey keyed by phone number. Test disconnect/reconnect preserves state.