//! HTTP Client implementation

use crate::error::HttpError;
use reqwest::Client;
use serde::de::DeserializeOwned;
use tracing::{debug, info};

/// HTTP client wrapper
pub struct HttpClient {
    client: Client,
    base_url: String,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Perform a GET request
    /// 
    /// TODO: Add retry logic with exponential backoff
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, HttpError> {
        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "Making GET request");

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(HttpError::StatusError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let body = response.json::<T>().await?;
        info!("GET request successful");
        Ok(body)
    }

    /// Perform a POST request
    /// 
    /// TODO: Add retry logic with exponential backoff
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, HttpError> {
        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "Making POST request");

        let response = self.client.post(&url).json(body).send().await?;

        if !response.status().is_success() {
            return Err(HttpError::StatusError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let body = response.json::<T>().await?;
        info!("POST request successful");
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = HttpClient::new("https://api.example.com");
        assert_eq!(client.base_url, "https://api.example.com");
    }

    // TODO: Add tests for retry logic
    // - Test that retries happen on 429/503
    // - Test exponential backoff timing
    // - Test max retry limit
    // - Test jitter randomization
}



