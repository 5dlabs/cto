## Decision Points

- Should the design PRs surface as a new route (`/dashboard/design-prs`) or as a tab/section within an existing dashboard page? This depends on the current app's navigation pattern and information architecture.
- What is the confirmed API contract shape for `GET /api/pipeline/design-prs` and the detail endpoint — specifically the response schema for PR metadata and scaffold file listings? A mismatch will require rework.

## Coordination Notes

- Agent owner: blaze
- Primary stack: React/Next.js