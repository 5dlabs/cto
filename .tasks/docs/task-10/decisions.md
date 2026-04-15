## Decision Points

- Operator keypair secret format: should the CI secret store the keypair as a base58 string, a JSON byte array, or a base64-encoded file? This affects how the deployment step decodes it and must align with how the keypair was generated in Task 2.

## Coordination Notes

- Agent owner: atlas
- Primary stack: CI/CD Platforms