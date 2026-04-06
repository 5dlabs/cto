## Decision Points

- Stripe client library: the details mention 'stripe 0.17 (async stripe-rust crate or reqwest-based Stripe REST client)' — the team must choose between the async-stripe crate and a manual reqwest client before implementation, as the API surface differs significantly.
- Currency rate provider: 'exchangerate.host or similar free API' is unresolved — a specific provider with its API key management strategy must be selected before the background sync task is implemented.
- SMTP email provider: POST /api/v1/invoices/:id/send mentions 'triggers email via optional SMTP' with no provider or credential strategy specified — the team must decide whether to implement SMTP in v1 or defer entirely to the Morgan integration.
- JWT middleware: JWT validation requires a shared signing key or JWKS endpoint — the key distribution mechanism (shared secret vs. public key from an identity service) must be decided before the middleware is implemented.
- Tax calculation scope: the task states 'full tax engine deferred' but implements CAD=13% HST, USD=0% — the team must confirm whether any other currencies/locales need tax rates for v1 to avoid rework.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7