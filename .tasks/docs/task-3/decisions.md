## Decision Points

- Lead scoring algorithm: What specific criteria and thresholds determine GREEN/YELLOW/RED scoring? The PRD mentions 'vetting data + opportunity value' but the exact formula, weight of each factor, and threshold values (e.g., is $5000 the confirmed cutoff?) need to be defined by the product owner.
- Google Calendar integration: Which Google Cloud project and service account should be used? Should it use a service account with domain-wide delegation or OAuth2 per-user consent flow? This affects how calendar events are owned and visible.
- JWT service token validation: Where does the JWT signing key come from — a shared secret in a Kubernetes secret, a JWKS endpoint from an identity provider, or the auth service from task dependencies? The RBAC ConfigMap format also needs to be agreed upon across services.
- Inventory barcode format: What barcode standard is used (Code128, QR, etc.) and what encoding scheme maps barcodes to inventory_item_ids? Is this a simple lookup or does the barcode encode metadata like store location?

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go 1.22+/gRPC