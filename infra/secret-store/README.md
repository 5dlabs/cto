# Secret Store

This directory contains the namespace configuration for platform secrets.

## Migration to Vault Secrets Operator (VSO)

**As of November 2025, secrets are managed by HashiCorp Vault Secrets Operator (VSO).**

The previous External Secrets Operator (ESO) configuration has been removed. VaultStaticSecret resources are now located in `infra/vault/secrets/`.

## Current Configuration

- `namespace-and-rbac.yaml` - Creates the `secret-store` namespace used by some VaultStaticSecrets

## Where Secrets Are Defined

All secret definitions are now in:

```
infra/vault/
├── vault-connection.yaml    # Connection to Vault server
├── vault-auth.yaml          # Kubernetes authentication config
├── secrets/                 # VaultStaticSecret resources
│   ├── api-keys.yaml
│   ├── github-apps.yaml
│   ├── infrastructure.yaml
│   ├── doc-server.yaml
│   ├── ghcr.yaml
│   └── toolman.yaml
└── README.md                # Full documentation
```

## Adding New Secrets

1. Add the secret in Vault UI at `secret/<path>`
2. Create a VaultStaticSecret in `infra/vault/secrets/`
3. Commit and push - ArgoCD will sync automatically

See `infra/vault/README.md` for complete documentation.
