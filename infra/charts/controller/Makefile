# Makefile for controller Helm chart

.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: generate-templates
generate-templates: ## Generate split ConfigMaps from agent-templates directory
	@echo "Generating agent templates ConfigMaps..."
	@./scripts/generate-agent-templates-configmaps-split.sh

.PHONY: lint
lint: ## Run helm lint on the chart
	@echo "Linting Helm chart..."
	@helm lint .

.PHONY: template
template: ## Generate Kubernetes manifests from the chart
	@echo "Generating Kubernetes manifests..."
	@helm template controller . --values values.yaml

.PHONY: package
package: ## Package the Helm chart
	@echo "Packaging Helm chart..."
	@helm package .

.PHONY: update-templates
update-templates: generate-templates ## Update templates and prepare for commit
	@echo "Templates updated. Don't forget to commit the changes!"
	@git status templates/agent-templates-*.yaml

.PHONY: validate
validate: lint template ## Validate the chart (lint + template generation)
	@echo "✅ Chart validation complete!"

# Development targets
.PHONY: dev-deploy
dev-deploy: generate-templates ## Deploy to local development cluster
	@echo "Deploying to local cluster..."
	@helm upgrade --install controller . \
		--namespace agent-platform \
		--create-namespace \
		--values values.yaml

.PHONY: dev-delete
dev-delete: ## Delete from local development cluster
	@echo "Deleting from local cluster..."
	@helm delete controller --namespace agent-platform || true
