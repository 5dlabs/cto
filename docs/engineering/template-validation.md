# Agent Template Validation

This document describes the template validation system for agent templates in `infra/charts/controller/agent-templates/`.

## Quick Start

```bash
# Full validation (Handlebars + shell + JSON + TOML)
make validate-templates

# Quick syntax check only (faster, for pre-commit)
make validate-templates-quick

# Verbose output with detailed errors
./scripts/validate-templates.sh --verbose
```

## What Gets Validated

### 1. Handlebars Syntax
- Block matching: Ensures every `{{#if}}`, `{{#each}}`, `{{#unless}}` has a matching `{{/if}}`, `{{/each}}`, `{{/unless}}`
- Empty references: Detects empty `{{}}` which would fail at runtime

### 2. Shell Script Syntax (`.sh.hbs` files)
- **Basic syntax**: Uses `bash -n` to validate rendered scripts
- **Shellcheck** (when available): Runs linting with appropriate exclusions for template patterns

### 3. JSON Template Syntax (`.json.hbs` files)
- Validates that rendered output produces valid JSON
- Handles expected template artifacts (trailing commas from conditionals)

### 4. TOML Template Syntax (`.toml.hbs` files)
- Validates basic TOML structure (section headers, key-value pairs)

## CI Integration

The validation is integrated into the build pipeline:

- `make ci` runs quick template validation
- `make pre-commit` runs full template validation

## Adding New Templates

When adding new templates:

1. Follow existing naming conventions:
   - Shell scripts: `container-*.sh.hbs`
   - Config files: `*.json.hbs` or `*.toml.hbs`
   - Agent prompts: `agents-*.md.hbs`

2. Run validation before committing:
   ```bash
   make validate-templates-quick
   ```

3. For complex templates, test with verbose mode:
   ```bash
   ./scripts/validate-templates.sh --verbose
   ```

## Common Issues

### Unmatched Handlebars Blocks
```
✗ template.sh.hbs: Unmatched Handlebars blocks ({{# = 5, {{/ = 4)
```
**Fix**: Check that every `{{#if}}` has a matching `{{/if}}`

### Shell Syntax Errors
```
✗ container.sh.hbs: Shell syntax error
      line 123: syntax error near unexpected token `fi'
```
**Fix**: The rendered script has shell syntax issues. Check:
- Proper `if/then/fi` structure
- Quoted strings with embedded quotes
- Heredoc terminators

### JSON Validation Failures
```
✗ config.json.hbs: Invalid JSON structure
```
**Fix**: Common issues include:
- Trailing commas in arrays/objects
- Missing quotes around string values
- Unescaped special characters

## Rust-Based Testing

For deeper template validation using the actual Handlebars renderer, run:

```bash
cd controller
cargo run --bin test_templates
```

This uses the same Handlebars engine as the controller and tests with realistic mock data.

## Script Location

- Validation script: `scripts/validate-templates.sh`
- Rust test binary: `controller/src/bin/test_templates.rs`












