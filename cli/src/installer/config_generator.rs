use anyhow::{Context, Result};
use std::path::Path;

use crate::config::{CtoConfig, InstallConfig};

pub struct ConfigGenerator;

impl ConfigGenerator {
    pub const fn new(_config: &InstallConfig) -> Self {
        Self
    }

    pub fn write_config(cto_config: &CtoConfig, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(cto_config)
            .context("Failed to serialize config")?;

        std::fs::write(path, json).context("Failed to write config file")?;

        Ok(())
    }
}

