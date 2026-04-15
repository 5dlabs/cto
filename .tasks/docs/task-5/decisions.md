## Decision Points

- Receipt upload invocation method: should the controller shell out to `bun packages/receipt-uploader/scripts/upload.ts` as a subprocess, or should the receipt-uploader be wrapped as a lightweight HTTP sidecar? Subprocess is simpler but has cold-start and error-reporting drawbacks; HTTP sidecar is cleaner for retry/observability but adds deployment complexity.
- Customer wallet mapping storage: should the customer's Solana pubkey be stored as a namespace annotation, a ConfigMap entry, or a field on a CustomerProfile CRD? Namespace annotation is simplest for hackathon but doesn't scale; a CRD field is more principled but requires schema changes.
- Anchor client vs raw RPC: should Solana transactions be constructed using `anchor-client` (type-safe, IDL-driven) or raw `solana-client` with manually-serialized instruction data? Anchor-client adds a heavy dependency tree but is less error-prone.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum