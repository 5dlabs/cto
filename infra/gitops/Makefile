# Makefile for GitOps testing and validation

.PHONY: help test validate lint security diff preview clean install-tools

# Default target
help: ## Show this help message
	@echo "GitOps Testing Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

test: validate ## Run all tests
	@echo "✅ All tests completed"

validate: lint structure security ## Run all validation checks
	@echo "🔍 Running all validation checks..."

lint: ## Lint YAML files
	@echo "📝 Linting YAML files..."
	@if command -v yamllint >/dev/null 2>&1; then \
		yamllint -c ../../.yamllint.yaml . || exit 1; \
	else \
		echo "⚠️  yamllint not installed. Run: make install-tools"; \
	fi

structure: ## Validate ArgoCD application structure
	@echo "🏗️  Validating application structure..."
	@for file in applications/*.yaml; do \
		[ -f "$$file" ] || continue; \
		grep -q "kind: Application" "$$file" || (echo "❌ Not an Application: $$file" && exit 1); \
		grep -q "spec:" "$$file" || (echo "❌ Missing spec: $$file" && exit 1); \
		grep -q "destination:" "$$file" || (echo "❌ Missing destination: $$file" && exit 1); \
		grep -q "source:" "$$file" || (echo "❌ Missing source: $$file" && exit 1); \
	done
	@echo "✅ Structure validation passed"

security: ## Run security checks
	@echo "🔒 Running security checks..."
	@if grep -r "password:" . --include="*.yaml" | grep -v "secretKeyRef" | grep -v "CHANGE_THIS" | grep -v "examples" | grep -v "{{" > /dev/null; then \
		echo "❌ Found potential hardcoded passwords"; \
		exit 1; \
	fi
	@echo "✅ Security checks passed"

diff: ## Show diff against main branch
	@echo "📊 Showing diff against main branch..."
	@git diff origin/main...HEAD -- .

preview: ## Preview what would be deployed
	@echo "👁️  Preview mode..."
	@echo "Applications to be deployed:"
	@ls -1 applications/*.yaml 2>/dev/null | sed 's/applications\//  - /g' | sed 's/\.yaml//g'
	@echo ""
	@echo "Database resources to be deployed:"
	@ls -1 databases/*.yaml 2>/dev/null | grep -v examples | grep -v README | sed 's/databases\//  - /g' | sed 's/\.yaml//g'

dry-run: ## Dry run with ArgoCD (requires ArgoCD CLI)
	@echo "🧪 Dry run with ArgoCD..."
	@if command -v argocd >/dev/null 2>&1; then \
		for app in applications/*.yaml; do \
			[ -f "$$app" ] || continue; \
			echo "Dry run for $$app:"; \
			kubectl apply --dry-run=client -f "$$app" || exit 1; \
		done; \
	else \
		echo "⚠️  argocd CLI not installed"; \
	fi

install-tools: ## Install required testing tools
	@echo "📦 Installing testing tools..."
	@echo "Installing yamllint..."
	@pip install --user yamllint || echo "Failed to install yamllint"
	@echo ""
	@echo "Installing pre-commit..."
	@pip install --user pre-commit || echo "Failed to install pre-commit"
	@echo ""
	@echo "Installing yq..."
	@if [ "$$(uname)" = "Darwin" ]; then \
		brew install yq || echo "Failed to install yq"; \
	else \
		curl -fsSL https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -o /tmp/yq && \
		chmod +x /tmp/yq && \
		sudo mv /tmp/yq /usr/local/bin/yq || echo "Failed to install yq"; \
	fi
	@echo ""
	@echo "Installing kubeconform..."
	@if [ "$$(uname)" = "Darwin" ]; then \
		brew install kubeconform || echo "Failed to install kubeconform"; \
	else \
		curl -fsSL https://github.com/yannh/kubeconform/releases/latest/download/kubeconform-linux-amd64.tar.gz | \
		tar xz && sudo mv kubeconform /usr/local/bin || echo "Failed to install kubeconform"; \
	fi
	@echo ""
	@echo "✅ Tool installation complete"

pre-commit-install: ## Install pre-commit hooks
	@echo "🪝 Installing pre-commit hooks..."
	@pre-commit install
	@echo "✅ Pre-commit hooks installed"

pre-commit-run: ## Run pre-commit on all files
	@echo "🏃 Running pre-commit on all files..."
	@pre-commit run --all-files

clean: ## Clean up temporary files
	@echo "🧹 Cleaning up..."
	@find . -name "*.tmp" -delete
	@find . -name "*.bak" -delete
	@echo "✅ Cleanup complete"

# Quick test for CI
ci-test: lint structure security ## Run CI tests
	@echo "✅ CI tests passed"
