## Update Web Experience for Hermes Path Surfacing (Blaze - React/Next.js)

### Objective
Build the frontend web experience that surfaces Hermes deliberation results, displays snapshot artifacts (current-site screenshots and variant snapshots) in a comparison view, and implements the environment distinction pattern (banner + accent color) for staging vs production.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: high
- Status: pending
- Dependencies: 1, 2, 3

### Implementation Details
Step-by-step implementation:

1. **Environment distinction (per D4):**
   - Create `components/EnvironmentBanner.tsx` — persistent top bar component:
     - Reads `NEXT_PUBLIC_ENVIRONMENT` env var (injected at build time)
     - Renders `⚠ STAGING` banner with amber background when env !== 'production'
     - Hidden in production (or shows subtle production indicator)
   - CSS custom property theming: define `--accent-color` in `globals.css` or Tailwind config
     - Staging: `--accent-color: theme(colors.amber.500)`
     - Production: `--accent-color: theme(colors.brand.500)` (or primary brand color)
   - Add `<EnvironmentBanner />` to root layout (`app/layout.tsx`)

2. **shadcn/ui integration (per D9):**
   - Initialize shadcn/ui: `npx shadcn-ui@latest init` — select Tailwind CSS, configure `components.json`
   - Install needed components into `components/ui/`: Card, Badge, Tabs, Dialog, Skeleton, Button, Table
   - All components are copied into the codebase — no runtime dependency

3. **Hermes deliberation dashboard:** Create `app/hermes/page.tsx`:
   - List view of deliberations using shadcn Card components
   - Each card shows: deliberation ID, status (Badge with color coding), triggered by, timestamp, artifact count
   - Pagination via query params
   - Loading states via Skeleton components

4. **Deliberation detail page:** Create `app/hermes/[id]/page.tsx`:
   - Header: deliberation metadata (status, trigger time, duration)
   - Status indicator: real-time or polling-based status updates during processing
   - Artifact section: tabbed view (Tabs component) — "Current Site" and "Variants"

5. **Snapshot comparison view:** Create `components/hermes/ArtifactComparison.tsx`:
   - Side-by-side comparison: current-site screenshot on left, selected variant on right
   - Image loading from presigned URLs (`GET /api/hermes/artifacts/:id/url`)
   - Variant selector: thumbnail strip below the comparison area
   - Zoom/pan capability for detail inspection (use CSS transform or a lightweight image viewer)

6. **Artifact viewer:** Create `components/hermes/ArtifactViewer.tsx`:
   - Full-screen dialog (Dialog component) for individual artifact viewing
   - Download button for artifact PNG
   - Metadata display: viewport, URL captured, timestamp

7. **Feature flag:** Gate all Hermes UI behind `NEXT_PUBLIC_HERMES_ENABLED` env var:
   - When false: Hermes nav item hidden, `/hermes` routes return 404 or redirect
   - Feature flag respects environment awareness (can be enabled in staging but disabled in production)

8. **API client:** Create `lib/hermes-api.ts`:
   - Typed API client for all Hermes endpoints using fetch or SWR
   - Types imported from or mirroring the backend's `types.ts` definitions
   - Error handling with user-friendly toast notifications

9. **Navigation:** Add "Hermes" item to the main navigation (conditionally rendered based on feature flag and RBAC claims from session).

10. **Accessibility:** Ensure all new components meet WCAG 2.1 AA:
    - Environment banner: text label (not color alone) per D4
    - Image alt text on all screenshots
    - Keyboard navigation for comparison view and artifact viewer
    - Focus management in Dialog components (handled by Radix)

### Subtasks
- [ ] Implement EnvironmentBanner component with CSS custom property theming: Create `components/EnvironmentBanner.tsx` that reads `NEXT_PUBLIC_ENVIRONMENT` and renders a staging/production-aware banner, plus wire up CSS custom property theming for `--accent-color` across environments.
- [ ] Initialize shadcn/ui and install required components: Run `shadcn-ui init` to set up the component system with Tailwind CSS, then install all needed components (Card, Badge, Tabs, Dialog, Skeleton, Button, Table) into `components/ui/`.
- [ ] Create typed Hermes API client: Create `lib/hermes-api.ts` — a typed API client for all Hermes backend endpoints, with SWR hooks for data fetching, error handling, and types mirroring the backend definitions.
- [ ] Build Hermes deliberation dashboard page: Create `app/hermes/page.tsx` — the deliberation list view with Card components showing deliberation details, status badges, pagination, and loading skeleton states.
- [ ] Build deliberation detail page with status updates and tabbed artifacts: Create `app/hermes/[id]/page.tsx` — the deliberation detail view showing metadata, polling-based status updates, and a tabbed artifact section for current-site screenshots and variant snapshots.
- [ ] Build snapshot comparison view component: Create `components/hermes/ArtifactComparison.tsx` — a side-by-side image comparison component showing the current-site screenshot alongside a selected variant, with a thumbnail variant selector strip.
- [ ] Build full-screen artifact viewer dialog component: Create `components/hermes/ArtifactViewer.tsx` — a full-screen Dialog component for viewing individual artifact images with download capability and metadata display.
- [ ] Implement feature flag gating and navigation integration: Gate all Hermes UI behind the `NEXT_PUBLIC_HERMES_ENABLED` feature flag, add conditional Hermes navigation item, and implement route-level protection for `/hermes` paths.
- [ ] Accessibility audit and remediation for all Hermes components: Run axe-core accessibility audits on all new Hermes pages and components, then fix any critical or serious violations to meet WCAG 2.1 AA compliance.