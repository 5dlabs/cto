#![allow(clippy::unused_async)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::redundant_else)]

pub mod common;
pub mod http_transport_tests;
pub mod real_servers;
pub mod simple_forwarding_test;
pub mod tools_server_tests;

// Re-export common utilities for tests
pub use common::*;

use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup_integration_tests() {
    INIT.call_once(|| {
        // Set up logging for integration tests
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();

        tracing::info!("Integration test environment initialized");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_framework() {
        setup_integration_tests();
        // Integration framework setup successful
    }
}
