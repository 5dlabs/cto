# Model Catalog Sync

This workflow ingests provider model metadata from `mmmodels` and generates
stable artifacts consumed by CTO infra and web APIs.

## Run

```bash
./scripts/sync-model-catalog.sh
```

Optional provider scope:

```bash
MODEL_SYNC_PROVIDERS=anthropic,openai ./scripts/sync-model-catalog.sh
```

## Generated Artifacts

- `infra/model-catalog/normalized-model-catalog.json`
  - Shared normalized catalog (`schemaVersion`, `generatedAt`, providers/models)
- `infra/charts/openclaw-agent/files/model-catalog.generated.json`
  - OpenClaw-focused provider model map consumed by chart templates
- `apps/web/src/generated/model-catalog.json`
  - Provider -> model-id list used by web key-validation API responses

## Stability Rules

- Deterministic provider ordering (alphabetical)
- Deterministic model ordering (alphabetical by id)
- Per-provider model dedupe by model id, preferring latest `lastUpdated`
- Stable JSON formatting (`indent=2`, sorted keys)

## Overlay Controls (Helm)

`infra/charts/openclaw-agent/values.yaml` supports catalog controls:

- `models.catalog.enabled`
- `models.catalog.generatedPath`
- `models.catalog.allowProviders`
- `models.catalog.denyModels`

These let operators pin provider scope and block known-bad model ids while
still consuming market-fed catalog updates.
