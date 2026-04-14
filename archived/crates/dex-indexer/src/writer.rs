use std::time::Duration;

use questdb::ingress::{Buffer, ProtocolVersion, Sender, TimestampNanos};
use tokio::sync::{broadcast, mpsc};

use crate::config::Config;
use crate::error::Error;
use crate::SwapEvent;

pub struct Writer {
    sender: Sender,
    buffer: Buffer,
    batch_size: usize,
    flush_interval: Duration,
    row_count: usize,
    total_flushed: u64,
}

impl Writer {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let addr = config
            .questdb_url
            .strip_prefix("http://")
            .unwrap_or(&config.questdb_url);
        let conf_str = format!("http::addr={addr};");
        let sender = Sender::from_conf(&conf_str).map_err(|e| Error::QuestDb(e.to_string()))?;

        Ok(Self {
            sender,
            buffer: Buffer::new(ProtocolVersion::V1),
            batch_size: config.flush_batch_size,
            flush_interval: Duration::from_millis(config.flush_interval_ms),
            row_count: 0,
            total_flushed: 0,
        })
    }

    pub async fn run(
        &mut self,
        mut swap_rx: mpsc::Receiver<SwapEvent>,
        mut shutdown: broadcast::Receiver<()>,
    ) -> Result<(), Error> {
        let mut interval = tokio::time::interval(self.flush_interval);
        // Don't burst-fire missed ticks after a slow flush.
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        tracing::info!("writer ready");

        loop {
            tokio::select! {
                Some(swap) = swap_rx.recv() => {
                    self.buffer_swap(&swap)?;
                    if self.row_count >= self.batch_size {
                        self.flush()?;
                    }
                }
                _ = interval.tick() => {
                    if self.row_count > 0 {
                        self.flush()?;
                    }
                }
                _ = shutdown.recv() => {
                    tracing::info!("writer shutting down, flushing remaining rows");
                    if self.row_count > 0 {
                        self.flush()?;
                    }
                    tracing::info!(total = self.total_flushed, "writer stopped");
                    return Ok(());
                }
            }
        }
    }

    fn buffer_swap(&mut self, swap: &SwapEvent) -> Result<(), Error> {
        self.buffer
            .table("dex_swaps")
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .symbol("dex", &swap.dex)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .symbol("pool", &swap.pool)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .symbol("signature", &swap.signature)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .symbol("token_in", &swap.token_in)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .symbol("token_out", &swap.token_out)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .symbol("signer", &swap.signer)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .column_f64("amount_in", swap.amount_in)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .column_f64("amount_out", swap.amount_out)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .column_f64("price", swap.price)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .column_i64("slot", swap.slot as i64)
            .map_err(|e| Error::QuestDb(e.to_string()))?
            .at(TimestampNanos::new(swap.timestamp))
            .map_err(|e| Error::QuestDb(e.to_string()))?;

        self.row_count += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        let count = self.row_count;
        self.sender
            .flush(&mut self.buffer)
            .map_err(|e| Error::QuestDb(e.to_string()))?;
        self.total_flushed += count as u64;
        self.row_count = 0;
        tracing::debug!(
            rows = count,
            total = self.total_flushed,
            "flushed to QuestDB"
        );
        Ok(())
    }
}
