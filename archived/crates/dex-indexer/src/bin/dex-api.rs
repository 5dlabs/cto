use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::sync::broadcast;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

use dex_indexer::api::{self, ApiState};
use dex_indexer::db::DbClient;
use dex_indexer::proto::dex_query_server::DexQueryServer;
use dex_indexer::proto::dex_stream_server::DexStreamServer;
use dex_indexer::SwapEvent;

#[derive(Parser, Clone, Debug)]
#[command(name = "dex-api", about = "gRPC DEX price feed API backed by QuestDB")]
struct Config {
    /// gRPC listen address.
    #[arg(long, env = "LISTEN_ADDR", default_value = "0.0.0.0:50051")]
    listen_addr: String,

    /// QuestDB PG wire host.
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
        listen = %config.listen_addr,
        pg_host = %config.questdb_pg_host,
        pg_port = config.questdb_pg_port,
        "dex-api starting"
    );

    let db = DbClient::connect(
        &config.questdb_pg_host,
        config.questdb_pg_port,
        &config.questdb_pg_user,
        &config.questdb_pg_password,
    )
    .await?;

    let db = Arc::new(db);

    let (trade_tx, _) = broadcast::channel::<SwapEvent>(4096);

    // Background poller for streaming.
    api::spawn_trade_poller(Arc::clone(&db), trade_tx.clone());

    let state = ApiState { db, trade_tx };

    let addr = config.listen_addr.parse()?;
    tracing::info!(%addr, "gRPC server listening");

    Server::builder()
        .add_service(DexQueryServer::new(state.clone()))
        .add_service(DexStreamServer::new(state))
        .serve(addr)
        .await?;

    Ok(())
}
