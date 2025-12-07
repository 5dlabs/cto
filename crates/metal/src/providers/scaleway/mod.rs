//! Scaleway bare metal provider (Dedibox / Elastic Metal).
//!
//! Implements the [`Provider`] trait for Scaleway API.
//!
//! ## Overview
//!
//! Scaleway is a French cloud provider offering budget-friendly bare metal,
//! ARM-based servers, and Apple Silicon for CI/CD.
//! Locations: Paris, Amsterdam.
//!
//! ## Recommended Plans
//!
//! - **START-2-S-SATA**: Intel Atom, 16GB, 2x1TB SATA - €10/mo
//! - **PRO-6-S-SSD**: Intel Xeon E-2274G, 64GB, 2x500GB SSD - €90/mo
//! - **CORE-8-L-H**: AMD EPYC 7402P, 256GB, 2x1.92TB `NVMe` - €299/mo

mod client;
mod models;

pub use client::Scaleway;
pub use models::*;
