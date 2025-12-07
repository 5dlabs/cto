//! Cherry Servers bare metal provider.
//!
//! Implements the [`Provider`] trait for Cherry Servers API.
//!
//! ## Overview
//!
//! Cherry Servers offers fully customizable bare metal with DevOps integrations.
//! Locations: Lithuania, Netherlands, USA.
//!
//! ## Recommended Plans
//!
//! - **E3-1240v3**: Intel Xeon E3-1240v3 (4c/8t), 32GB DDR3, 2x500GB SSD - ~$107/mo
//! - **E5-1620v4**: Intel Xeon E5-1620v4 (4c/8t), 64GB DDR4, 2x500GB SSD - ~$159/mo
//! - **AMD EPYC**: AMD EPYC 7402P (24c/48t), 128GB DDR4, 2x1TB `NVMe` - ~$399/mo

mod client;
mod models;

pub use client::Cherry;
pub use models::*;
