Implement subtask 8007: Embed Morgan web chat widget

## Objective
Integrate the Morgan AI web chat widget into the website as a persistent floating component that connects to Morgan's WebSocket endpoint for real-time customer conversations.

## Steps
1. Create a `ChatWidget` client component (`'use client'`) that renders a floating chat bubble in the bottom-right corner.
2. On click, expand to a chat panel with message history, input field, and send button.
3. Establish a WebSocket connection to Morgan's web chat endpoint (URL from environment variable).
4. Implement the message protocol: send JSON messages with text and session ID; receive and display agent responses.
5. Implement session persistence: store session ID in localStorage so returning visitors resume conversations.
6. Show typing indicator while waiting for agent response.
7. Handle connection errors and reconnection logic.
8. Add the ChatWidget to the root layout so it appears on all pages.
9. Ensure the widget is accessible: keyboard navigable, aria labels, screen reader announcements for new messages.

## Validation
Chat bubble appears on all pages; clicking opens the chat panel; sending a message establishes WebSocket connection and delivers the message; agent response appears in the chat; session persists across page navigations; typing indicator shows during agent processing; widget is keyboard accessible; connection loss shows error state and attempts reconnection.