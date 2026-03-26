use std::collections::HashMap;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterTransactions,
};

use crate::config::Config;
use crate::decoder::Decoder;
use crate::dex::DexRegistry;
use crate::error::Error;
use crate::SwapEvent;

pub struct Subscriber {
    config: Config,
    registry: DexRegistry,
    decoder: Decoder,
}

impl Subscriber {
    pub fn new(config: Config) -> Self {
        let registry = DexRegistry::new();
        let decoder = Decoder::new(registry.clone());
        Self {
            config,
            registry,
            decoder,
        }
    }

    pub async fn run(
        self,
        swap_tx: mpsc::Sender<SwapEvent>,
        mut shutdown: broadcast::Receiver<()>,
    ) -> Result<(), Error> {
        let mut backoff = Duration::from_secs(1);

        loop {
            tokio::select! {
                result = self.connect_and_stream(&swap_tx) => {
                    match result {
                        Ok(()) => return Ok(()),
                        Err(e) => {
                            tracing::error!(
                                error = %e,
                                backoff_secs = backoff.as_secs(),
                                "gRPC stream error, reconnecting"
                            );
                            tokio::time::sleep(backoff).await;
                            backoff = (backoff * 2).min(Duration::from_secs(60));
                        }
                    }
                }
                _ = shutdown.recv() => {
                    tracing::info!("subscriber shutting down");
                    return Ok(());
                }
            }
        }
    }

    async fn connect_and_stream(&self, swap_tx: &mpsc::Sender<SwapEvent>) -> Result<(), Error> {
        tracing::info!(url = %self.config.grpc_url, "connecting to Yellowstone gRPC");

        let mut client = GeyserGrpcClient::build_from_shared(self.config.grpc_url.clone())
            .map_err(|e| Error::GrpcTransport(e.to_string()))?
            .x_token(None::<String>)
            .map_err(|e| Error::GrpcTransport(e.to_string()))?
            .connect()
            .await
            .map_err(|e| Error::GrpcTransport(e.to_string()))?;

        let program_ids = self.registry.all_program_ids();
        tracing::info!(
            program_count = program_ids.len(),
            "subscribing to DEX programs"
        );

        let request = SubscribeRequest {
            transactions: HashMap::from([(
                "dex_swaps".to_string(),
                SubscribeRequestFilterTransactions {
                    vote: Some(false),
                    failed: Some(false),
                    account_include: program_ids,
                    ..Default::default()
                },
            )]),
            commitment: Some(CommitmentLevel::Confirmed.into()),
            ..Default::default()
        };

        let (_sink, mut stream) = client
            .subscribe_with_request(Some(request))
            .await
            .map_err(|e| Error::GrpcTransport(e.to_string()))?;

        tracing::info!("subscribed, processing transaction stream");
        let mut swap_count = 0u64;
        let mut tx_count = 0u64;

        while let Some(msg) = stream.next().await {
            let msg = msg.map_err(|e| Error::GrpcTransport(e.to_string()))?;

            if let Some(UpdateOneof::Transaction(tx_update)) = msg.update_oneof {
                tx_count += 1;
                if tx_count <= 5 || tx_count.is_multiple_of(10000) {
                    tracing::debug!(tx_count, slot = tx_update.slot, "received transaction");
                }
                if let Some(info) = tx_update.transaction {
                    match self.decoder.decode(&info, tx_update.slot) {
                        Ok(swaps) => {
                            for swap in swaps {
                                swap_count += 1;
                                if swap_count.is_multiple_of(1000) {
                                    tracing::info!(
                                        swaps = swap_count,
                                        txs = tx_count,
                                        "processing"
                                    );
                                }
                                if swap_tx.send(swap).await.is_err() {
                                    return Ok(()); // writer dropped, shutting down
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!(error = %e, "decode error");
                        }
                    }
                }
            }
        }

        Err(Error::GrpcTransport("stream ended unexpectedly".into()))
    }
}
