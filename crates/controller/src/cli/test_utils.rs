//! Shared test utilities for CLI adapter tests

use std::path::PathBuf;

/// Returns the path to agent templates directory from the workspace root.
/// Uses `CARGO_MANIFEST_DIR` and navigates up to workspace root.
#[cfg(test)]
#[must_use]
pub fn templates_root() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../infra/charts/controller/templates")
        .to_string_lossy()
        .into_owned()
}
