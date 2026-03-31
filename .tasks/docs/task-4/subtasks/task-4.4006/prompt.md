Implement subtask 4006: Build snapshot comparison view component

## Objective
Create `components/hermes/ArtifactComparison.tsx` — a side-by-side image comparison component showing the current-site screenshot alongside a selected variant, with a thumbnail variant selector strip.

## Steps
1. Create `components/hermes/ArtifactComparison.tsx`:
   - Props: `currentSiteArtifact: Artifact`, `variantArtifacts: Artifact[]`
   - Layout: two-column side-by-side view — left column for current site, right column for selected variant
2. Image loading:
   - For each artifact, call `fetchArtifactUrl(artifact.id)` to get the presigned URL
   - Use `<Image>` from Next.js or a plain `<img>` with the presigned URL as `src`
   - Show Skeleton loader while presigned URL is being fetched and while image is loading
   - Handle expired presigned URLs gracefully — re-fetch if image load fails
3. Variant selector:
   - Thumbnail strip below the comparison area showing all variant artifacts as small thumbnails
   - Clicking a thumbnail selects that variant and displays it in the right column
   - Visual indicator on the currently selected thumbnail (border highlight)
4. Zoom/pan capability:
   - Implement CSS transform-based zoom on mouse wheel / pinch gesture
   - Click-and-drag to pan when zoomed in
   - Sync zoom level between left and right panels for easier comparison
   - Reset zoom button
5. Labels: display "Current Site" and "Variant {n}" labels above each image.
6. Responsive: stack vertically on mobile screens (`md:` breakpoint for side-by-side).

## Validation
Component test: render with a mock current-site artifact and 2 variant artifacts (mock presigned URL fetches). Verify both image panels render, thumbnail strip shows 2 thumbnails. Click the second thumbnail, verify the right panel image src changes. Verify 'Current Site' and variant labels are present. Responsive test: render at mobile width, verify vertical stacking. Zoom test: simulate wheel event on an image panel, verify CSS transform scale changes.