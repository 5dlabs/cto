# CTO Platform Root Makefile
# Main entry point for common development tasks

.PHONY: help
help: ## Show this help message
	@echo 'CTO Platform - Development Commands'
	@echo '==================================='
	@echo ''
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-25s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ''
	@echo 'E2E Testing targets:'
	@echo '  make e2e-help           Show all E2E testing commands'

# Include E2E testing targets
-include scripts/Makefile.e2e

# Quick shortcuts for common operations
.PHONY: reset
reset: e2e-reset ## Quick full E2E environment reset

.PHONY: status
status: e2e-status ## Check E2E environment status

.PHONY: logs
logs: e2e-logs ## Follow agent workflow logs

# Build targets
.PHONY: build
build: ## Build all project binaries
	@echo "Building controller..."
	@cd controller && cargo build --release --bin agent-controller
	@echo "Building MCP server..."
	@cd mcp && cargo build --release
	@cp mcp/target/release/cto-mcp dist/
	@echo "Build complete!"

.PHONY: test
test: ## Run all tests
	@echo "Running controller tests..."
	@cd controller && cargo test
	@echo "Running MCP tests..."
	@cd mcp && cargo test
	@echo "All tests passed!"

.PHONY: lint
lint: ## Run linting checks
	@echo "Running Rust formatting check..."
	@cargo fmt --all -- --check
	@echo "Running Clippy..."
	@cargo clippy --all-targets -- -D warnings
	@echo "Linting complete!"

.PHONY: clippy-pedantic
clippy-pedantic: ## Run Clippy with pedantic settings
	@echo "Running Clippy pedantic..."
	@cargo clippy --all-targets -- -W clippy::pedantic -D warnings

# Helm chart operations
.PHONY: helm-generate
helm-generate: ## Generate Helm template ConfigMaps
	@cd infra/charts/controller && make generate-templates

.PHONY: helm-lint
helm-lint: ## Lint Helm charts
	@cd infra/charts/controller && make lint

# GitOps operations
.PHONY: gitops-validate
gitops-validate: ## Validate GitOps manifests
	@cd infra/gitops && make validate

.PHONY: gitops-lint
gitops-lint: ## Lint GitOps YAML files
	@cd infra/gitops && make lint

# Development workflow
.PHONY: dev-setup
dev-setup: ## Set up development environment
	@echo "Setting up development environment..."
	@echo "1. Installing Rust dependencies..."
	@cargo fetch
	@echo "2. Building binaries..."
	@make build
	@echo "3. Checking prerequisites..."
	@command -v kubectl >/dev/null 2>&1 || echo "  ⚠️  kubectl not found"
	@command -v gh >/dev/null 2>&1 || echo "  ⚠️  gh CLI not found"
	@command -v helm >/dev/null 2>&1 || echo "  ⚠️  helm not found"
	@echo "Development setup complete!"

.PHONY: clean
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -f dist/cto-mcp
	@echo "Clean complete!"

# Workflow shortcuts
.PHONY: play
play: ## Run CTO play workflow (requires TASK_ID)
	@if [ -z "$(TASK_ID)" ]; then \
		echo "Error: TASK_ID is required"; \
		echo "Usage: make play TASK_ID=1"; \
		exit 1; \
	fi
	@echo "Starting CTO play workflow for task $(TASK_ID)..."
	@cto play --task-id $(TASK_ID)

.PHONY: workflow-status
workflow-status: ## Check workflow status
	@echo "Current workflows in cto namespace:"
	@kubectl get workflows -n cto

.PHONY: workflow-cleanup
workflow-cleanup: ## Clean up all workflows
	@echo "Cleaning up all workflows..."
	@kubectl delete workflows --all -n cto --force --grace-period=0

# Combined operations
.PHONY: fresh-test
fresh-test: reset ## Reset environment and show test command
	@echo ""
	@echo "Environment reset complete!"
	@echo ""
	@echo "To start a test, run:"
	@echo "  make play TASK_ID=1"
	@echo ""
	@echo "To monitor, run:"
	@echo "  make logs"

.PHONY: ci
ci: lint test ## Run CI checks (lint and test)
	@echo "CI checks passed!"

.PHONY: pre-commit
pre-commit: lint clippy-pedantic test ## Run all pre-commit checks
	@echo "Pre-commit checks passed!"

# Documentation
.PHONY: docs
docs: ## Show documentation links
	@echo "CTO Platform Documentation"
	@echo "========================="
	@echo ""
	@echo "📚 Main Documentation:"
	@echo "  - README.md                    - Project overview"
	@echo "  - docs/architecture.md         - System architecture"
	@echo "  - ROADMAP.md                  - Development roadmap"
	@echo ""
	@echo "🔧 E2E Testing:"
	@echo "  - scripts/E2E_RESET_README.md - E2E reset documentation"
	@echo ""
	@echo "🚀 Quick Start:"
	@echo "  1. make dev-setup             - Set up development"
	@echo "  2. make reset                 - Reset E2E environment"
	@echo "  3. make play TASK_ID=1        - Run a test"
	@echo "  4. make logs                  - Monitor execution"

# Default target
.DEFAULT_GOAL := help



