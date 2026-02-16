//! Cherry Servers bare metal provider.
//!
//! Implements the [`Provider`] trait for Cherry Servers API.
//!
//! ## Overview
//!
//! Cherry Servers offers fully customizable bare metal with DevOps integrations.
//! Locations: Lithuania (LT-Siauliai), Netherlands (NL-Rotterdam), USA (US-Chicago, US-Seattle).
//!
//! ## Networking
//!
//! Unlike Latitude, Cherry Servers uses **IP-based networking** instead of VLANs:
//! - Public IPs are assigned automatically
//! - Private IPs can be created separately and assigned to servers
//! - No explicit VLAN configuration - network isolation is via IP assignment
//!
//! ## Recommended NVMe Plans (Cheapest)
//!
//! | Plan | CPU | RAM | Storage | est. Price |
//! |------|-----|-----|---------|------------|
//! | **E3-1240v3** | Intel Xeon E3-1240v3 (4c/8t) | 32GB DDR3 | 2x500GB SSD | ~$107/mo |
//! | **E5-1620v4** | Intel Xeon E5-1620v4 (4c/8t) | 64GB DDR4 | 2x500GB SSD | ~$159/mo |
//! | **D3-1435** | Intel Xeon D-1435 (10c/20t) | 64GB DDR4 | 2x480GB SSD | ~$199/mo |
//! | **AMD EPYC** | AMD EPYC 7402P (24c/48t) | 128GB DDR4 | 2x1TB NVMe | ~$399/mo |
//!
//! ## iPXE Support
//!
//! Cherry Servers supports custom iPXE boot via user_data field:
//! ```text
//! user_data: Some("#!ipxe\nchain http://example.com/ipxe".to_string())
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Initialize cherryctl (creates config at ~/.config/cherry/default.yaml)
//! cherryctl init
//!
//! # Create a server
//! cherryctl server create -p <project_id> --plan <plan_slug> --region LT-Siauliai --hostname test-server --image ubuntu_24_04
//!
//! # List servers
//! cherryctl server list
//! ```

mod client;
mod models;

pub use client::Cherry;
pub use models::*;
