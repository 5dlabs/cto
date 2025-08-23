# Task 16: Controller Template Loading - Tool Usage Guide

## Overview
This guide covers the tools and best practices for implementing agent-specific container script selection in the Rust controller. The implementation requires careful coordination between Rust development tools, template management, and testing infrastructure.

## Required Tools

### 1. Rust Development Stack
**Primary Tools**: `cargo`, `rust-analyzer`, `rustc`

```bash
# Check Rust toolchain
rustup show
cargo --version

# Development workflow
cargo check                    # Fast syntax checking
cargo build                   # Full compilation
cargo test                    # Run test suite
cargo clippy                  # Linting
cargo fmt                     # Code formatting
```

**Best Practices**:
- Use `cargo watch` for continuous development feedback
- Enable `rust-analyzer` in your editor for real-time error detection
- Configure clippy with project-specific lints in `Cargo.toml`

### 2. Template Management
**Primary Tools**: `handlebars-cli`, `bash`, `shellcheck`

```bash
# Validate Handlebars templates
handlebars-cli compile templates/container-rex.sh.hbs

# Validate shell script syntax
bash -n templates/container-rex.sh.hbs
shellcheck templates/container-rex.sh.hbs

# Template development workflow
find templates/ -name "*.hbs" -exec bash -n {} \;
```

**Template Organization**:
```
templates/
├── container-rex.sh.hbs      # Implementation workflow (Rex/Blaze)
├── container-cleo.sh.hbs     # Code quality workflow
├── container-tess.sh.hbs     # Testing workflow
└── shared/
    ├── common-setup.hbs      # Shared setup logic
    └── error-handling.hbs    # Common error handling
```

### 3. Testing and Validation
**Primary Tools**: `cargo test`, `cargo bench`, `integration-test-runner`

```bash
# Unit testing
cargo test --lib tasks::code::templates

# Integration testing
cargo test --test template_integration

# Performance benchmarking
cargo bench --bench template_loading

# Test coverage
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

**Testing Strategy**:
- **Unit Tests**: Template mapping logic, agent name extraction
- **Integration Tests**: End-to-end template selection and loading
- **Performance Tests**: Template caching and lookup speed
- **Regression Tests**: Backward compatibility verification

### 4. File System Operations
**Primary Tools**: `find`, `ls`, `stat`, `inotify-tools`

```bash
# Template discovery
find templates/ -name "*.hbs" -type f

# Template validation pipeline
for template in templates/*.hbs; do
    echo "Validating $template"
    bash -n "$template" || echo "Syntax error in $template"
done

# Monitor template changes during development
inotifywait -m -r templates/ -e modify,create,delete
```

### 5. Version Control Integration
**Primary Tools**: `git`, `pre-commit`, `git-hooks`

```bash
# Pre-commit template validation
#!/bin/bash
# .git/hooks/pre-commit
find templates/ -name "*.hbs" -exec bash -n {} \; || exit 1
cargo test --lib tasks::code::templates || exit 1
```

## Development Workflow

### Phase 1: Setup and Planning
```bash
# 1. Create feature branch
git checkout -b feature/controller-template-loading

# 2. Set up development environment
cargo install cargo-watch cargo-expand cargo-llvm-cov

# 3. Create basic structure
mkdir -p controller/src/tasks/code/
touch controller/src/tasks/code/templates.rs
```

### Phase 2: Core Implementation
```bash
# 1. Continuous development with cargo-watch
cargo watch -x "check --package controller"

# 2. Iterative development cycle
# - Write code
# - Run cargo check for immediate feedback
# - Run cargo test for validation
# - Use cargo clippy for code quality

# 3. Template creation and validation
for agent in rex cleo tess; do
    cp templates/container-base.sh.hbs templates/container-$agent.sh.hbs
    # Customize each template for specific agent workflow
done
```

### Phase 3: Testing and Validation
```bash
# 1. Comprehensive test execution
cargo test --package controller --lib tasks::code::templates -- --nocapture

# 2. Integration testing
cargo test --test template_integration --features integration-tests

# 3. Performance benchmarking
cargo bench --bench template_loading

# 4. Template syntax validation
bash -c 'for f in templates/*.hbs; do bash -n "$f" || exit 1; done'
```

### Phase 4: Quality Assurance
```bash
# 1. Code formatting and linting
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings

# 2. Documentation generation
cargo doc --no-deps --package controller

# 3. Coverage analysis
cargo llvm-cov --html --output-dir coverage/

# 4. Security audit
cargo audit
```

## Common Issues and Solutions

### Issue 1: Template Loading Failures
**Symptoms**: File not found errors, template compilation failures

**Diagnosis**:
```bash
# Check template files exist
ls -la templates/container-*.hbs

# Validate template syntax
bash -n templates/container-rex.sh.hbs

# Check file permissions
stat templates/container-rex.sh.hbs
```

**Solutions**:
- Ensure templates directory is in correct location
- Verify file permissions allow reading
- Validate Handlebars syntax with `handlebars-cli`

### Issue 2: Agent Name Extraction Issues
**Symptoms**: Wrong templates selected, fallback usage

**Diagnosis**:
```bash
# Test agent name extraction logic
cargo test test_agent_name_extraction -- --nocapture

# Debug with println! statements
RUST_LOG=debug cargo test
```

**Solutions**:
- Add comprehensive test cases for various input formats
- Implement robust string parsing with proper error handling
- Add logging for debugging agent name extraction

### Issue 3: Performance Bottlenecks
**Symptoms**: Slow template loading, high memory usage

**Diagnosis**:
```bash
# Profile template loading performance
cargo bench --bench template_loading

# Memory usage analysis
valgrind --tool=massif cargo test --release
```

**Solutions**:
- Implement template caching with `lazy_static` or `once_cell`
- Use string interning for template names
- Profile and optimize hot paths

### Issue 4: Integration Problems
**Symptoms**: Existing workflows break, compilation failures

**Diagnosis**:
```bash
# Check backward compatibility
cargo test --all

# Verify no breaking changes
git diff HEAD~1 --name-only | xargs cargo check
```

**Solutions**:
- Maintain existing API compatibility
- Use feature flags for gradual rollout
- Implement adapter patterns for legacy code

## Best Practices

### Code Organization
```rust
// templates.rs structure
mod agent_mapper;      // Core mapping logic
mod template_loader;   // File loading utilities
mod cache;            // Performance optimizations
mod error_types;      // Custom error definitions

pub use agent_mapper::AgentTemplateMapper;
pub use template_loader::load_agent_template;
```

### Error Handling
```rust
// Use structured errors with context
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Agent '{agent}' not found")]
    AgentNotFound { agent: String },

    #[error("Template '{template}' failed to load: {source}")]
    LoadError { template: String, source: std::io::Error },
}
```

### Testing Structure
```rust
// Comprehensive test organization
#[cfg(test)]
mod tests {
    mod unit {
        // Fast, isolated tests
        mod agent_mapping;
        mod name_extraction;
        mod error_handling;
    }

    mod integration {
        // End-to-end workflow tests
        mod template_loading;
        mod performance;
    }
}
```

### Performance Monitoring
```bash
# Set up continuous performance monitoring
cargo install flamegraph
cargo bench --bench template_loading
flamegraph -o template-loading.svg cargo bench
```

## Troubleshooting Checklist

### Pre-Development
- [ ] Rust toolchain is up to date
- [ ] All required dependencies are installed
- [ ] Development environment is configured
- [ ] Template files are present and valid

### During Development
- [ ] Code passes `cargo check` continuously
- [ ] Unit tests pass after each change
- [ ] Templates validate with `bash -n`
- [ ] No clippy warnings or errors

### Pre-Commit
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] No linting issues (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Performance benchmarks are stable

### Post-Integration
- [ ] Integration tests pass
- [ ] Backward compatibility maintained
- [ ] Performance metrics within acceptable ranges
- [ ] Error handling works as expected
