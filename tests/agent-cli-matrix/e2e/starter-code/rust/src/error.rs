//! Error types for the HTTP client

use thiserror::Error;

/// HTTP client errors
#[derive(Error, Debug)]
pub enum HttpError {
    /// Request failed with an error status code
    #[error("HTTP request failed with status {status}: {message}")]
    StatusError {
        /// HTTP status code
        status: u16,
        /// Error message from the response body
        message: String,
    },

    /// Network or connection error
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}
