/// Jupiter aggregator program IDs on Solana mainnet.
///
/// Jupiter routes through underlying DEXes.  When a transaction includes both
/// a Jupiter program and a specific DEX program, the registry prefers the
/// underlying DEX so that price data is attributed to the actual pool.
pub fn program_ids() -> Vec<(&'static str, &'static str)> {
    vec![
        // v6 aggregator
        ("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", "jupiter_v6"),
    ]
}
