/// Raydium DEX program IDs on Solana mainnet.
pub fn program_ids() -> Vec<(&'static str, &'static str)> {
    vec![
        // AMM v4 (legacy constant-product)
        ("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", "raydium_amm_v4"),
        // Concentrated Liquidity Market Maker
        ("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK", "raydium_clmm"),
        // Constant Product Market Maker (newer)
        ("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C", "raydium_cpmm"),
    ]
}
