Implement subtask 9007: Build Morgan Chat tab with WebSocket connection and message history

## Objective
Implement the dedicated Morgan chat screen with full-screen WebSocket-based conversation, message history persistence in AsyncStorage, and support for rich messages (interactive product cards).

## Steps
1. Create `lib/chat/websocket.ts`: WebSocket client connecting to Morgan agent endpoint (same URL as web frontend). Handle connection lifecycle: connect, reconnect on drop (exponential backoff), disconnect on screen unmount or app background.
2. Build `screens/chat/ChatScreen.tsx`: Full-screen layout with message list (`FlatList` inverted), text input bar with send button, and connection status indicator.
3. Render messages using `ChatBubble` component. Differentiate user messages (right-aligned) from Morgan messages (left-aligned, with avatar).
4. **Rich messages**: Parse Morgan responses for embedded product references. Render interactive `ProductCard` within the chat that the user can tap to navigate to Equipment product detail.
5. **Message history**: On conversation load, fetch message history from server API. Persist messages locally in AsyncStorage keyed by conversation ID. On reconnect, sync delta from server.
6. **Photo sending**: Integrate `expo-image-picker` for camera/gallery photo selection. Upload photo to social engine endpoint. Send photo message reference through WebSocket. Display photo thumbnails in chat.
7. Implement typing indicator when Morgan is processing.
8. Auto-scroll to latest message on new incoming message. Manual scroll-up should disable auto-scroll until user scrolls back to bottom.

## Validation
Mock WebSocket: send a user message, verify it appears in FlatList as sent bubble. Mock incoming Morgan message, verify it appears as received bubble with avatar. Rich message test: mock a message containing product reference, verify ProductCard renders inline. Photo test: mock image picker selection, verify upload API called and photo thumbnail appears in chat. History test: mock API returning prior messages, verify they render on screen load.