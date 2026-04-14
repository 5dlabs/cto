use dashmap::DashMap;

/// Concurrent cache mapping pool address → (token_a_mint, token_b_mint).
///
/// Populated lazily from observed swap transactions.  Can be extended later
/// to decode on-chain pool account layouts for pre-population.
pub struct PoolCache {
    cache: DashMap<String, (String, String)>,
}

impl PoolCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Record a pool's token pair (idempotent).
    pub fn insert(&self, pool: &str, token_a: &str, token_b: &str) {
        self.cache
            .entry(pool.to_string())
            .or_insert_with(|| (token_a.to_string(), token_b.to_string()));
    }

    /// Look up a pool's token pair.
    pub fn get(&self, pool: &str) -> Option<(String, String)> {
        self.cache.get(pool).map(|r| r.value().clone())
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for PoolCache {
    fn default() -> Self {
        Self::new()
    }
}
