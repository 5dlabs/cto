//! `DigitalOcean` bare metal provider (Dedicated Droplets).
//!
//! Implements the [`Provider`] trait for `DigitalOcean` API.
//!
//! ## Overview
//!
//! `DigitalOcean` offers dedicated CPU droplets and premium AMD instances.
//! While not traditional bare metal, dedicated CPU instances provide
//! consistent performance without noisy neighbors.
//!
//! ## Recommended Plans
//!
//! - **c-8**: 8 vCPU dedicated, 16GB RAM - $168/mo
//! - **c-16**: 16 vCPU dedicated, 32GB RAM - $336/mo
//! - **c-32**: 32 vCPU dedicated, 64GB RAM - $672/mo
//! - **m-8vcpu-64gb**: Memory-optimized, 8 vCPU, 64GB - $336/mo

mod client;
mod models;

pub use client::DigitalOcean;
pub use models::*;
