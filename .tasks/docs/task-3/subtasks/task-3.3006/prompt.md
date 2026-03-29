Implement subtask 3006: Add secret rotation annotations and document rotation procedure

## Objective
Annotate existing secrets for compatibility with external-secrets-operator or sealed-secrets, and create documentation for the secret rotation procedure.

## Steps
1. Update `infra/notifycore/templates/` secret templates to include annotations:
   - For external-secrets-operator compatibility: add `external-secrets.io/` annotations (if ESO approach is chosen).
   - For sealed-secrets compatibility: document how to create SealedSecret equivalents.
   - Add labels for secret management: `app.kubernetes.io/managed-by: helm`, `notifycore.io/secret-type: credentials`.
2. Add rotation-related annotations: `notifycore.io/rotation-period: 90d`, `notifycore.io/last-rotated: {{ now | date "2006-01-02" }}`.
3. Create `docs/secret-rotation.md` documenting:
   - How to rotate PostgreSQL credentials (CloudNativePG supports rolling secret updates).
   - How to rotate Redis password (update secret, rolling restart of Redis and app pods).
   - Verification steps after rotation.
   - Rollback procedure if rotation fails.
4. Values file should include `secretManagement.provider: none|external-secrets|sealed-secrets` for conditional annotation rendering.

## Validation
Secret templates include rotation annotations. `docs/secret-rotation.md` exists and covers PostgreSQL rotation, Redis rotation, verification, and rollback sections. `helm template` renders secrets with appropriate annotations based on values.