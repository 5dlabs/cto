// Commands module for Tauri frontend communication
// Provides wrapper functions that expose runtime and keychain operations

pub mod cluster;
pub mod config;

pub use crate::runtime::ensure_kind_installed;
pub use crate::runtime::get_all_cluster_status;
pub use crate::runtime::get_container_runtime;
pub use crate::runtime::get_runtime_info;
pub use crate::runtime::is_kind_cluster_running;

pub use crate::keychain::{delete_password, get_password, set_password};

pub use cluster::{
    delete_cluster, get_cluster_info, get_clusters_status, list_clusters, restart_cluster,
    start_cluster, stop_cluster, ClusterInfo, ClusterStatus, NodeInfo,
};
