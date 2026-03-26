use std::sync::Arc;

use tonic::Status;

use crate::db::DbClient;
use crate::proto::GetTokenOverviewResponse;

pub async fn handle_get_token_overview(
    db: &Arc<DbClient>,
    token: &str,
) -> Result<GetTokenOverviewResponse, Status> {
    if token.is_empty() {
        return Err(Status::invalid_argument("token is required"));
    }
    db.get_token_overview(token).await.map_err(Status::from)
}
