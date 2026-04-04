Implement subtask 7003: Add relative timestamp formatting for memo timestamp

## Objective
Format the research memo timestamp as a human-readable relative time string (e.g., '2 hours ago') using a lightweight date utility.

## Steps
1. Use `date-fns/formatDistanceToNow` (or similar from an already-installed date library) to convert the ISO timestamp string to relative time. 2. In the ResearchMemo component, display the formatted timestamp below the markdown content as a muted metadata line, e.g., `<p className='text-xs text-muted-foreground'>2 hours ago</p>`. 3. Also display the `source` field on the same metadata line or adjacent to it, e.g., 'Source: hermes · 2 hours ago'. 4. Handle edge cases: invalid timestamp string should fall back to displaying the raw string.

## Validation
Component test: pass timestamp '2024-01-15T10:30:00Z' and mock Date.now to a known value; verify the rendered text contains a relative time string like '3 months ago'. Test with an invalid timestamp string; verify it falls back gracefully.