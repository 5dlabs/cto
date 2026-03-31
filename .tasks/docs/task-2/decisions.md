## Decision Points

- Artifact storage schema: use a parallel `hermes_artifacts` table with FK to deliberations, or extend the existing artifacts schema with a polymorphic association? This depends on the D6 decision and affects the IHermesArtifactWriter abstraction boundary.
- Pagination strategy: cursor-based vs offset-based pagination for GET /api/hermes/deliberations list endpoint? Cursor-based is more scalable but offset-based is simpler for initial UI integration.

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia