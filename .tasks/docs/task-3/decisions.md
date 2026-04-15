## Decision Points

- Refund fund flow: should refund_task transfer USDC from the treasury token account back to the vault, or should a dedicated refund reserve account be maintained separately? Treasury-to-vault is simpler but requires the treasury to hold sufficient balance; a refund reserve adds an account but isolates funds.
- receipt_hash population in settle_task: should receipt_hash be passed as an instruction argument (computed off-chain from the Arweave receipt) or should the on-chain program compute it from instruction data? Passing it as an argument is simpler but trusts the caller; computing on-chain is more trustless but limited by what data is available in the instruction context.
- Overflow-safe split arithmetic: should the author_share calculation use u128 intermediate values (amount as u128 * split_bps as u128 / 10000) or strictly use checked_mul/checked_div on u64 with explicit overflow error handling? u128 intermediate eliminates overflow risk for any valid u64 amount; checked u64 is more idiomatic but requires careful ordering.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Anchor