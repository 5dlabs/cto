## Decision Points

- Whether to use the official Solana devnet USDC mint (EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v) or always create a program-controlled mock mint for deterministic testing — affects test reproducibility vs production parity
- Exact Solana validator image version to pin (v1.18.x patch release) — impacts feature compatibility with Anchor 0.30.x and devnet parity

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm