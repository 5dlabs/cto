use std::sync::Arc;

use tonic::Status;

use crate::db::{self, DbClient};
use crate::error::Error;
use crate::proto::{
    GetOhlcvRequest, GetOhlcvResponse, GetPriceHistoryRequest, GetPriceHistoryResponse,
};

pub async fn handle_get_ohlcv(
    db: &Arc<DbClient>,
    req: &GetOhlcvRequest,
) -> Result<GetOhlcvResponse, Status> {
    if req.token.is_empty() {
        return Err(Status::invalid_argument("token is required"));
    }
    if req.time_from >= req.time_to {
        return Err(Status::invalid_argument("time_from must be before time_to"));
    }

    let interval = db::interval_to_sample_by(req.interval).map_err(|e: Error| Status::from(e))?;
    let candles = db
        .get_ohlcv(&req.token, interval, req.time_from, req.time_to)
        .await
        .map_err(Status::from)?;

    Ok(GetOhlcvResponse { candles })
}

pub async fn handle_get_price_history(
    db: &Arc<DbClient>,
    req: &GetPriceHistoryRequest,
) -> Result<GetPriceHistoryResponse, Status> {
    if req.token.is_empty() {
        return Err(Status::invalid_argument("token is required"));
    }
    if req.time_from >= req.time_to {
        return Err(Status::invalid_argument("time_from must be before time_to"));
    }

    let interval = db::interval_to_sample_by(req.interval).map_err(|e: Error| Status::from(e))?;
    let points = db
        .get_price_history(&req.token, interval, req.time_from, req.time_to)
        .await
        .map_err(Status::from)?;

    Ok(GetPriceHistoryResponse { points })
}
