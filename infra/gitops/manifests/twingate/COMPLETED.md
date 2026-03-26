# Twingate Migration Notes (Historical)

This file is a historical record of the initial Twingate rollout.
For current operations, use [`SETUP.md`](./SETUP.md) and [`README.md`](./README.md).

## What changed in March 2026

- Removed broken/non-existent CRD usage (`TwingateRemoteNetwork`) from GitOps manifests.
- Standardized manifests on `twingate.com/v1beta` `TwingateResource` and `TwingateResourceAccess`.
- Added Argo CD `ignoreDifferences` for normalized fields in `twingateresourceaccesses.twingate.com`.
- Kept connector deployment separate as Helm app `twingate-connector` in namespace `cto`.

## Current steady state

- Remote network identity is configured in the operator app Helm values.
- Access policy is represented by `*-access.yaml` bindings in this directory.
- Connector auth comes from `cto/twingate-connector-tokens` populated by External Secrets.

## Operational reminder

Do not store real connector/API tokens in repository docs. Keep credentials only in 1Password and OpenBao.
