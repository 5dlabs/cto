Implement subtask 8003: Build root layout with navigation, footer, responsive shell, and TanStack Query provider

## Objective
Implement the root App Router layout with site-wide navigation header, footer, responsive mobile menu, TanStack Query client provider with Effect integration layer, and error boundary setup.

## Steps
1. Create `app/layout.tsx` as the root layout:
   - Import Inter font via `next/font/google`.
   - Wrap children in TanStack QueryClientProvider with a custom query client.
   - Include a top-level error boundary component.
2. Configure TanStack Query client in `lib/query-client.ts`:
   - Default stale times: 1 min for catalog queries, 5 min for category queries.
   - Default error handler that logs and toasts.
   - Create an Effect-to-TanStack-Query adapter in `lib/effect-query.ts` that wraps Effect.runPromise for use in queryFn.
3. Build `components/sigma1/header.tsx`:
   - Logo (left), nav links: Home, Equipment, Quote Builder, Portfolio (center/right).
   - Mobile: hamburger menu opening a Sheet (shadcn/ui) with nav links.
   - Active link highlighting based on current pathname.
4. Build `components/sigma1/footer.tsx`:
   - Company info, contact details, social links, copyright.
   - Links to /llms.txt.
5. Ensure the layout is fully responsive: mobile-first with breakpoints for tablet (md) and desktop (lg).
6. The layout must reserve space/portal for the chat widget (implemented separately) — add a div with id `chat-widget-root` at layout level.

## Validation
Render the layout at mobile (375px), tablet (768px), and desktop (1280px) widths and verify navigation collapses to hamburger on mobile. Verify TanStack QueryClientProvider is present by checking React DevTools. Verify the `chat-widget-root` div is present in the DOM.