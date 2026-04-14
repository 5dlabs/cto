use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use dex_indexer::backfill::Backfiller;
use dex_indexer::config::Config as IndexerConfig;
use dex_indexer::db::DbClient;
use dex_indexer::dex::DexRegistry;
use dex_indexer::writer::Writer;

#[derive(Parser, Clone, Debug)]
#[command(
    name = "dex-backfill",
    about = "Backfill historical DEX swaps from Solana RPC"
)]
struct Config {
    /// Solana JSON-RPC URL.
    #[arg(long, env = "SOLANA_RPC_URL", default_value = "http://127.0.0.1:8899")]
    solana_rpc_url: String,

    /// QuestDB HTTP endpoint (for writing via ILP).
    #[arg(
        long,
        env = "QUESTDB_URL",
        default_value = "http://questdb.questdb.svc:9000"
    )]
    questdb_url: String,

    /// QuestDB PG wire host (for reading earliest timestamp).
    #[arg(long, env = "QUESTDB_PG_HOST", default_value = "questdb.questdb.svc")]
    questdb_pg_host: String,

    /// QuestDB PG wire port.
    #[arg(long, env = "QUESTDB_PG_PORT", default_value = "8812")]
    questdb_pg_port: u16,

    /// QuestDB PG wire user.
    #[arg(long, env = "QUESTDB_PG_USER", default_value = "admin")]
    questdb_pg_user: String,

    /// QuestDB PG wire password.
    #[arg(long, env = "QUESTDB_PG_PASSWORD", default_value = "quest")]
    questdb_pg_password: String,

    /// Specific DEX program to backfill (if omitted, backfills all).
    #[arg(long)]
    program: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("dex_indexer=info")),
        )
        .init();

    tracing::info!(
        rpc = %config.solana_rpc_url,
        questdb = %config.questdb_url,
        "dex-backfill starting"
    );

    // Connect to QuestDB PG wire to find earliest existing data.
    let db = DbClient::connect(
        &config.questdb_pg_host,
        config.questdb_pg_port,
        &config.questdb_pg_user,
        &config.questdb_pg_password,
    )
    .await?;

    let earliest = db.earliest_timestamp().await?;
    tracing::info!(earliest_nanos = ?earliest, "existing data boundary");

    // Prepare writer for inserting backfilled swaps.
    let indexer_config = IndexerConfig {
        grpc_url: String::new(),
        questdb_url: config.questdb_url.clone(),
        flush_interval_ms: 250,
        flush_batch_size: 500,
    };
    let mut writer = Writer::new(&indexer_config)?;

    // Determine which programs to backfill.
    let registry = DexRegistry::new();
    let program_ids = match &config.program {
        Some(p) => vec![p.clone()],
        None => registry.all_program_ids(),
    };

    let backfiller = Backfiller::new(config.solana_rpc_url.clone());

    let (swap_tx, swap_rx) = tokio::sync::mpsc::channel(10_000);
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
    let writer_shutdown = shutdown_tx.subscribe();

    // Spawn writer task.
    let writer_handle = tokio::spawn(async move { writer.run(swap_rx, writer_shutdown).await });

    // Walk each program.
    let mut total = 0usize;
    for program_id in &program_ids {
        tracing::info!(program_id, "starting backfill");

        match backfiller.walk_program(program_id, earliest).await {
            Ok(swaps) => {
                let count = swaps.len();
                for swap in swaps {
                    if swap_tx.send(swap).await.is_err() {
                        tracing::error!("writer channel closed");
                        break;
                    }
                }
                total += count;
                tracing::info!(program_id, swaps = count, "backfill batch complete");
            }
            Err(e) => {
                tracing::error!(program_id, error = %e, "backfill failed");
            }
        }
    }

    // Signal writer to flush and stop.
    drop(swap_tx);
    let _ = shutdown_tx.send(());
    let _ = writer_handle.await;

    tracing::info!(total, "backfill complete");
    Ok(())
}
