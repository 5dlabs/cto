pub mod ohlcv;
pub mod overview;
pub mod price;
pub mod stream;
pub mod suggestions;
pub mod trades;

use std::sync::Arc;

use tokio::sync::broadcast;
use tonic::{Request, Response, Status};

use crate::db::DbClient;
use crate::proto::dex_query_server::DexQuery;
use crate::proto::dex_stream_server::DexStream;
use crate::proto::{
    GetMultiPriceRequest, GetMultiPriceResponse, GetMultiPriceSuggestionRequest,
    GetMultiPriceSuggestionResponse, GetOhlcvRequest, GetOhlcvResponse, GetPairTradesRequest,
    GetPairTradesResponse, GetPriceHistoryRequest, GetPriceHistoryResponse, GetPriceRequest,
    GetPriceResponse, GetPriceSuggestionRequest, GetPriceSuggestionResponse,
    GetTokenOverviewRequest, GetTokenOverviewResponse, GetTradesRequest, GetTradesResponse,
    PriceSuggestion, StreamPriceRequest, StreamSuggestionRequest, StreamTradesRequest, TokenPrice,
    Trade,
};
use crate::SwapEvent;

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<DbClient>,
    pub trade_tx: broadcast::Sender<SwapEvent>,
}

// ── DexQuery ─────────────────────────────────────────────────────────────

#[tonic::async_trait]
impl DexQuery for ApiState {
    async fn get_price(
        &self,
        req: Request<GetPriceRequest>,
    ) -> Result<Response<GetPriceResponse>, Status> {
        let token = &req.into_inner().token;
        let p = price::handle_get_price(&self.db, token).await?;
        Ok(Response::new(p))
    }

    async fn get_multi_price(
        &self,
        req: Request<GetMultiPriceRequest>,
    ) -> Result<Response<GetMultiPriceResponse>, Status> {
        let tokens = &req.into_inner().tokens;
        let p = price::handle_get_multi_price(&self.db, tokens).await?;
        Ok(Response::new(p))
    }

    async fn get_ohlcv(
        &self,
        req: Request<GetOhlcvRequest>,
    ) -> Result<Response<GetOhlcvResponse>, Status> {
        let r = req.into_inner();
        let resp = ohlcv::handle_get_ohlcv(&self.db, &r).await?;
        Ok(Response::new(resp))
    }

    async fn get_trades(
        &self,
        req: Request<GetTradesRequest>,
    ) -> Result<Response<GetTradesResponse>, Status> {
        let r = req.into_inner();
        let resp = trades::handle_get_trades(&self.db, &r).await?;
        Ok(Response::new(resp))
    }

    async fn get_pair_trades(
        &self,
        req: Request<GetPairTradesRequest>,
    ) -> Result<Response<GetPairTradesResponse>, Status> {
        let r = req.into_inner();
        let resp = trades::handle_get_pair_trades(&self.db, &r).await?;
        Ok(Response::new(resp))
    }

    async fn get_token_overview(
        &self,
        req: Request<GetTokenOverviewRequest>,
    ) -> Result<Response<GetTokenOverviewResponse>, Status> {
        let token = &req.into_inner().token;
        let resp = overview::handle_get_token_overview(&self.db, token).await?;
        Ok(Response::new(resp))
    }

    async fn get_price_history(
        &self,
        req: Request<GetPriceHistoryRequest>,
    ) -> Result<Response<GetPriceHistoryResponse>, Status> {
        let r = req.into_inner();
        let resp = ohlcv::handle_get_price_history(&self.db, &r).await?;
        Ok(Response::new(resp))
    }

    async fn get_price_suggestion(
        &self,
        req: Request<GetPriceSuggestionRequest>,
    ) -> Result<Response<GetPriceSuggestionResponse>, Status> {
        let token = &req.into_inner().token;
        let resp = suggestions::handle_get_price_suggestion(&self.db, token).await?;
        Ok(Response::new(resp))
    }

    async fn get_multi_price_suggestion(
        &self,
        req: Request<GetMultiPriceSuggestionRequest>,
    ) -> Result<Response<GetMultiPriceSuggestionResponse>, Status> {
        let tokens = &req.into_inner().tokens;
        let resp = suggestions::handle_get_multi_price_suggestion(&self.db, tokens).await?;
        Ok(Response::new(resp))
    }
}

// ── DexStream ────────────────────────────────────────────────────────────

type TradeStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<Trade, Status>> + Send>>;
type PriceStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<TokenPrice, Status>> + Send>>;
type SuggestionStream =
    std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<PriceSuggestion, Status>> + Send>>;

#[tonic::async_trait]
impl DexStream for ApiState {
    type StreamTradesStream = TradeStream;
    type StreamPriceStream = PriceStream;
    type StreamSuggestionStream = SuggestionStream;

    async fn stream_trades(
        &self,
        req: Request<StreamTradesRequest>,
    ) -> Result<Response<Self::StreamTradesStream>, Status> {
        let r = req.into_inner();
        let rx = self.trade_tx.subscribe();
        let s = stream::trade_stream(rx, r.token, r.pair_token);
        Ok(Response::new(Box::pin(s)))
    }

    async fn stream_price(
        &self,
        req: Request<StreamPriceRequest>,
    ) -> Result<Response<Self::StreamPriceStream>, Status> {
        let r = req.into_inner();
        let rx = self.trade_tx.subscribe();
        let db = Arc::clone(&self.db);
        let s = stream::price_stream(db, rx, r.token);
        Ok(Response::new(Box::pin(s)))
    }

    async fn stream_suggestion(
        &self,
        req: Request<StreamSuggestionRequest>,
    ) -> Result<Response<Self::StreamSuggestionStream>, Status> {
        let r = req.into_inner();
        let rx = self.trade_tx.subscribe();
        let db = Arc::clone(&self.db);
        let s = stream::suggestion_stream(db, rx, r.token);
        Ok(Response::new(Box::pin(s)))
    }
}

/// Spawn background task that polls QuestDB for new swaps and broadcasts them.
pub fn spawn_trade_poller(db: Arc<DbClient>, tx: broadcast::Sender<SwapEvent>) {
    tokio::spawn(async move {
        let mut last_ts: i64 = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;

            match db.get_recent_swaps(last_ts).await {
                Ok(swaps) => {
                    for swap in swaps {
                        if swap.timestamp > last_ts {
                            last_ts = swap.timestamp;
                        }
                        let _ = tx.send(swap);
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "trade poller query failed");
                }
            }
        }
    });
}
