//! HTTP Client Library
//!
//! A simple HTTP client wrapper around reqwest.
//! 
//! ## TODO: Add retry logic
//! 
//! This client needs exponential backoff retry logic for:
//! - Connection errors
//! - 429 (Too Many Requests) responses
//! - 503 (Service Unavailable) responses

pub mod client;
pub mod error;

pub use client::HttpClient;
pub use error::HttpError;



