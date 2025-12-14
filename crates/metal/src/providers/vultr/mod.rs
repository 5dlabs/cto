//! Vultr bare metal provider.
//!
//! Implements the [`Provider`] trait for Vultr API.
//!
//! ## Overview
//!
//! Vultr is US-based with 32 global locations and excellent developer experience.
//! Features: hourly billing, GPU instances, 10 Gbit/s network.
//!
//! ## Recommended Plans
//!
//! - **vbm-4c-32gb**: Intel E-2286G (6c/12t), 32GB, 2x480GB SSD - $120/mo
//! - **vbm-8c-64gb**: Intel E-2288G (8c/16t), 64GB, 2x960GB SSD - $185/mo
//! - **vbm-24c-128gb**: AMD EPYC 7443P (24c/48t), 128GB, 2x960GB `NVMe` - $350/mo

mod client;
mod models;

pub use client::Vultr;
pub use models::*;
