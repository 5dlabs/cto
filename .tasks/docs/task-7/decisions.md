## Decision Points

- The auth endpoint `POST /api/auth/login` may not exist on the PM server yet — decide whether to add it to cto-pm (Task 2 scope expansion) or implement a stub/mock auth for v1.
- JWT refresh mechanism: should the frontend use a dedicated `/api/auth/refresh` endpoint with a separate refresh token, or silently re-authenticate using stored credentials? This affects both security posture and backend requirements.
- Data fetching strategy: SWR vs. React Query (TanStack Query) vs. raw `setInterval` + `fetch`. SWR is mentioned in the details but this choice affects caching, deduplication, and error retry behavior.

## Coordination Notes

- Agent owner: blaze
- Primary stack: React/Next.js