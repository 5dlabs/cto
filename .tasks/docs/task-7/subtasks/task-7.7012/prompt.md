Implement subtask 7012: Implement conversation state management and context window handling in Valkey

## Objective
Build the shared conversation state management layer that stores conversation history in Valkey across all channels (Signal, web chat, voice), manages the 50-message context window, and provides skill routing based on intent detection.

## Steps
1. Create ConversationManager module:
   - Unified interface for storing/retrieving conversations regardless of channel
   - Key scheme: `conv:{channel}:{identifier}` (e.g., conv:signal:+1234567890, conv:web:uuid, conv:voice:+1234567890)
2. Message storage format:
   - Each message: { role: 'user'|'assistant'|'system'|'tool', content: string, timestamp: ISO8601, channel: string, tool_name?: string, tool_result?: object }
   - Store as Valkey list or JSON array
3. Context window management:
   - Always include system prompt as first message
   - Keep last 50 messages in context
   - When exceeding 50: summarize older messages using LLM → store summary as a system message
   - Ensure tool call/result pairs are never split across the window boundary
4. Skill routing logic:
   - Analyze latest user message for intent signals:
     - Equipment/rental/event inquiry keywords → `sales-qual` skill
     - Invoice/payment/billing keywords → `finance` skill
     - Social media/post/content keywords → `social-media` skill
     - General question → default conversational mode
   - Active skill persists across messages until conversation topic changes or user explicitly switches
   - Skill context: when a skill is active, include skill-specific instructions in the system prompt
5. Cross-channel session linking:
   - If a Signal user's phone number matches a web chat session's provided phone, link conversations
   - Provide conversation continuity: 'I see we were discussing X earlier via text'
6. Implement cleanup: conversations older than 7 days are archived or purged.

## Validation
Store 60 messages in a conversation and verify only the last 50 + system prompt are returned in the context window. Verify summarization triggers when messages exceed 50. Test skill routing: send 'I need to rent some lights for an event' and verify sales-qual skill is activated. Test cross-channel: create a Signal conversation, then query by same phone number from web chat and verify linking works. Verify 7-day TTL cleanup.