Implement subtask 8008: Embed Morgan web chat widget

## Objective
Integrate the Morgan AI chat widget into the site as a floating chat interface available on all pages, connecting to the Morgan chat API.

## Steps
1. Create `@/components/morgan-chat/chat-widget.tsx`: a floating chat button (bottom-right corner) that opens a chat panel/dialog.
2. Implement chat UI: message list (user and Morgan messages with distinct styling), text input with send button, typing indicator, auto-scroll to latest message.
3. Connect to Morgan chat API (WebSocket or REST polling depending on backend implementation). Use Effect for message send/receive handling.
4. Store chat session state in component state; optionally persist conversation ID in sessionStorage for continuity across page navigations.
5. Add the widget to `app/layout.tsx` so it's available on every page.
6. Handle states: connecting, connected, error/reconnecting, chat minimized/maximized.
7. Add subtle entrance animation and unread message badge when minimized.
8. Ensure the widget doesn't interfere with page content (proper z-index, doesn't block CTAs on mobile).

## Validation
Chat widget appears on all pages as a floating button. Clicking opens the chat panel. User can type and send a message. Morgan's response appears in the chat. Chat persists across page navigations within the same session. Widget is usable on mobile without blocking page content. Error state shows reconnection message.