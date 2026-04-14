## Decision Points

- Whether to use Anchor Optional<> accounts, remaining accounts pattern, or two separate instruction variants (settle_task_default vs settle_task_with_package) for handling the optional agent_package/author_ata — impacts client-side complexity and IDL shape

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Anchor/Solana