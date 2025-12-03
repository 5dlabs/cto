//! Latitude.sh bare metal provider.
//!
//! Implements the [`Provider`] trait for Latitude.sh API.

mod client;
mod models;

pub use client::Latitude;
pub use models::*;
