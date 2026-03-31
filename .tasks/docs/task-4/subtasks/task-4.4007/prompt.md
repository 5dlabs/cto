Implement subtask 4007: Build full-screen artifact viewer dialog component

## Objective
Create `components/hermes/ArtifactViewer.tsx` — a full-screen Dialog component for viewing individual artifact images with download capability and metadata display.

## Steps
1. Create `components/hermes/ArtifactViewer.tsx`:
   - Props: `artifact: Artifact`, `open: boolean`, `onOpenChange: (open: boolean) => void`
   - Uses shadcn Dialog component for the full-screen overlay
2. Dialog content:
   - Full-resolution image loaded from presigned URL (fetched on open)
   - Image fits to viewport with maintain aspect ratio
   - Zoom/pan controls (same CSS transform approach as comparison view, or simplified)
3. Download button:
   - Fetches the presigned URL and triggers a browser download
   - Filename: `{artifact_type}_{artifact_id}.png`
   - Use `<a download>` with blob URL approach for reliable cross-browser download
4. Metadata display panel (collapsible sidebar or bottom panel):
   - Viewport dimensions from `artifact.metadata.viewport`
   - Source URL from `artifact.metadata.url`
   - Capture timestamp from `artifact.createdAt`
   - File size formatted (e.g., '1.2 MB')
   - Capture duration from `artifact.metadata.durationMs`
5. Keyboard shortcuts: Escape to close (handled by Radix Dialog), arrow keys for prev/next if used in a gallery context.
6. Focus management: focus trapped within dialog (handled by Radix Dialog primitive).

## Validation
Component test: render ArtifactViewer with `open=true` and a mock artifact. Verify the Dialog is visible, image renders, metadata fields (viewport, URL, timestamp, size) are displayed. Download test: click download button, verify a download is triggered (mock `URL.createObjectURL`). Close test: click the close button or press Escape, verify `onOpenChange` is called with `false`. Accessibility test: verify focus is trapped inside the dialog when open.