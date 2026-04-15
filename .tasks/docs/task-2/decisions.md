## Decision Points

- Whether daily spending cap resets should use slot-based timing (~216,000 slots/day) or Clock sysvar unix_timestamp-based timing — slot times can vary under network congestion, timestamps are more calendar-intuitive but introduce Clock sysvar dependency
- Whether the USDC vault should be a single shared PDA-owned token account (seeds: [b"vault"]) or per-customer vault accounts — single vault is simpler but creates a larger attack surface; per-customer is more isolated but increases account overhead

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Anchor