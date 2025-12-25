//! Talos Linux configuration and bootstrapping.
//!
//! This module provides utilities for generating Talos machine configs,
//! constructing iPXE URLs from the Talos Image Factory, and bootstrapping
//! Talos clusters.

mod bootstrap;
mod config;

pub use bootstrap::{
    apply_config, apply_config_with_vlan, bootstrap_cluster, check_talosctl, full_bootstrap,
    generate_config, generate_secrets, get_kubeconfig, wait_for_install, wait_for_kubernetes,
    wait_for_kubernetes_api_port, wait_for_node_ready, wait_for_talos, BootstrapConfig,
    GeneratedConfigs, VlanConfig, K8S_API_PORT, TALOS_API_PORT,
};
pub use config::{TalosConfig, TalosVersion, DEFAULT_SCHEMATIC_ID, DEFAULT_TALOS_VERSION};
