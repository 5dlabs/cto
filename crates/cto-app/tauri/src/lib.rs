// Library root - exports all modules and commands
// CTO App Tauri Backend

pub mod commands;
pub mod keychain;
pub mod runtime;

// Re-export runtime functions for use as Tauri commands
pub use runtime::get_container_runtime;
pub use runtime::get_runtime_info;
pub use runtime::ensure_kind_installed;
pub use runtime::is_kind_cluster_running;
pub use runtime::get_all_cluster_status;

pub use keychain::{get_password, set_password, delete_password};
