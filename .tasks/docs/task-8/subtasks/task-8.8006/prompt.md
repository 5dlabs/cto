Implement subtask 8006: Implement Morgan web chat widget integration

## Objective
Build and embed the Morgan web chat widget that connects to the Morgan agent's WebSocket endpoint, supporting real-time conversational interaction.

## Steps
1. Create a ChatWidget React component (floating button + expandable chat panel, or full-page per dp-11 decision).
2. Implement WebSocket connection to Morgan agent's /ws/chat endpoint.
3. Handle connection lifecycle: connect on widget open, reconnect on disconnect, close on widget close.
4. Implement message UI: scrollable message list, user messages (right-aligned), agent messages (left-aligned), typing indicator during agent response.
5. Stream agent responses token-by-token for real-time feel.
6. Generate and persist session_id in localStorage for conversation continuity.
7. Add the widget to the root layout so it appears on all pages (or create a /chat route for full-page).
8. Style consistently with the site's design system (shadcn/ui components, Tailwind).
9. Handle error states: connection failure, agent timeout.

## Validation
Chat widget appears on all pages (or /chat route is accessible). Clicking opens the chat interface. Sending a message connects via WebSocket and receives a response from Morgan. Messages display correctly with sender alignment. Session persists across page navigations. Connection failure shows a user-friendly error.