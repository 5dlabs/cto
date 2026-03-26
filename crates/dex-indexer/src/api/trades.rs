use std::sync::Arc;

use tonic::Status;

use crate::db::DbClient;
use crate::proto::{
    GetPairTradesRequest, GetPairTradesResponse, GetTradesRequest, GetTradesResponse,
};

pub async fn handle_get_trades(
    db: &Arc<DbClient>,
    req: &GetTradesRequest,
) -> Result<GetTradesResponse, Status> {
    if req.token.is_empty() {
        return Err(Status::invalid_argument("token is required"));
    }
    let limit = if req.limit == 0 { 50 } else { req.limit };
    let trades = db
        .get_trades(&req.token, limit, req.before)
        .await
        .map_err(Status::from)?;

    Ok(GetTradesResponse { trades })
}

pub async fn handle_get_pair_trades(
    db: &Arc<DbClient>,
    req: &GetPairTradesRequest,
) -> Result<GetPairTradesResponse, Status> {
    if req.token_a.is_empty() || req.token_b.is_empty() {
        return Err(Status::invalid_argument(
            "both token_a and token_b are required",
        ));
    }
    let limit = if req.limit == 0 { 50 } else { req.limit };
    let trades = db
        .get_pair_trades(&req.token_a, &req.token_b, limit, req.before)
        .await
        .map_err(Status::from)?;

    Ok(GetPairTradesResponse { trades })
}
