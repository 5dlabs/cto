# Rust Coding Guidelines

This document provides coding standards and best practices for Rust development in this project.

## Pre-PR Quality Gates (MANDATORY)

Before opening any pull request or requesting merge:

- Ensure formatting passes:
  - Run: `cargo fmt --all -- --check`
- Ensure Clippy passes with pedantic lints and no warnings:
  - Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
  - If a pedantic lint must be allowed, use the narrowest scope with `#[allow(clippy::lint_name)]` and include a short justification above the code. Avoid crate-wide allows.
- Ensure tests pass and coverage is very high (strive for ~100% on critical code paths):
  - Run: `cargo test --workspace --all-features`
  - Recommended coverage tools:
    - If available: `cargo llvm-cov --workspace --all-features --fail-under-lines 95`
    - Alternatively: `cargo tarpaulin --all --fail-under 95`
- Do not create a PR until all gates above are green locally.

## Code Quality Standards

### Live Data Implementation (MANDATORY)
- **NO MOCK DATA**: Always implement with real data sources and live APIs
- **Parameterized Configuration**: Hard-coded values are prohibited - use configuration
- **Environment-driven**: Support multiple environments (dev, staging, prod) via config
- **Runtime Configurable**: Business logic parameters must be externally configurable

#### Configuration Architecture:
```rust
// ✅ DO: Layered configuration with defaults
#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub api_endpoints: ApiEndpoints,
    pub trading_config: TradingConfig,
    pub feature_flags: FeatureFlags,
}

#[derive(Deserialize, Debug)]
pub struct TradingConfig {
    pub supported_pairs: Vec<String>,  // From config, not hard-coded
    pub price_precision: u8,
    pub min_trade_amount: Decimal,
    pub rate_limits: RateLimits,
}

// ✅ DO: Load from multiple sources
impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", env::var("ENV")?)))
            .add_source(Environment::with_prefix("APP"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

#### Database Integration:
```rust
// ❌ NEVER: Mock database or hardcoded data
fn get_users() -> Vec<User> {
    vec![User { id: 1, name: "test".to_string() }]  // WRONG
}

// ✅ DO: Real database operations with proper error handling
pub async fn get_users(pool: &PgPool) -> Result<Vec<User>, DatabaseError> {
    let users = sqlx::query_as!(User, "SELECT id, name FROM users WHERE active = true")
        .fetch_all(pool)
        .await?;
    Ok(users)
}
```

#### API Integration:
```rust
// ❌ NEVER: Mock external API responses
async fn get_market_price() -> f64 { 100.0 }  // WRONG

// ✅ DO: Real API integration with proper clients
pub struct MarketDataClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl MarketDataClient {
    pub async fn get_price(&self, pair: &str) -> Result<Decimal, ApiError> {
        let response = self.client
            .get(&format!("{}/price/{}", self.base_url, pair))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .send()
            .await?;
        
        let price_data: PriceResponse = response.json().await?;
        Ok(price_data.price)
    }
}
```

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `anyhow::Result` for application-level errors
- Use `thiserror` for library-level custom errors
- Always handle errors explicitly - avoid `unwrap()` in production code
- Use `?` operator for error propagation
- Provide meaningful error messages with context

### Memory Management
- Prefer owned types (`String`, `Vec<T>`) over borrowed types for struct fields
- Use `Cow<str>` when you need flexibility between owned and borrowed strings
- Minimize `clone()` calls - consider borrowing or moving when possible
- Use `Arc<T>` for shared immutable data across threads
- Use `Rc<T>` for shared data within single-threaded contexts

### Async Programming
- Use `async`/`await` for I/O-bound operations
- Use `tokio` runtime for async execution
- Prefer `async fn` over `impl Future`
- Use `tokio::spawn` for concurrent tasks
- Handle cancellation with `tokio::select!` when appropriate

## Code Organization

### Module Structure
```rust
// Public API at the top
pub use self::public_types::*;

// Private modules
mod private_implementation;
mod public_types;

// Re-exports for convenience
pub mod prelude {
    pub use super::{PublicType, PublicTrait};
}
```

### Naming Conventions
- Use `snake_case` for variables, functions, and modules
- Use `PascalCase` for types, traits, and enum variants
- Use `SCREAMING_SNAKE_CASE` for constants
- Use descriptive names - avoid abbreviations
- Prefix boolean functions with `is_`, `has_`, or `can_`

### Documentation
- Document all public APIs with `///` comments
- Include examples in documentation when helpful
- Use `//!` for module-level documentation
- Keep documentation up-to-date with code changes

## Performance Guidelines

### Allocations
- Minimize heap allocations in hot paths
- Use `Vec::with_capacity()` when size is known
- Consider `SmallVec` for collections that are usually small
- Use string formatting (`format!`) judiciously

### Collections
- Use `HashMap` for general key-value storage
- Use `BTreeMap` when ordering matters
- Use `HashSet` for unique values
- Use `VecDeque` for FIFO/LIFO operations

### Iterators
- Prefer iterator chains over explicit loops when readable
- Use `collect()` only when necessary
- Consider `fold()` and `reduce()` for aggregations
- Use `Iterator::find()` instead of filtering then taking first

## Testing Guidelines

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Given
        let input = setup_test_data();

        // When
        let result = function_under_test(input);

        // Then
        assert_eq!(result, expected_value);
    }

    #[test]
    #[should_panic(expected = "specific error message")]
    fn test_error_conditions() {
        // Test error conditions
    }
}
```

### Integration Tests
- Place integration tests in `tests/` directory
- Test public API only
- Use realistic data and scenarios
- Test error conditions and edge cases

## Security Guidelines

### Input Validation
- Validate all external input
- Use type-safe parsing (`str::parse()`)
- Sanitize data before storage or transmission
- Use prepared statements for database queries

### Secrets Management
- Never hardcode secrets in source code
- Use environment variables for configuration
- Use secure random number generation (`rand::thread_rng()`)
- Clear sensitive data from memory when possible

## Rust-Specific Best Practices

### Pattern Matching
```rust
// Prefer exhaustive matching
match value {
    Some(x) => handle_some(x),
    None => handle_none(),
}

// Use if-let for single pattern
if let Some(value) = optional_value {
    process_value(value);
}
```

### Ownership
- Pass by reference (`&T`) for read-only access
- Pass by mutable reference (`&mut T`) for modification
- Pass by value (`T`) for ownership transfer
- Use `Clone` when multiple ownership is needed

### Traits
- Implement common traits (`Debug`, `Clone`, `PartialEq`)
- Use trait bounds instead of concrete types in generics
- Prefer composition over inheritance (use traits)

## Service Architecture Guidelines

### Project Structure
```
src/
├── bin/           # Binary targets
├── lib.rs         # Library root
├── config/        # Configuration management
├── handlers/      # Request handlers
├── models/        # Data models
├── services/      # Business logic
└── utils/         # Utility functions
```

### Configuration
- Use `serde` for configuration deserialization
- Support both file-based and environment-based config
- Provide sensible defaults
- Validate configuration on startup

### Logging
- Use `tracing` for structured logging
- Include relevant context in log messages
- Use appropriate log levels (error, warn, info, debug, trace)
- Avoid logging sensitive information

## Common Patterns

### Builder Pattern
```rust
pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { host: None, port: None }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn build(self) -> Result<Config> {
        Ok(Config {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(8080),
        })
    }
}
```

### Resource Management
```rust
// Use RAII for resource cleanup
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub fn new(url: &str) -> Result<Self> {
        let connection = DatabaseConnection::open(url)?;
        Ok(Self { connection })
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // Cleanup happens automatically
        self.connection.close();
    }
}
```

Remember: These guidelines promote code that is safe, performant, and maintainable. When in doubt, choose clarity over cleverness.

## Documentation-Driven Implementation

When implementing or modifying code covered by these guidelines and when an internal document server is available:

- Always query the document server for the recommended, best-practice approach before significant implementation work.
- Prefer patterns and examples from the document server to reduce rework and testing iteration.
- If a divergence from the recommended approach is necessary, document the rationale in the PR description and in code comments above the relevant implementation.
- Re-check the document server for updates when addressing review feedback or refactoring.