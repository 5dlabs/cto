Implement subtask 8011: Build Morgan web chat widget with WebSocket connection and rich messages

## Objective
Implement the persistent floating chat widget for Morgan AI agent: bottom-right floating button, expandable near-full-screen chat UI, WebSocket connection, rich message support (product cards, quote summaries), typing indicators, localStorage message persistence, and minimizable state.

## Steps
1. Create `components/custom/ChatWidget.tsx` — Client Component, rendered in root layout.
2. Floating trigger button:
   - Fixed bottom-right position, circular button with chat icon and badge for unread count.
   - Animated pulse/glow effect to draw attention (subtle, not obnoxious).
3. Expanded chat interface:
   - Near-full-screen panel (mobile: full screen, desktop: right-side panel ~400px wide, ~80vh tall).
   - Header: 'Morgan' name, avatar, status indicator (online/typing), minimize/close buttons.
   - Message list: scrollable, auto-scrolls to bottom on new messages.
   - Input area: text input + send button, Enter to send.
4. WebSocket connection (`hooks/useChatSocket.ts`):
   - Connect to `NEXT_PUBLIC_WS_URL` with session ID.
   - Generate/persist session ID in localStorage.
   - Handle events: message, typing_start, typing_stop, error, connection status.
   - Auto-reconnect with exponential backoff on disconnect.
   - Effect-based WebSocket wrapper for clean error handling.
5. Message types and rendering (`components/custom/ChatBubble.tsx`):
   - Text messages: plain text bubbles (user right-aligned, Morgan left-aligned).
   - Product cards: inline card with product image, name, price, 'View' link to `/equipment/:id`.
   - Quote summaries: mini table of products, dates, estimated total.
   - Availability results: date grid or text summary.
   - Message status: sent ✓, delivered ✓✓ indicators.
6. Typing indicator: animated dots shown when Morgan is composing.
7. Persistence:
   - Store message history in localStorage keyed by session ID.
   - On page load, restore previous messages.
   - Clear history option in settings.
8. State management:
   - Widget open/closed/minimized state persisted in localStorage.
   - Remembers state across page navigations (SPA navigation doesn't unmount).
9. Accessibility: focus trap when chat is open, Escape to close, aria-live for new messages.

## Validation
Unit test ChatWidget: verify floating button renders, click opens chat panel, click minimize collapses to button. Mock WebSocket: send a text message, verify it appears in message list as user bubble. Simulate Morgan response, verify it appears as Morgan bubble. Test rich messages: send mock product card message, verify image and product name render. Test typing indicator: simulate typing_start event, verify dots animation shown. Test localStorage persistence: send messages, unmount component, remount, verify messages restored. Test reconnection: simulate WebSocket close, verify reconnect attempt after delay.