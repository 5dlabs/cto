use anyhow::{Context, Result};
use std::path::Path;

use crate::config::{CtoConfig, InstallConfig};

pub struct ConfigGenerator<'a> {
    _config: &'a InstallConfig,
}

impl<'a> ConfigGenerator<'a> {
    pub fn new(config: &'a InstallConfig) -> Self {
        Self { _config: config }
    }

    pub fn write_config(&self, cto_config: &CtoConfig, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(cto_config)
            .context("Failed to serialize config")?;

        std::fs::write(path, json).context("Failed to write config file")?;

        Ok(())
    }
}

