Implement subtask 2011: Implement Valkey caching layer for categories and products

## Objective
Add Valkey-based caching for category list (5-minute TTL) and product detail (1-minute TTL with invalidation on update), keeping availability queries uncached.

## Steps
1. Create `cache.rs` module with generic cache helpers.
2. Implement `async fn get_or_set<T: Serialize + DeserializeOwned>(valkey: &Pool, key: &str, ttl_secs: u64, fetch: impl Future<Output=Result<T>>) -> Result<T>` — check Valkey for key, return deserialized value if present, otherwise call fetch, serialize to JSON, SET with EX ttl, return value.
3. Category caching: wrap the categories query in the GET /categories handler with `get_or_set("catalog:categories", 300, ...)`. 
4. Product detail caching: wrap GET /products/:id query with `get_or_set(format!("catalog:product:{id}"), 60, ...)`.
5. Cache invalidation on PATCH /products/:id: after successful update, DEL `catalog:product:{id}` from Valkey.
6. Cache invalidation on POST /products: DEL `catalog:categories` key (since product counts change).
7. Availability queries: explicitly document and verify these bypass the cache entirely.
8. Add cache hit/miss metrics using shared-observability (counter: `cache_hits_total`, `cache_misses_total` with label `cache_name`).

## Validation
Integration tests: (1) GET /categories twice, verify second request is faster (cache hit). (2) Verify Valkey contains `catalog:categories` key after first GET. (3) GET /products/:id, verify product cached in Valkey. (4) PATCH product, verify cache key deleted. (5) Verify availability endpoint does NOT set any cache keys. (6) Verify cache_hits_total metric increments on cache hit via /metrics endpoint.