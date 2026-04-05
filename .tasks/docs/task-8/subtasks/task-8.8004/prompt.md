Implement subtask 8004: Embed Morgan web chat widget

## Objective
Integrate the Morgan AI web chat widget into the Sigma-1 website, connecting to the Morgan agent's WebSocket endpoint for real-time conversational UI with message history, typing indicators, and minimizable chat window.

## Steps
Step 1: Create a chat widget React component: floating button in the bottom-right corner that expands to a chat window. Step 2: Implement WebSocket connection to Morgan's web chat endpoint — handle connection lifecycle (connect, reconnect on disconnect, heartbeat). Step 3: Implement the chat UI: message list with user/agent bubbles, timestamp display, typing indicator animation, and auto-scroll to latest message. Step 4: Implement message input: text field with send button, enter-to-send, and disabled state during agent processing. Step 5: Implement session management: store session token in localStorage, resume conversations on page revisit. Step 6: Implement minimize/maximize toggle with unread message badge. Step 7: Make the widget available globally across all pages via the root layout. Step 8: Ensure the widget is responsive and doesn't obstruct page content on mobile.

## Validation
Chat widget appears on all pages; clicking the button opens the chat window; messages are sent and received via WebSocket in real-time; typing indicator shows during agent processing; session persists across page navigation; widget is usable on mobile viewports; unread badge appears when minimized.