Implement subtask 5010: Implement Valkey caching layer for vetting results

## Objective
Add a caching layer using Valkey (Redis-compatible) to cache vetting results for 24 hours per org_id, avoiding redundant external API calls.

## Steps
1. Create `src/cache/mod.rs`.
2. Add `redis` crate dependency (or `fred` for async Redis) to Cargo.toml.
3. Define `VettingCache` struct with a Redis connection manager.
4. Implement:
   - `pub async fn get_cached_result(&self, org_id: &Uuid) -> Result<Option<VettingResult>, VettingError>` — key: `vetting:result:{org_id}`, deserialize from JSON.
   - `pub async fn set_cached_result(&self, org_id: &Uuid, result: &VettingResult) -> Result<(), VettingError>` — serialize to JSON, SET with EX 86400 (24h TTL).
   - `pub async fn invalidate(&self, org_id: &Uuid) -> Result<(), VettingError>` — DEL key.
5. Integrate into pipeline: after computing result, call `set_cached_result`.
6. Integrate into GET endpoint: check cache first, return if hit, otherwise query DB.
7. Integrate into DELETE (GDPR) endpoint: call `invalidate` after DB deletion.
8. Ensure VettingResult implements Serialize/Deserialize for JSON cache storage.
9. Connection URL read from `sigma1-infra-endpoints` ConfigMap (VALKEY_URL env var).

## Validation
Integration test: set_cached_result then get_cached_result returns the same data. Test: TTL is set to 86400. Test: invalidate removes the key, subsequent get returns None. Test: GET endpoint returns cached result on second call (mock external APIs verify zero additional calls).