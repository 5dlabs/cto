## Decision Points

- Legacy storage location: The actual source of legacy snapshot artifacts is unspecified — it could be a filesystem path, an existing MinIO bucket under GitLab, or database BLOBs. This must be confirmed before implementing the legacy-scanner, as the discovery and streaming logic differs significantly for each.
- Migration tracker persistence: Should the migration tracker state be stored in the application database, in a dedicated MinIO metadata object, or in Redis? Database gives transactional guarantees with artifact metadata writes; MinIO keeps it self-contained; Redis is simple but volatile.

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia