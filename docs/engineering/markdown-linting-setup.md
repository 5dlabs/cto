# Markdown Linting Setup

This document describes the comprehensive Markdown linting setup implemented for the CTO project to ensure consistent, high-quality documentation across all 263+ Markdown files.



## Overview

The project now includes:


- **Pre-commit hooks** for automatic linting on commit


- **GitHub Actions workflow** for CI/CD linting


- **Manual linting script** for local development


- **Comprehensive configuration** with sensible defaults for technical documentation

## Configuration Files

### `.markdownlint.yaml`
The main configuration file that defines linting rules. Key settings include:

- **Line length**: 120 characters (appropriate for technical docs)
- **Code blocks**: Must use fenced style (```)
- **Lists**: Must be surrounded by blank lines
- **Headings**: Must be surrounded by blank lines
- **HTML elements**: Allow common elements like `<kbd>`, `<br>`, `<img>`, etc.

### `.pre-commit-config.yaml`
Updated to include Markdown linting alongside existing YAML and shell script checks.

### `.github/workflows/markdown-lint.yaml`
GitHub Actions workflow that runs on:


- Push to `main` or `develop` branches


- Pull requests to `main` or `develop` branches


- Only when Markdown files are changed



## Usage

### Automatic Linting (Recommended)

The linting runs automatically via pre-commit hooks:




```bash
# Install pre-commit hooks (if not already done)
pre-commit install

# Run on all files
pre-commit run --all-files

# Run only markdownlint
pre-commit run markdownlint --all-files






```

### Manual Linting

Use the provided script for manual linting and fixing:




```bash
# Show linting statistics
./fix-markdown-lint.sh stats

# Run linting only
./fix-markdown-lint.sh lint

# Run linting and attempt automatic fixes
./fix-markdown-lint.sh fix

# Install markdownlint-cli if not present
./fix-markdown-lint.sh install



# Show help
./fix-markdown-lint.sh help






```



### Direct CLI Usage

You can also use markdownlint directly:




```bash
# Install globally
npm install -g markdownlint-cli

# Run on all Markdown files
markdownlint --config .markdownlint.yaml "**/*.md"

# Run on specific files
markdownlint --config .markdownlint.yaml README.md docs/






```

## Common Issues and Fixes

### 1. Line Length (MD013)
**Issue**: Lines longer than 120 characters
**Fix**: Break long lines at appropriate points (spaces, punctuation)

### 2. Missing Blank Lines (MD022, MD031, MD032)
**Issue**: Headings, code blocks, or lists not surrounded by blank lines
**Fix**: Add blank lines before and after these elements

### 3. Trailing Spaces (MD009)
**Issue**: Spaces at the end of lines
**Fix**: Remove trailing spaces

### 4. Multiple Blank Lines (MD012)
**Issue**: More than one consecutive blank line
**Fix**: Reduce to single blank lines

### 5. Emphasis as Headings (MD036)
**Issue**: Using `**text**` instead of `# text` for headings
**Fix**: Use proper heading syntax

## Current Status

As of the initial setup:
- **Total Markdown files**: 263
- **Files with issues**: ~50+ (varies by run)
- **Common issues**: Missing blank lines, long lines, trailing spaces

## Integration with Development Workflow



### Pre-commit
Linting runs automatically before each commit, preventing issues from being committed.



### CI/CD
GitHub Actions runs linting on all PRs and pushes to main branches, ensuring quality gates.

### Local Development
Use the manual script for:


- Checking specific files


- Bulk fixing common issues


- Understanding linting statistics

## Customization

### Modifying Rules
Edit `.markdownlint.yaml` to adjust rules:




```yaml
# Example: Increase line length limit
MD013:
  line_length: 140



# Example: Disable a rule
MD036: false






```

### Adding Exclusions
Add files to exclude in `.markdownlint.yaml`:




```yaml
# Exclude specific files or patterns
exclude:


  - "docs/legacy/*.md"


  - "*.template.md"






```



## Best Practices



### For Developers
1. **Run linting before committing**: Use `pre-commit run --all-files`
2. **Fix issues locally**: Use `./fix-markdown-lint.sh fix` for bulk fixes
3. **Check CI results**: Ensure GitHub Actions passes before merging

### For Documentation Writers
1. **Follow the style guide**: Use consistent formatting
2. **Keep lines under 120 characters**: Break long lines appropriately
3. **Use proper heading hierarchy**: Start with `#` for main headings
4. **Surround elements with blank lines**: Headings, lists, code blocks

### For Maintainers
1. **Review linting configuration**: Periodically update rules as needed
2. **Monitor CI failures**: Address systematic issues
3. **Update documentation**: Keep this guide current

## Troubleshooting

### Common Problems



**Pre-commit fails on markdownlint**



```bash
# Install markdownlint-cli
npm install -g markdownlint-cli

# Reinstall pre-commit hooks
pre-commit install






```



**GitHub Actions fails**


- Check the workflow logs for specific error messages


- Ensure markdownlint-cli is properly installed in the workflow


- Verify the configuration file is valid



**Script permission denied**



```bash
chmod +x fix-markdown-lint.sh






```

### Getting Help

1. **Check the configuration**: Review `.markdownlint.yaml` for rule explanations
2. **Run with verbose output**: `markdownlint --config .markdownlint.yaml --verbose "**/*.md"`
3. **Consult markdownlint documentation**: [https://github.com/DavidAnson/markdownlint](https://github.com/DavidAnson/markdownlint)

## Future Improvements

Potential enhancements to consider:
- **Custom rules**: Project-specific linting rules
- **Integration with editors**: VS Code/Cursor extensions
- **Automated fixing**: More sophisticated auto-fix capabilities
- **Performance optimization**: Parallel processing for large file sets
- **Reporting**: Detailed reports with fix suggestions

## Conclusion

This Markdown linting setup provides a robust foundation for maintaining high-quality documentation across the CTO project. The combination of pre-commit hooks, CI/CD integration, and manual tools ensures that documentation quality is maintained throughout the development process.
