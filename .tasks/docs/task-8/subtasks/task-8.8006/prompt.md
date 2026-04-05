Implement subtask 8006: Embed Morgan web chat widget on the website

## Objective
Integrate the Morgan AI web chat widget into the website, connecting it to the Morgan agent's WebSocket/HTTP chat endpoint.

## Steps
1. Create a `ChatWidget` React component that renders a floating chat bubble and expandable chat panel.
2. Implement WebSocket or HTTP streaming connection to the Morgan web chat endpoint (defined in task 7009's communication contract).
3. Build the chat UI: message list (user and Morgan messages with timestamps), text input with send button, typing indicator, and connection status.
4. Implement session management: generate or restore a session ID, maintain conversation history in local state.
5. Add open/close/minimize states for the widget.
6. Place the widget in the root layout so it's available on all pages.
7. Ensure the widget is responsive and doesn't interfere with page content or navigation.
8. Style the widget to match the site's design system (shadcn/ui + TailwindCSS tokens).

## Validation
Chat widget appears on all pages as a floating button; clicking opens the chat panel; messages can be sent and Morgan responses are displayed; typing indicator appears while waiting for response; widget can be minimized and reopened with conversation preserved; widget is responsive and doesn't overlap critical page elements.