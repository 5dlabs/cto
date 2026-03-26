pub mod api;
pub mod backfill;
pub mod config;
pub mod db;
pub mod decoder;
pub mod dex;
pub mod error;
pub mod pool_cache;
pub mod subscriber;
pub mod writer;

pub mod proto {
    tonic::include_proto!("dex_feed");
}

/// A decoded DEX swap event, flowing from subscriber → writer.
#[derive(Debug, Clone)]
pub struct SwapEvent {
    /// Timestamp in nanoseconds since epoch.
    pub timestamp: i64,
    /// Solana slot number.
    pub slot: u64,
    /// Transaction signature (base58).
    pub signature: String,
    /// DEX label (e.g. "raydium_amm_v4", "orca_whirlpool").
    pub dex: String,
    /// Pool address (base58) or "unknown".
    pub pool: String,
    /// Mint address of the token sold (base58).
    pub token_in: String,
    /// Mint address of the token bought (base58).
    pub token_out: String,
    /// Decimal-adjusted amount sold.
    pub amount_in: f64,
    /// Decimal-adjusted amount bought.
    pub amount_out: f64,
    /// Price = amount_out / amount_in.
    pub price: f64,
    /// Transaction signer (base58).
    pub signer: String,
}
