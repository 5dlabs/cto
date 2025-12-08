//! `OVHcloud` bare metal provider.
//!
//! Implements the [`Provider`] trait for `OVHcloud` API.
//!
//! ## Overview
//!
//! `OVHcloud` is a major European provider with global presence and unmetered bandwidth.
//! Locations: 40 data centers across Europe, Americas, Asia-Pacific.
//!
//! ## Recommended Plans
//!
//! - **Rise-1**: Intel Xeon-E 2388G (8c/16t), 32GB DDR4, 2x512GB `NVMe` - ~$70/mo
//! - **Rise-2**: Intel Xeon-E 2388G (8c/16t), 64GB DDR4, 2x960GB `NVMe` - ~$95/mo
//! - **Advance-1**: AMD EPYC 4344P (8c/16t), 64GB DDR5, 2x960GB `NVMe` - ~$130/mo
//! - **Scale-1**: AMD EPYC 9124 (16c/32t), 128GB DDR5, 2x960GB `NVMe` - ~$295/mo

mod client;
mod models;

pub use client::Ovh;
pub use models::*;
