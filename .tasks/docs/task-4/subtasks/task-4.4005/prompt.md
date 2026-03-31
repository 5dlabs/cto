Implement subtask 4005: Build deliberation detail page with status updates and tabbed artifacts

## Objective
Create `app/hermes/[id]/page.tsx` — the deliberation detail view showing metadata, polling-based status updates, and a tabbed artifact section for current-site screenshots and variant snapshots.

## Steps
1. Create `app/hermes/[id]/page.tsx` as a client component.
2. Use `useDeliberation(id)` hook — configure SWR to poll every 3 seconds when status is `pending` or `processing`, stop polling when `completed` or `failed`.
3. Header section:
   - Deliberation ID with copy button
   - Status Badge (same color coding as dashboard)
   - Processing time indicator (elapsed time if in progress, total duration if completed)
   - Target URL (linked)
   - Triggered by and timestamp
4. Artifact section using shadcn Tabs:
   - Tab 1: "Current Site" — shows the current-site screenshot artifact
   - Tab 2: "Variants" — shows all variant snapshot artifacts
   - Use `useDeliberationArtifacts(id)` to fetch artifacts, then filter by `artifactType`
5. Each tab renders artifact cards with thumbnail previews loaded from presigned URLs.
6. Loading state: Skeleton components for the entire page while initial data loads.
7. Error/failed state: if deliberation status is `failed`, show a prominent error message with any available error details from metadata.
8. Back navigation: breadcrumb or back link to `/hermes`.

## Validation
Component test: mock a completed deliberation with 1 current-site and 2 variant artifacts. Verify header shows correct metadata, Tabs component has 2 tabs, 'Current Site' tab shows 1 artifact, 'Variants' tab shows 2 artifacts. Polling test: mock a 'processing' deliberation, verify SWR refetch interval is set (check `useSWR` config). Status transition test: start with 'processing' mock, update to 'completed', verify status badge updates and polling stops.