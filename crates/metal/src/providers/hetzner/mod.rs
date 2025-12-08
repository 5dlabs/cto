//! Hetzner bare metal provider.
//!
//! Implements the [`Provider`] trait for Hetzner Robot API.
//!
//! ## Overview
//!
//! Hetzner offers excellent price-to-performance ratio with 100% green energy.
//! Locations: Germany (Falkenstein, Nuremberg), Finland (Helsinki).
//!
//! ## Recommended Plans
//!
//! - **AX52**: AMD Ryzen 7 7700 (8c/16t), 64GB DDR5, 2x1TB `NVMe` - €60/mo
//! - **AX102**: AMD Ryzen 9 7950X3D (16c/32t), 128GB DDR5 ECC - €104/mo
//! - **AX162-R**: AMD EPYC 9454P (48c/96t), 256GB DDR5 ECC - €199/mo

mod client;
mod models;

pub use client::Hetzner;
pub use models::*;
