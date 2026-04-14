use std::sync::Arc;

use tokio::sync::{broadcast, mpsc};
use tonic::Status;

use crate::db::DbClient;
use crate::proto::{PriceSuggestion, TokenPrice, Trade};
use crate::SwapEvent;

fn swap_to_trade(s: &SwapEvent) -> Trade {
    Trade {
        signature: s.signature.clone(),
        dex: s.dex.clone(),
        pool: s.pool.clone(),
        token_in: s.token_in.clone(),
        token_out: s.token_out.clone(),
        amount_in: s.amount_in,
        amount_out: s.amount_out,
        price: s.price,
        signer: s.signer.clone(),
        timestamp: s.timestamp,
        slot: s.slot,
    }
}

/// Server-streaming trade feed, filtered by token and optional pair.
pub fn trade_stream(
    mut rx: broadcast::Receiver<SwapEvent>,
    token: String,
    pair_token: String,
) -> tokio_stream::wrappers::ReceiverStream<Result<Trade, Status>> {
    let (tx, out_rx) = mpsc::channel(128);

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(swap) => {
                    let matches_token =
                        token.is_empty() || swap.token_in == token || swap.token_out == token;
                    let matches_pair = pair_token.is_empty()
                        || swap.token_in == pair_token
                        || swap.token_out == pair_token;

                    if matches_token
                        && matches_pair
                        && tx.send(Ok(swap_to_trade(&swap))).await.is_err()
                    {
                        break; // client disconnected
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::debug!(skipped = n, "trade stream lagged");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(out_rx)
}

/// Server-streaming price updates for a specific token.
pub fn price_stream(
    db: Arc<DbClient>,
    mut rx: broadcast::Receiver<SwapEvent>,
    token: String,
) -> tokio_stream::wrappers::ReceiverStream<Result<TokenPrice, Status>> {
    let (tx, out_rx) = mpsc::channel(128);

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(swap) => {
                    let involves = swap.token_in == token || swap.token_out == token;
                    if !involves {
                        continue;
                    }

                    match db.get_price(&token).await {
                        Ok(p) => {
                            if tx.send(Ok(p)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::debug!(error = %e, "price stream lookup failed");
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::debug!(skipped = n, "price stream lagged");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(out_rx)
}

/// Server-streaming suggestion updates for a specific token.
pub fn suggestion_stream(
    db: Arc<DbClient>,
    mut rx: broadcast::Receiver<SwapEvent>,
    token: String,
) -> tokio_stream::wrappers::ReceiverStream<Result<PriceSuggestion, Status>> {
    let (tx, out_rx) = mpsc::channel(128);

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(swap) => {
                    let involves = swap.token_in == token || swap.token_out == token;
                    if !involves {
                        continue;
                    }

                    match db.get_price_suggestion(&token).await {
                        Ok(suggestion) => {
                            if tx.send(Ok(suggestion)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::debug!(error = %e, "suggestion stream lookup failed");
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::debug!(skipped = n, "suggestion stream lagged");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(out_rx)
}
