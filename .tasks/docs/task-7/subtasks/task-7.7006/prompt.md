<identity>
You are blaze working on subtask 7006 of task 7.
</identity>

<context>
<scope>
Implement the main page split-panel layout with 60/40 width ratio for settlement feed and customer balance areas, responsive stacking on mobile, optimized for 1920×1080 screen recording, and a 'Not Connected' fallback state.
</scope>
</context>

<implementation_plan>
1. Create or update `src/app/page.tsx` as a client component.
2. Use a flex row layout at desktop (min-width: 1024px) with left panel at 60% width and right panel at 40% width. Add a gap between panels.
3. On mobile/tablet (<1024px), switch to flex-col so panels stack vertically.
4. Left panel: a container div with placeholder text 'Settlement Feed' and a subtle border or card background (bg-solana-card rounded-lg). This will be replaced by Task 8's SettlementFeed component.
5. Right panel: a container div with placeholder text 'Customer Balance' styled similarly. This will be replaced by Task 8's CustomerBalance component.
6. Optimize minimum heights for 1920×1080: the content area (below header) should fill the viewport height using min-h-[calc(100vh-headerHeight)] or similar.
7. Use the `useWallet()` hook to check connection state. If not connected, render the 'Not Connected' state instead of the split panels.
8. 'Not Connected' state: centered content area with CTO branding/logo, a brief description of the settlement protocol, and a prominent 'Connect Wallet' button (which triggers the wallet modal via `useWalletModal().setVisible(true)`).
9. Add smooth transitions between connected/disconnected states if possible.
10. Include the Header component at the top of the page or confirm it's rendered from layout.
</implementation_plan>

<validation>
Verify at http://localhost:3000 on 1920×1080 viewport: (1) Without wallet connected, the 'Not Connected' state shows CTO branding and connect prompt. (2) Clicking the connect button opens the wallet modal. (3) After connecting, the split-panel layout appears with 60/40 ratio — measure panel widths to confirm approximately 60% and 40%. (4) On 375px viewport, panels stack vertically. (5) Content area fills viewport height without scrollbar when panels have minimal content. (6) Placeholder text is visible in both panels.
</validation>