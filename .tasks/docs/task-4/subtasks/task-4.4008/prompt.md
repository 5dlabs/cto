Implement subtask 4008: Implement feature flag gating and navigation integration

## Objective
Gate all Hermes UI behind the `NEXT_PUBLIC_HERMES_ENABLED` feature flag, add conditional Hermes navigation item, and implement route-level protection for `/hermes` paths.

## Steps
1. Create `lib/feature-flags.ts`:
   - Export `isHermesEnabled(): boolean` — reads `NEXT_PUBLIC_HERMES_ENABLED` env var, returns true only when value is `'true'`
   - This can be used both at build time (server components) and client side
2. Route-level gating:
   - Create `app/hermes/layout.tsx` that checks `isHermesEnabled()`
   - When disabled: redirect to home page (`redirect('/')`) or render a 404-style page
   - When enabled: render children normally
3. Navigation integration:
   - Locate the main navigation component (sidebar, header nav, etc.)
   - Add a "Hermes" nav item conditionally: `{isHermesEnabled() && <NavItem href="/hermes" icon={...}>Hermes</NavItem>}`
   - Additionally gate on RBAC: only show if the user session has `hermes:read` claim (read from auth context/session)
4. Environment-aware flags: document that `NEXT_PUBLIC_HERMES_ENABLED` can be set to `true` in staging and `false` in production `.env` files.
5. Ensure that direct URL access to `/hermes/*` is also gated (handled by the layout redirect).

## Validation
Feature flag off: set `NEXT_PUBLIC_HERMES_ENABLED=false`, render the navigation, verify no 'Hermes' nav item is in the DOM. Navigate to `/hermes`, verify redirect to home or 404 rendering. Feature flag on: set `NEXT_PUBLIC_HERMES_ENABLED=true`, verify 'Hermes' nav item is present and `/hermes` renders the dashboard. RBAC test: with flag enabled but without `hermes:read` claim in session, verify nav item is hidden.