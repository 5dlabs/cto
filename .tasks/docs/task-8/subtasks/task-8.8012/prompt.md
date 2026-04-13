<identity>
You are blaze working on subtask 8012 of task 8.
</identity>

<context>
<scope>
Add the Morgan chat widget JavaScript to the root layout using next/script with afterInteractive strategy, sourcing the URL from the NEXT_PUBLIC_MORGAN_WIDGET_URL environment variable.
</scope>
</context>

<implementation_plan>
1. In app/layout.tsx, import Script from 'next/script'.
2. Add after the main content:
   ```tsx
   <Script
     src={process.env.NEXT_PUBLIC_MORGAN_WIDGET_URL + '/widget.js'}
     strategy='afterInteractive'
     data-agent='morgan'
   />
   ```
3. The widget.js (served by Morgan via Cloudflare Tunnel) is expected to self-initialize and render a floating chat button in the bottom-right corner.
4. Add NEXT_PUBLIC_MORGAN_WIDGET_URL to .env.local.example and document in apps/website/README.md.
5. Add a defensive check: if NEXT_PUBLIC_MORGAN_WIDGET_URL is empty (e.g., in test environment), do not render the Script tag to avoid 404 errors breaking page load.
6. In the Content Security Policy (if configured in next.config.ts), add the Morgan tunnel domain to script-src and connect-src.
</implementation_plan>

<validation>
Playwright (with NEXT_PUBLIC_MORGAN_WIDGET_URL set to mock URL): GET / — Script tag with correct src attribute is present in DOM. With a running Morgan instance: floating chat button renders in bottom-right, clicking it opens the slide-up chat panel. GET / in test environment with empty NEXT_PUBLIC_MORGAN_WIDGET_URL — no Script tag rendered, no console errors.
</validation>