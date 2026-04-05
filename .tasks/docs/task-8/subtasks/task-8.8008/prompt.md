Implement subtask 8008: Embed Morgan web chat widget and connect to agent WebSocket endpoint

## Objective
Implement the Morgan AI chat widget component that connects to the Morgan agent's web chat endpoint, providing real-time conversational AI support across all pages.

## Steps
1. Create a ChatWidget React component (src/components/chat/ChatWidget.tsx).
2. Implement WebSocket connection to the Morgan agent's /ws/chat endpoint.
3. Implement chat UI: message list (user + agent messages), text input, send button, typing indicator.
4. Handle connection lifecycle: connect on widget open, reconnect on disconnect, graceful close.
5. Implement session management: generate/persist sessionId in localStorage.
6. Style the widget with shadcn/ui components and TailwindCSS to match Sigma-1 branding.
7. Implement the widget trigger: floating button in bottom-right corner (or per dp-11 decision).
8. Add open/close animation and minimize/maximize states.
9. Render the widget in the root layout so it's available on all pages.
10. Handle edge cases: agent offline message, message send failure, long messages.

## Validation
Chat widget renders on all pages; clicking the trigger opens the widget; sending a message establishes WebSocket connection and receives a response; conversation history persists within a session; widget gracefully handles agent unavailability; widget is keyboard accessible and screen-reader friendly.