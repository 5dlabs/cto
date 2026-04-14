## Decision Points

- Redis client library choice: deadpool-redis (connection pooling built-in) vs redis-rs with manual async connection management. Deadpool adds a dependency but simplifies lifecycle in the reconciliation loop.
- S3 client approach: reuse an existing S3/object-store client from the controller codebase, or add a lightweight dependency (aws-sdk-s3 or rust-s3). If no existing client exists, this is a new dependency decision.
- Billing rate model: hardcoded tier rates (standard=$0.75, high-mem=$1.50, GPU=$3.00) vs configurable rates in SolanaBillingConfig. Hardcoded is simpler for hackathon but less flexible.
- Quality determination: how exactly to derive quality_met from Tess/Cipher/Stitch attestations — are these annotations on the CodeRun CR, status conditions, or separate CRs? This affects the reconciliation loop integration.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum