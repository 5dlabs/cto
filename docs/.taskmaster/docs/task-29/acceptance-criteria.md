# Acceptance Criteria: Agent-Specific Container Scripts for Multi-Agent Workflows

## Rex Agent Script Requirements (container-rex.sh.hbs)

### ✅ MCP Documentation Integration


- [ ] Script successfully queries MCP documentation server at configured endpoint


- [ ] Uses `rustdocs_query_rust_docs` or equivalent API for documentation retrieval


- [ ] Handles MCP server connectivity failures gracefully with retry logic


- [ ] Incorporates retrieved documentation into implementation context


- [ ] Saves documentation context to `documentation_context.md` for reference

### ✅ Documentation Query Functionality


- [ ] Extracts key technologies from task description (rust, cargo, kubernetes, etc.)


- [ ] Queries documentation for relevant architectural patterns


- [ ] Retrieves API documentation with code examples


- [ ] Handles API rate limiting with appropriate delays


- [ ] Caches documentation responses to avoid redundant queries

### ✅ Implementation Plan Generation


- [ ] Generates comprehensive implementation plan based on documentation


- [ ] Creates step-by-step approach following documented patterns


- [ ] Includes architecture decisions with documentation references


- [ ] Saves implementation plan to `implementation_plan.md`


- [ ] Plan includes testing strategy and documentation requirements

### ✅ Documentation-First Code Generation


- [ ] Creates implementation files with extensive inline documentation


- [ ] Comments reference source documentation and architectural decisions


- [ ] Includes comprehensive module-level documentation


- [ ] Generates usage examples based on documentation patterns


- [ ] Creates README files with architecture overview and usage instructions

### ✅ Git Integration


- [ ] Commits changes with descriptive messages referencing documentation


- [ ] Includes co-author attribution to MCP documentation server


- [ ] Creates meaningful commit history explaining implementation decisions


- [ ] Stages files appropriately (implementation, documentation, examples)


- [ ] Handles git configuration for Rex agent identity

### ✅ Error Handling and Logging


- [ ] Implements structured logging with timestamps and agent identification


- [ ] Handles MCP server unavailability without failing workflow


- [ ] Provides meaningful error messages for debugging


- [ ] Implements retry logic for transient network failures


- [ ] Logs all major workflow steps for audit trail

## Cleo Agent Script Requirements (container-cleo.sh.hbs)

### ✅ Code Formatting Pipeline


- [ ] Runs `cargo fmt --check` to identify formatting issues before applying fixes


- [ ] Applies `cargo fmt` to auto-fix formatting problems


- [ ] Generates before/after comparison showing formatting changes


- [ ] Handles projects with no formatting issues gracefully


- [ ] Preserves original files if formatting fails

### ✅ Linting and Quality Analysis


- [ ] Executes `cargo clippy` with strict warning levels (-D warnings)


- [ ] Generates clippy analysis reports with issue categorization


- [ ] Applies automatic clippy fixes where possible using `--fix` flag


- [ ] Handles clippy issues requiring manual intervention


- [ ] Reports remaining issues that need developer attention

### ✅ Import Organization


- [ ] Organizes use statements following Rust conventions


- [ ] Removes unused imports and dead code detection


- [ ] Groups imports logically (std, external crates, local modules)


- [ ] Handles complex import scenarios without breaking compilation


- [ ] Reports changes made to import organization

### ✅ Quality Reporting


- [ ] Generates comprehensive quality report with metrics and improvements


- [ ] Creates JSON summary for automation and dashboard consumption


- [ ] Includes before/after quality metrics (LOC, TODO count, potential panics)


- [ ] Lists all changes applied during quality improvement process


- [ ] Provides recommendations for further quality improvements

### ✅ PR Integration


- [ ] Labels PR as 'ready-for-qa' after successful quality checks


- [ ] Adds 'cleo-processed' label to indicate Cleo has processed the PR


- [ ] Handles GitHub API authentication and rate limiting


- [ ] Creates PR comments summarizing quality improvements if applicable


- [ ] Fails gracefully if GitHub integration is not available

### ✅ Tool Validation


- [ ] Verifies cargo, rustfmt, and clippy are available before proceeding


- [ ] Installs missing components (rustfmt, clippy) if needed


- [ ] Validates tool versions are compatible with project requirements


- [ ] Handles tool installation failures gracefully


- [ ] Reports tool status and versions in logs

## Tess Agent Script Requirements (container-tess.sh.hbs)

### ✅ Comprehensive Test Execution


- [ ] Runs unit tests with `cargo test --all-features --no-fail-fast`


- [ ] Executes integration tests with `cargo test --test '*'`


- [ ] Runs documentation tests with `cargo test --doc`


- [ ] Handles test failures without stopping entire workflow


- [ ] Generates detailed test logs for each test category

### ✅ Coverage Analysis


- [ ] Generates coverage reports using cargo llvm-cov or tarpaulin


- [ ] Creates LCOV format reports for integration with other tools


- [ ] Generates HTML coverage reports for human review


- [ ] Calculates overall coverage percentage accurately


- [ ] Tracks coverage by file, function, and line level

### ✅ Coverage Threshold Validation


- [ ] Validates coverage meets configured threshold (default 95%)


- [ ] Supports different thresholds for existing vs new code


- [ ] Fails workflow if coverage threshold not met


- [ ] Provides detailed feedback on coverage gaps


- [ ] Generates recommendations for improving coverage

### ✅ Deployment Readiness Validation


- [ ] Tests release build compilation with `cargo build --release`


- [ ] Runs performance benchmarks if available with `cargo bench`


- [ ] Executes security audit with `cargo audit`


- [ ] Validates configuration files (Cargo.toml, Dockerfile, etc.)


- [ ] Checks for common deployment issues

### ✅ Test Reporting


- [ ] Generates comprehensive test report in Markdown format


- [ ] Creates JSON summary for automation and dashboard consumption


- [ ] Includes test results, coverage metrics, and deployment readiness


- [ ] Provides actionable recommendations for test failures


- [ ] Reports performance benchmark results if available

### ✅ PR Management


- [ ] Approves PR automatically if all tests pass and coverage requirements met


- [ ] Creates detailed test summary in PR comments


- [ ] Adds appropriate labels ('tess-approved', 'tests-passing')


- [ ] Blocks approval if tests fail or coverage insufficient


- [ ] Provides clear feedback on why approval was withheld

### ✅ Tool Management


- [ ] Installs required testing tools (cargo-llvm-cov, cargo-nextest, etc.)


- [ ] Handles tool installation failures gracefully


- [ ] Validates tool versions and compatibility


- [ ] Reports tool status and availability in logs


- [ ] Uses appropriate fallback tools if primary tools unavailable

## Common Script Requirements

### ✅ Template Variable Handling


- [ ] Correctly processes all template variables (github_app, task_id, etc.)


- [ ] Handles missing or invalid template variables gracefully


- [ ] Uses agent-specific variables appropriately (mcp_server_url, coverage_threshold)


- [ ] Escapes template variables properly to prevent injection attacks


- [ ] Validates template variable values before use

### ✅ Environment Initialization


- [ ] Sets up workspace environment correctly for each agent


- [ ] Configures git with appropriate agent identity


- [ ] Validates required tools and dependencies


- [ ] Initializes logging with structured format


- [ ] Creates necessary directories and files

### ✅ Error Handling and Recovery


- [ ] Uses `set -euo pipefail` for robust error handling


- [ ] Implements retry logic for transient failures


- [ ] Provides meaningful error messages for debugging


- [ ] Handles network timeouts and API rate limits


- [ ] Cleans up resources on failure

### ✅ Logging and Monitoring


- [ ] Implements structured logging with timestamps and agent identification


- [ ] Logs all major workflow steps and decisions


- [ ] Provides progress indicators for long-running operations


- [ ] Separates INFO, WARN, and ERROR log levels appropriately


- [ ] Saves logs to agent-specific log files

### ✅ Workspace Management


- [ ] Operates within designated workspace path


- [ ] Handles concurrent execution without conflicts


- [ ] Maintains workspace isolation between agents


- [ ] Cleans up temporary files after completion


- [ ] Preserves important artifacts (reports, logs, etc.)

## Integration Requirements

### ✅ CodeRun CRD Compatibility


- [ ] Integrates properly with existing Custom Resource Definition structure


- [ ] Supports workflow stage transitions with status updates


- [ ] Handles all CRD fields appropriately


- [ ] Maintains backward compatibility with existing workflows


- [ ] Updates workflow metadata and annotations correctly

### ✅ GitHub Integration


- [ ] Handles GitHub API authentication securely


- [ ] Creates appropriate PR comments for agent-specific results


- [ ] Updates PR labels based on workflow outcomes


- [ ] Handles GitHub API rate limits and errors gracefully


- [ ] Supports PR approvals and status checks

### ✅ Kubernetes Integration


- [ ] Updates workflow status via Kubernetes API


- [ ] Emits metrics for Grafana dashboard monitoring


- [ ] Handles Kubernetes API authentication


- [ ] Supports graceful shutdown and cleanup


- [ ] Integrates with pod lifecycle management

### ✅ Monitoring and Observability


- [ ] Emits metrics for workflow execution time and status


- [ ] Provides telemetry hooks for external monitoring


- [ ] Supports structured logging for log aggregation


- [ ] Creates audit trail for all major actions


- [ ] Enables debugging and troubleshooting

## Performance Requirements

### ✅ Script Performance


- [ ] Rex documentation queries complete within 2 minutes


- [ ] Cleo quality checks complete within 5 minutes


- [ ] Tess comprehensive testing completes within 15 minutes


- [ ] Script initialization completes within 30 seconds


- [ ] Memory usage stays within container limits (1-2GB)

### ✅ Resource Management


- [ ] Handles large codebases efficiently


- [ ] Manages temporary file usage appropriately


- [ ] Optimizes network requests (caching, batching)


- [ ] Uses CPU resources efficiently


- [ ] Scales with project size predictably

## Security Requirements

### ✅ Token and Secret Handling


- [ ] Never logs GitHub tokens or other sensitive credentials


- [ ] Uses environment variables for sensitive configuration


- [ ] Implements secure temporary file handling


- [ ] Validates all external inputs


- [ ] Uses least-privilege principles for file system access

### ✅ Input Validation


- [ ] Validates template variables before use


- [ ] Sanitizes outputs before logging or displaying


- [ ] Prevents command injection attacks


- [ ] Handles untrusted input safely


- [ ] Implements proper error boundaries

## Testing and Validation

### ✅ Unit Testing


- [ ] Each agent script can be tested in isolation


- [ ] Mock environments available for testing


- [ ] Error scenarios covered by tests


- [ ] Edge cases handled appropriately


- [ ] Integration points tested

### ✅ Integration Testing


- [ ] Scripts work with actual CodeRun CRDs


- [ ] GitHub integration functions correctly


- [ ] Kubernetes API interactions work


- [ ] Workflow stage transitions happen correctly


- [ ] Concurrent execution handled properly

### ✅ End-to-End Testing


- [ ] Complete Rex → Cleo → Tess pipeline works


- [ ] Feedback loops handle Rex updates appropriately


- [ ] Cleanup and resource management work correctly


- [ ] Performance meets requirements under load


- [ ] Error recovery mechanisms function properly

## Deployment Requirements

### ✅ Production Readiness


- [ ] Scripts deployed via GitOps in infra/charts/controller/claude-templates/


- [ ] Container images include all required dependencies


- [ ] Configuration management handles environment differences


- [ ] Monitoring and alerting configured for script execution


- [ ] Backup and recovery procedures documented



### ✅ Backward Compatibility


- [ ] Existing workflows continue to function


- [ ] Legacy PVC naming conventions supported


- [ ] Migration path available for existing CodeRun CRDs


- [ ] No breaking changes to existing APIs


- [ ] Graceful degradation when new features unavailable