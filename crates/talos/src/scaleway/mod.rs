pub mod client;
pub use client::ScalewayClient;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RescueModeResponse {
    pub rescue_ip: String,
    pub rescue_password: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub status: String,
    pub ip: Option<String>,
    pub rescue_mode: Option<bool>,
}
