## Decision Points

- Refund funding model ambiguity: if the treasury wallet has already forwarded settled funds off-chain, can a refund_task instruction drain the program vault? The team must decide whether refunds pull from vault reserves, require a treasury top-up, or mint credit-only (no token transfer). This affects whether the escrow is fully collateralized.
- CRITICAL finding remediation workflow: should CRITICAL findings block the CI/CD devnet deployment (Task 10) automatically, or should they be triaged manually given this is a hackathon demo?

## Coordination Notes

- Agent owner: cipher
- Primary stack: Security Tooling