# Rust Error Handling

Best practices for error handling in Rust applications.

## When to Use

- Building Rust applications and libraries
- Designing error types for the Rex agent
- Implementing robust error propagation
- Creating user-friendly error messages

## Core Concepts

### Result and Option

```rust
// Result for operations that can fail
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// Option for values that might not exist
fn find_user(id: u64) -> Option<User> {
    users.get(&id).cloned()
}
```

### The `?` Operator

```rust
// Propagate errors concisely
fn process_config() -> Result<Config, Error> {
    let content = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

## Error Libraries

### anyhow (Applications)

Best for applications where you want easy error handling without defining custom types.

```rust
use anyhow::{Context, Result, bail, anyhow};

fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read config file")?;
    
    let config: Config = serde_json::from_str(&content)
        .context("Failed to parse config")?;
    
    if config.port == 0 {
        bail!("Port cannot be 0");
    }
    
    Ok(config)
}

// With more context
fn process_user(id: u64) -> Result<()> {
    let user = fetch_user(id)
        .with_context(|| format!("Failed to fetch user {}", id))?;
    
    validate_user(&user)
        .with_context(|| format!("User {} validation failed", id))?;
    
    Ok(())
}
```

### thiserror (Libraries)

Best for libraries where you need custom, typed errors.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] serde_json::Error),
    
    #[error("Invalid port: {port}")]
    InvalidPort { port: u16 },
    
    #[error("Missing required field: {0}")]
    MissingField(String),
}

fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?; // auto-converted
    let config: Config = serde_json::from_str(&content)?;
    
    if config.port == 0 {
        return Err(ConfigError::InvalidPort { port: 0 });
    }
    
    Ok(config)
}
```

## Best Practices

### Use Context Liberally

```rust
// BAD: No context
let data = std::fs::read(path)?;

// GOOD: With context
let data = std::fs::read(path)
    .with_context(|| format!("Failed to read {}", path))?;
```

### Define Domain Errors

```rust
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Internal error")]
    Internal(#[source] anyhow::Error),
}

// Convert from anyhow for internal errors
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err)
    }
}
```

### Error Recovery

```rust
// Handle specific errors differently
match result {
    Ok(value) => process(value),
    Err(e) if e.is::<std::io::Error>() => {
        // Retry for IO errors
        retry_operation()
    }
    Err(e) => return Err(e.into()),
}

// With custom errors
match api_call() {
    Ok(data) => Ok(data),
    Err(ApiError::NotFound { .. }) => {
        // Return default for not found
        Ok(Default::default())
    }
    Err(e) => Err(e),
}
```

### Logging Errors

```rust
use tracing::{error, warn, info};

fn process() -> Result<()> {
    match operation() {
        Ok(result) => {
            info!(result = ?result, "Operation succeeded");
            Ok(())
        }
        Err(e) => {
            error!(error = ?e, "Operation failed");
            Err(e)
        }
    }
}

// Or using map_err
operation()
    .map_err(|e| {
        error!(error = ?e, "Operation failed");
        e
    })?;
```

## Patterns

### Early Return

```rust
fn validate(input: &Input) -> Result<()> {
    if input.name.is_empty() {
        return Err(anyhow!("Name cannot be empty"));
    }
    
    if input.age < 18 {
        return Err(anyhow!("Must be 18 or older"));
    }
    
    Ok(())
}
```

### Collect Results

```rust
// Collect all successes, fail on first error
let results: Result<Vec<_>, _> = items
    .iter()
    .map(process_item)
    .collect();

// Collect successes and errors separately
let (successes, errors): (Vec<_>, Vec<_>) = items
    .iter()
    .map(process_item)
    .partition(Result::is_ok);
```

### Custom Error Types with Source

```rust
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    Connection(#[source] sqlx::Error),
    
    #[error("Query failed: {query}")]
    Query {
        query: String,
        #[source]
        source: sqlx::Error,
    },
}
```

## Common Mistakes

| Mistake | Problem | Solution |
|---------|---------|----------|
| Using `.unwrap()` | Panics on error | Use `?` or handle error |
| Generic error messages | Hard to debug | Add context with `.context()` |
| Swallowing errors | Silent failures | Log or propagate |
| Too many error types | Complex to handle | Use `anyhow` for apps |
| No `#[source]` | Lost error chain | Add `#[source]` attribute |

## When to Panic

Panics are appropriate only for:

1. **Programming bugs**: Invariant violations that indicate bugs
2. **Unrecoverable states**: Truly impossible to continue
3. **Tests**: `unwrap()` in tests is acceptable

```rust
// OK to panic: programming error
fn get_item(index: usize) -> &Item {
    &self.items[index] // panics if out of bounds
}

// Better: return Option
fn get_item(index: usize) -> Option<&Item> {
    self.items.get(index)
}
```
