use std::sync::Arc;

use tonic::Status;

use crate::db::DbClient;
use crate::proto::{GetMultiPriceResponse, GetPriceResponse};

pub async fn handle_get_price(db: &Arc<DbClient>, token: &str) -> Result<GetPriceResponse, Status> {
    if token.is_empty() {
        return Err(Status::invalid_argument("token is required"));
    }
    let price = db.get_price(token).await.map_err(Status::from)?;
    Ok(GetPriceResponse { price: Some(price) })
}

pub async fn handle_get_multi_price(
    db: &Arc<DbClient>,
    tokens: &[String],
) -> Result<GetMultiPriceResponse, Status> {
    if tokens.is_empty() {
        return Err(Status::invalid_argument("at least one token required"));
    }
    if tokens.len() > 100 {
        return Err(Status::invalid_argument("max 100 tokens per request"));
    }
    let prices = db.get_multi_price(tokens).await.map_err(Status::from)?;
    Ok(GetMultiPriceResponse { prices })
}
