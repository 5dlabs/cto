use std::path::PathBuf;

use crate::providers::{
    cherry::Cherry, hetzner::Hetzner, latitude::Latitude, onprem::OnPrem, ovh::Ovh,
    scaleway::Scaleway, vultr::Vultr, Provider, ProviderError,
};

/// Supported bare-metal providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ProviderKind {
    Latitude,
    Hetzner,
    Ovh,
    Vultr,
    Scaleway,
    Cherry,
    OnPrem,
}

/// Configuration for creating a provider instance.
#[derive(Clone)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    // Latitude
    pub latitude_api_key: Option<String>,
    pub latitude_project_id: Option<String>,
    // Hetzner
    pub hetzner_user: Option<String>,
    pub hetzner_password: Option<String>,
    // OVH
    pub ovh_app_key: Option<String>,
    pub ovh_app_secret: Option<String>,
    pub ovh_consumer_key: Option<String>,
    pub ovh_subsidiary: Option<String>,
    // Vultr
    pub vultr_api_key: Option<String>,
    // Scaleway
    pub scaleway_secret_key: Option<String>,
    pub scaleway_org_id: Option<String>,
    pub scaleway_project_id: Option<String>,
    pub scaleway_zone: Option<String>,
    // Cherry
    pub cherry_api_key: Option<String>,
    pub cherry_team_id: Option<i64>,
    // OnPrem
    pub onprem_inventory_path: Option<PathBuf>,
}

/// Helper to extract a required config field or return a `ProviderError::Config`.
fn required(value: Option<String>, field: &str, provider: &str) -> Result<String, ProviderError> {
    value.ok_or_else(|| {
        ProviderError::Config(format!("{field} is required for {provider} provider"))
    })
}

/// Create a provider instance based on configuration.
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn Provider>, ProviderError> {
    match config.kind {
        ProviderKind::Latitude => {
            let api_key = required(config.latitude_api_key, "latitude_api_key", "Latitude")?;
            let project_id = required(
                config.latitude_project_id,
                "latitude_project_id",
                "Latitude",
            )?;
            Ok(Box::new(Latitude::new(api_key, project_id)?))
        }
        ProviderKind::Hetzner => {
            let user = required(config.hetzner_user, "hetzner_user", "Hetzner")?;
            let password = required(config.hetzner_password, "hetzner_password", "Hetzner")?;
            Ok(Box::new(Hetzner::new(user, password)?))
        }
        ProviderKind::Ovh => {
            let app_key = required(config.ovh_app_key, "ovh_app_key", "OVH")?;
            let app_secret = required(config.ovh_app_secret, "ovh_app_secret", "OVH")?;
            let consumer_key = required(config.ovh_consumer_key, "ovh_consumer_key", "OVH")?;
            if let Some(subsidiary) = config.ovh_subsidiary {
                Ok(Box::new(Ovh::with_subsidiary(
                    app_key,
                    app_secret,
                    consumer_key,
                    subsidiary,
                )?))
            } else {
                Ok(Box::new(Ovh::new(app_key, app_secret, consumer_key)?))
            }
        }
        ProviderKind::Vultr => {
            let api_key = required(config.vultr_api_key, "vultr_api_key", "Vultr")?;
            Ok(Box::new(Vultr::new(api_key)?))
        }
        ProviderKind::Scaleway => {
            let secret_key = required(
                config.scaleway_secret_key,
                "scaleway_secret_key",
                "Scaleway",
            )?;
            let org_id = required(config.scaleway_org_id, "scaleway_org_id", "Scaleway")?;
            let project_id = required(
                config.scaleway_project_id,
                "scaleway_project_id",
                "Scaleway",
            )?;
            let zone = required(config.scaleway_zone, "scaleway_zone", "Scaleway")?;
            Ok(Box::new(Scaleway::new(
                secret_key, org_id, project_id, zone,
            )?))
        }
        ProviderKind::Cherry => {
            let api_key = required(config.cherry_api_key, "cherry_api_key", "Cherry")?;
            let team_id = config.cherry_team_id.ok_or_else(|| {
                ProviderError::Config("cherry_team_id is required for Cherry provider".to_string())
            })?;
            Ok(Box::new(Cherry::new(api_key, team_id)?))
        }
        ProviderKind::OnPrem => Ok(Box::new(OnPrem::new(config.onprem_inventory_path)?)),
    }
}
