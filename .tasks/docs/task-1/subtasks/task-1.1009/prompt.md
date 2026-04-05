Implement subtask 1009: Document all infrastructure endpoints, secrets, and access patterns

## Objective
Create comprehensive documentation of all provisioned infrastructure, including endpoint URLs, secret key names, access patterns, and instructions for downstream service consumption.

## Steps
1. Create a markdown document (e.g., docs/infrastructure.md) listing: all namespaces and their purposes, all deployed services with their internal DNS names and ports, all Secrets with their key names (not values), the sigma1-infra-endpoints ConfigMap with all keys and descriptions, instructions for downstream services to consume the ConfigMap (envFrom example YAML), Signal-CLI registration instructions, Cloudflare Tunnel configuration details. 2. Include a troubleshooting section for common issues (pod not starting, connection refused, etc.). 3. Include a diagram of the namespace topology and service connectivity.

## Validation
Documentation file exists; all ConfigMap keys and Secret names referenced in the doc match what is actually deployed; a new developer can follow the doc to understand how to connect a new service to the infrastructure.