Implement subtask 8009: Build persistent Morgan Web Chat Widget with WebSocket, streaming text, and session management

## Objective
Implement the Intercom-style persistent chat widget fixed to bottom-right corner that connects to Morgan's /ws/chat endpoint via WebSocket, supports streaming text display (character-by-character), manages sessions via localStorage, and survives page navigation by lifting state to the root layout.

## Steps
1. Create `components/sigma1/chat-widget.tsx` as a client component.
2. Mount the widget in `app/layout.tsx` (layout-level) so it persists across all page navigations without remounting.
3. Widget UI:
   - Collapsed state: floating circular button (bottom-right, fixed position) with Morgan avatar/icon and unread message badge.
   - Expanded state: chat panel (~350px wide, ~500px tall) with header (Morgan name/avatar, minimize button), message area (scrollable), input bar (text input + send button).
   - Message bubbles: user messages right-aligned (brand accent), Morgan messages left-aligned (muted background).
   - Typing indicator: animated dots shown when Morgan is processing.
   - Streaming text display: Morgan's responses arrive character-by-character via WebSocket; render incrementally using a state buffer that appends each character and triggers re-render.
4. WebSocket connection management in `lib/ws/chat.ts`:
   - Connect to `NEXT_PUBLIC_WS_URL/ws/chat` on widget open.
   - Reconnect with exponential backoff on disconnect (max 5 retries, then show "Connection lost" message).
   - Send messages as JSON: `{ type: 'message', content: string, sessionToken: string }`.
   - Receive messages: `{ type: 'chunk', content: string }` for streaming, `{ type: 'done' }` for completion, `{ type: 'typing' }` for typing indicator.
5. Session management:
   - On first widget open, generate a UUID session token, store in localStorage key `sigma1_chat_session`.
   - On subsequent opens, send existing token for session continuity (Morgan/Valkey backend handles session lookup).
   - Send session token in WebSocket connection URL as query param or in first message.
6. Chat history: store messages in React state (and optionally localStorage for persistence across browser refreshes).
7. Accessibility:
   - ARIA: `role="dialog"`, `aria-label="Chat with Morgan"`, `aria-live="polite"` on message area for screen reader announcements.
   - Keyboard: Escape to minimize, Tab navigation between input and send button, Enter to send.
   - Focus trap when expanded (optional, depends on design intent).

## Validation
Component test: render chat widget, click to expand, verify chat panel appears with input field. Mock WebSocket: send a message, verify it appears as user bubble. Simulate receiving streaming chunks ('H', 'e', 'l', 'l', 'o'), verify characters render incrementally in Morgan's message bubble. Test session management: verify localStorage contains session token after first open. Test navigation persistence: render widget in layout with router navigation to different page, verify widget remains open with message history intact. Test accessibility: verify role='dialog', aria-label present, Escape key minimizes widget.