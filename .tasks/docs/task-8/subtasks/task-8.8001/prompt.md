Implement subtask 8001: Create SnapshotPR component with shadcn/ui Card and status badges

## Objective
Build a SnapshotPR component using shadcn/ui Card that displays PR title, clickable URL with external link icon, color-coded status badge (open=green, merged=purple, closed=red), file count, and branch name.

## Steps
1. Create `components/snapshot-pr.tsx`. 2. Accept props: `prResult: { title: string; url: string; status: 'open' | 'merged' | 'closed'; files_changed: number; branch: string } | null`. 3. When prResult is non-null, render a shadcn/ui Card with: CardHeader containing PR title, CardContent with branch name, file count, and status Badge. 4. Status badge color mapping: 'open' → green variant, 'merged' → purple/violet variant, 'closed' → red/destructive variant. Use shadcn/ui Badge with className overrides for colors. 5. PR URL rendered as an anchor tag with `target='_blank'` and `rel='noopener noreferrer'`. 6. Add an ExternalLink icon (from lucide-react) next to the URL text. 7. When prResult is null, render a muted message: 'No snapshot PR created' with optional reason text if a `reason` prop is provided. 8. Add `aria-label` on the link indicating it opens externally for screen reader accessibility.

## Validation
Render SnapshotPR with valid prResult having status='open'; verify Card renders with PR title, green badge with text 'Open', clickable URL, file count, and branch name. Render with status='merged'; verify purple badge. Render with null; verify 'No snapshot PR created' message.