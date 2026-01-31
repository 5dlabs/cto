//! Provider abstractions for bare metal cloud providers.

pub mod cherry;
pub mod factory;
pub mod hetzner;
pub mod latitude;
pub mod onprem;
pub mod ovh;
pub mod scaleway;
pub mod vultr;
mod traits;

pub use factory::{create_provider, ProviderConfig, ProviderKind};
pub use traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};
