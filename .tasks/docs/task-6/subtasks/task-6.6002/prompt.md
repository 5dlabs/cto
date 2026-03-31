Implement subtask 6002: Implement rollout phase tracking with environment mapping

## Objective
Add rollout phase detection logic that reads `ENVIRONMENT` from the `hermes-infra-endpoints` ConfigMap, maps it to the rollout phase enum, and logs phase transitions as explicit events.

## Steps
Step-by-step:
1. Create `src/modules/hermes/logging/rollout-phase.ts`.
2. Implement `getRolloutPhase()` function:
   - Read `ENVIRONMENT` from `process.env` (populated via `envFrom` on the ConfigMap).
   - Map: `'dev'` → `'dev'`, `'staging'` → `'staging'`, `'production'` → check canary indicator.
   - For canary detection (pending decision point): default to checking an env var `CANARY_MODE=true/false` for v1.
   - If `CANARY_MODE=true` → `'canary'`, else → `'production'`.
   - Default to `'dev'` if `ENVIRONMENT` is undefined.
3. Cache the resolved phase in a module-level variable (it doesn't change during runtime).
4. On initialization, log a phase event: `{ operation: 'rollout_phase_initialized', rollout_phase: '...', environment: '...' }` using HermesLogger.
5. Export `getRolloutPhase()` for use by HermesLogger and other modules.
6. Integrate into HermesLogger constructor so it auto-reads rollout phase on creation.

## Validation
Test env mapping: Set `ENVIRONMENT=staging`, call `getRolloutPhase()` — returns `'staging'`. Set `ENVIRONMENT=production` + `CANARY_MODE=true` — returns `'canary'`. Set `ENVIRONMENT=production` + `CANARY_MODE=false` — returns `'production'`. Set `ENVIRONMENT` undefined — returns `'dev'`. Verify initialization log entry is emitted with correct phase value.