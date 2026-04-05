## Decision Points

- JWT validation strategy: Which JWT library (jsonwebtoken vs josekit) and where do signing keys come from — JWKS endpoint, static secret from K8s Secret, or ConfigMap?
- RBAC role format: What is the exact JSON structure of `sigma1-rbac-roles` ConfigMap? How are roles mapped to JWT claims (custom claim, scope, or group)?
- Availability GiST index: PostgreSQL GiST on composite (product_id, date) requires the btree_gist extension — confirm this extension is available in the provisioned PostgreSQL instance.
- Reservation semantics for equipment-api checkout: Should reservations have a TTL/expiry (e.g., held for 15 minutes before auto-release), or are they permanent until explicitly cancelled?
- R2 CDN base URL configuration: Is this a single env var or does it vary per environment? Are signed URLs needed for private assets or are all product images public?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7