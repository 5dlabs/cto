//! Provider abstractions for bare metal cloud providers.

pub mod cherry;
pub mod hetzner;
pub mod latitude;
pub mod onprem;
pub mod ovh;
pub mod scaleway;
pub mod vultr;
mod traits;

pub use cherry::Cherry;
pub use hetzner::Hetzner;
pub use latitude::Latitude;
pub use onprem::OnPrem;
pub use ovh::Ovh;
pub use scaleway::Scaleway;
pub use traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};
pub use vultr::Vultr;
