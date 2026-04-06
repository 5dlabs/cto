Implement subtask 8004: Embed Morgan web chat widget with real-time WebSocket communication

## Objective
Integrate the Morgan AI chat widget into the website as a floating component that communicates with the Morgan agent's WebSocket endpoint in real-time.

## Steps
1. Create a ChatWidget React component that renders as a floating button in the bottom-right corner of every page.
2. On click, expand to a chat panel with message history, input field, and send button.
3. Establish a WebSocket connection to Morgan's /ws/chat endpoint on widget open.
4. Implement the message protocol: send user messages as JSON, receive agent responses, render typing indicators.
5. Display messages in a scrollable chat window with distinct styling for user vs. agent messages.
6. Handle connection lifecycle: reconnect on disconnect, show connection status indicator.
7. Implement session management: store session ID in sessionStorage for continuity within a browser session.
8. Add the ChatWidget to the root layout so it appears on all pages.
9. Implement fallback to HTTP polling if WebSocket connection fails.
10. Ensure the widget is responsive and works on mobile devices.

## Validation
Chat widget appears on all pages; clicking opens the chat panel; sending a message receives a response from Morgan; typing indicator displays during agent processing; reconnection works after network interruption; widget is usable on mobile viewport.