//! Provider abstractions for bare metal cloud providers.

pub mod cherry;
pub mod factory;
pub mod hetzner;
pub mod latitude;
pub mod onprem;
pub mod ovh;
pub mod scaleway;
mod traits;
pub mod vultr;

// Re-export client structs for convenience
pub use cherry::Cherry;
pub use hetzner::Hetzner;
pub use latitude::Latitude;
pub use onprem::OnPrem;
pub use ovh::Ovh;
pub use scaleway::Scaleway;
pub use vultr::Vultr;

// Re-export factory
pub use factory::{create_provider, ProviderConfig, ProviderKind};

// Re-export common types
pub use traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};
