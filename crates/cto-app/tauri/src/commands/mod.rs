// Commands module for Tauri frontend communication
// Provides wrapper functions that expose runtime and keychain operations

pub mod cluster;
pub mod config;

pub use crate::runtime::auto_provision_runtime;
pub use crate::runtime::check_docker_running;
pub use crate::runtime::ensure_kind_installed;
pub use crate::runtime::get_all_cluster_status;
pub use crate::runtime::get_container_runtime;
pub use crate::runtime::get_docker_socket;
pub use crate::runtime::get_runtime_info;
pub use crate::runtime::is_kind_cluster_running;

pub use crate::keychain::{delete_password, get_password, set_password};

pub use cluster::{
    delete_kind_cluster, get_cluster_status, list_clusters, start_kind_cluster, ClusterStatus,
};
