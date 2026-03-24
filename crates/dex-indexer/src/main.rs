use anyhow::Result;
use clap::Parser;
use tokio::sync::{broadcast, mpsc};
use tracing_subscriber::EnvFilter;

use dex_indexer::{config::Config, subscriber::Subscriber, writer::Writer, SwapEvent};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("dex_indexer=info")),
        )
        .init();

    tracing::info!(
        grpc_url = %config.grpc_url,
        questdb_url = %config.questdb_url,
        flush_batch_size = config.flush_batch_size,
        flush_interval_ms = config.flush_interval_ms,
        "dex-indexer starting"
    );

    let (swap_tx, swap_rx) = mpsc::channel::<SwapEvent>(10_000);
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let subscriber = Subscriber::new(config.clone());
    let mut writer = Writer::new(&config)?;

    let sub_shutdown = shutdown_tx.subscribe();
    let writer_shutdown = shutdown_tx.subscribe();

    let sub_handle = tokio::spawn(async move { subscriber.run(swap_tx, sub_shutdown).await });

    let writer_handle = tokio::spawn(async move { writer.run(swap_rx, writer_shutdown).await });

    tokio::signal::ctrl_c().await?;
    tracing::info!("shutdown signal received");
    let _ = shutdown_tx.send(());

    let (sub_result, writer_result) = tokio::join!(sub_handle, writer_handle);
    if let Err(e) = sub_result {
        tracing::error!(error = %e, "subscriber task panicked");
    }
    if let Err(e) = writer_result {
        tracing::error!(error = %e, "writer task panicked");
    }

    tracing::info!("dex-indexer stopped");
    Ok(())
}
