## Decision Points

- Kyverno vs Gatekeeper for container image admission policy enforcement — Kyverno is Kubernetes-native with simpler policy authoring, Gatekeeper uses OPA/Rego which is more powerful but complex. Decision affects policy maintenance burden.
- JWT token generation: deploy-time Job vs init container — a dedicated Job runs once and creates all service tokens as Secrets; init containers would generate per-service but add startup latency. Affects deployment workflow.
- GDPR data export/deletion orchestration language — the Job needs to call multiple service APIs and interact with R2. Should it be a Go binary, a shell script, or a lightweight Node/Bun script? Needs to fit the team's operational tooling.
- Cloudflare WAF rate limiting thresholds — 100 req/min per IP is proposed but may need tuning per endpoint (e.g., auth endpoints lower, read-heavy catalog endpoints higher). Requires product/ops input on acceptable limits.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm