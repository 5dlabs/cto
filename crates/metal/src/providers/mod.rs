//! Provider abstractions for bare metal cloud providers.

pub mod latitude;
mod traits;

pub use traits::{
    CreateServerRequest, Provider, ProviderError, ReinstallIpxeRequest, Server, ServerStatus,
};
