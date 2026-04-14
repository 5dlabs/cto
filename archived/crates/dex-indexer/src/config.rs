use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(name = "dex-indexer", about = "Solana DEX swap indexer → QuestDB")]
pub struct Config {
    /// Yellowstone gRPC endpoint URL.
    #[arg(
        long,
        env = "GRPC_URL",
        default_value = "http://agave-rpc-grpc.solana.svc:10000"
    )]
    pub grpc_url: String,

    /// QuestDB HTTP endpoint (ILP-over-HTTP).
    #[arg(
        long,
        env = "QUESTDB_URL",
        default_value = "http://questdb.questdb.svc:9000"
    )]
    pub questdb_url: String,

    /// Maximum milliseconds between QuestDB flushes.
    #[arg(long, env = "FLUSH_INTERVAL_MS", default_value = "250")]
    pub flush_interval_ms: u64,

    /// Maximum rows buffered before forcing a flush.
    #[arg(long, env = "FLUSH_BATCH_SIZE", default_value = "500")]
    pub flush_batch_size: usize,
}
