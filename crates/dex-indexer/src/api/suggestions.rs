use std::sync::Arc;

use tonic::Status;

use crate::db::DbClient;
use crate::proto::{GetMultiPriceSuggestionResponse, GetPriceSuggestionResponse};

pub async fn handle_get_price_suggestion(
    db: &Arc<DbClient>,
    token: &str,
) -> Result<GetPriceSuggestionResponse, Status> {
    if token.is_empty() {
        return Err(Status::invalid_argument("token is required"));
    }
    let suggestion = db.get_price_suggestion(token).await.map_err(Status::from)?;
    Ok(GetPriceSuggestionResponse {
        suggestion: Some(suggestion),
    })
}

pub async fn handle_get_multi_price_suggestion(
    db: &Arc<DbClient>,
    tokens: &[String],
) -> Result<GetMultiPriceSuggestionResponse, Status> {
    if tokens.is_empty() {
        return Err(Status::invalid_argument("at least one token required"));
    }
    if tokens.len() > 100 {
        return Err(Status::invalid_argument("max 100 tokens per request"));
    }
    let suggestions = db
        .get_multi_price_suggestion(tokens)
        .await
        .map_err(Status::from)?;
    Ok(GetMultiPriceSuggestionResponse { suggestions })
}
