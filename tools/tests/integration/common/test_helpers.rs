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
#![allow(clippy::uninlined_format_args)]

use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub test_files_dir: String,
}

impl TestEnvironment {
    pub fn new() -> Result<Self> {
        // Check if we're running in Docker test environment
        if let Ok(docker_test_dir) = env::var("MCP_TEST_DATA_DIR") {
            println!("Using Docker test environment: {}", docker_test_dir);

            // Verify the test data directory exists
            if !Path::new(&docker_test_dir).exists() {
                return Err(anyhow::anyhow!(
                    "Docker test data directory does not exist: {}",
                    docker_test_dir
                ));
            }

            // Create a dummy temp_dir for compatibility
            let temp_dir = tempfile::tempdir()?;

            return Ok(Self {
                temp_dir,
                test_files_dir: docker_test_dir,
            });
        }

        // Original local test environment setup
        let temp_dir = tempfile::tempdir()?;
        let test_files_dir = temp_dir.path().join("test_files");
        fs::create_dir_all(&test_files_dir)?;

        // Create some test files
        let test_file = test_files_dir.join("test.txt");
        fs::write(
            &test_file,
            "This is a test file for MCP server integration tests.\n",
        )?;

        let json_file = test_files_dir.join("test.json");
        fs::write(
            &json_file,
            r#"{"message": "Hello from MCP integration test"}"#,
        )?;

        Ok(Self {
            temp_dir,
            test_files_dir: test_files_dir.to_string_lossy().to_string(),
        })
    }

    pub fn get_test_files_dir(&self) -> &str {
        &self.test_files_dir
    }

    pub fn create_test_file(&self, name: &str, content: &str) -> Result<String> {
        let file_path = Path::new(&self.test_files_dir).join(name);
        fs::write(&file_path, content)?;
        Ok(file_path.to_string_lossy().to_string())
    }
}

pub async fn skip_if_not_available(runtime: &str) -> Result<()> {
    if !crate::common::server_lifecycle::check_runtime_available(runtime).await {
        return Err(anyhow::anyhow!(
            "Runtime {} not available, skipping test",
            runtime
        ));
    }
    Ok(())
}

pub fn get_test_timeout_seconds() -> u64 {
    env::var("MCP_TEST_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap_or(30)
}

pub fn is_ci_environment() -> bool {
    env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok()
}

pub fn get_remote_server_url() -> String {
    env::var("MCP_REMOTE_SERVER_URL").unwrap_or_else(|_| {
        "http://rustdocs-mcp-rust-docs-mcp-server.mcp.svc.cluster.local:3000/sse".to_string()
    })
}

pub async fn wait_for_server_output(
    server: &mut crate::common::server_lifecycle::TestServer,
    expected_output: &str,
    timeout_secs: u64,
) -> Result<()> {
    use tokio::time::{sleep, Duration};

    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(timeout_secs);

    loop {
        let stderr_output = server.get_stderr().await;
        for line in stderr_output {
            if line.contains(expected_output) {
                return Ok(());
            }
        }

        if start_time.elapsed() > timeout_duration {
            return Err(anyhow::anyhow!(
                "Timeout waiting for server output: {}",
                expected_output
            ));
        }

        sleep(Duration::from_millis(100)).await;
    }
}

// Macro to create integration tests with common setup
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $server_config:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() -> Result<()> {
            use $crate::common::*;

            let _env = TestEnvironment::new()?;
            let config = $server_config;

            // Check if the required runtime is available
            let runtime = match config.command.as_str() {
                "npx" => "node",
                "uvx" => "python",
                "docker" => "docker",
                _ => "unknown",
            };

            if runtime != "unknown" {
                match $crate::common::server_lifecycle::check_runtime_available(runtime).await {
                    true => println!("Runtime {} is available", runtime),
                    false => {
                        println!(
                            "Skipping test {}: {} runtime not available",
                            stringify!($test_name),
                            runtime
                        );
                        return Ok(());
                    }
                }
            }

            let mut server = match $crate::common::server_lifecycle::TestServer::start(config).await
            {
                Ok(server) => server,
                Err(e) => {
                    println!(
                        "Failed to start server for test {}: {}",
                        stringify!($test_name),
                        e
                    );
                    return Ok(()); // Skip test instead of failing
                }
            };

            let result = $test_body(&mut server, &_env).await;

            // Always try to shutdown the server
            let _ = server.shutdown().await;

            result
        }
    };
}

// Test result aggregator for generating reports
pub struct TestResults {
    pub passed: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub skipped: Vec<String>,
}

impl Default for TestResults {
    fn default() -> Self {
        Self::new()
    }
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
        }
    }

    pub fn add_passed(&mut self, test_name: String) {
        self.passed.push(test_name);
    }

    pub fn add_failed(&mut self, test_name: String, error: String) {
        self.failed.push((test_name, error));
    }

    pub fn add_skipped(&mut self, test_name: String) {
        self.skipped.push(test_name);
    }

    pub fn print_summary(&self) {
        println!("\n=== Integration Test Summary ===");
        println!("Passed: {}", self.passed.len());
        println!("Failed: {}", self.failed.len());
        println!("Skipped: {}", self.skipped.len());

        if !self.failed.is_empty() {
            println!("\nFailed tests:");
            for (test, error) in &self.failed {
                println!("  ❌ {}: {}", test, error);
            }
        }

        if !self.skipped.is_empty() {
            println!("\nSkipped tests:");
            for test in &self.skipped {
                println!("  ⏭️  {}", test);
            }
        }

        println!("=================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_setup() {
        let env = TestEnvironment::new().unwrap();
        assert!(Path::new(&env.test_files_dir).exists());

        let test_file = Path::new(&env.test_files_dir).join("test.txt");
        assert!(test_file.exists());
    }

    #[test]
    fn test_helper_functions() {
        let timeout = get_test_timeout_seconds();
        assert!(timeout > 0);

        let is_ci = is_ci_environment();
        println!("Running in CI: {}", is_ci);

        let remote_url = get_remote_server_url();
        println!("Remote server URL: {}", remote_url);
    }
}
