# Runtime Image Rust Setup Issues

## Issue Description
The agents are burning tokens setting up Rust toolchain every time they work on Rust projects. This is inefficient and costly.



## Observed Behavior
When agents try to run `cargo init`, they get:








```
error: rustup could not choose a version of cargo to run, because one wasn't specified explicitly, and no default is configured.
help: run 'rustup default stable' to download the latest stable release of Rust and set it as your default toolchain.








```

The agent then has to:


1. Run `rustup default stable`


2. Download and install entire Rust toolchain (cargo, clippy, rust-docs, rust-std, rustc, rustfmt)


3. Then proceed with actual work

This happens **every time** a new CodeRun is created for Rust tasks.



## Impact
- **Token waste**: Agents spend tokens figuring out Rust setup instead of doing actual work
- **Time waste**: Extra setup time for every Rust-related task
- **Inconsistent experience**: Should be ready to go immediately



## Root Cause
The runtime image doesn't have Rust toolchain pre-installed or pre-configured.



## Suggested Fix
Update the runtime image to include:


- Pre-installed Rust stable toolchain


- Pre-configured default toolchain (`rustup default stable` already run)


- Common Rust tools (cargo, clippy, rustfmt) ready to use



## Example Log
From workflow `play-task-1-p58n6`:








```
{"type":"assistant","message":{"id":"msg_01WvuTQ77LM4oKDvDqR5EbKX","type":"message","role":"assistant","model":"claude-sonnet-4-20250514","content":[{"type":"text","text":"Let me set up Rust first:"}],"stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":5,"cache_creation_input_tokens":196,"cache_read_input_tokens":30577,"cache_creation":{"ephemeral_5m_input_tokens":196,"ephemeral_1h_input_tokens":0},"output_tokens":1,"service_tier":"standard"}},"parent_tool_use_id":null,"session_id":"7926f2d9-39a0-41fd-8d85-7535d3b363bc"}








```



## Priority
Medium - affects token efficiency but doesn't block functionality



## Status
Logged - needs runtime image improvement



## Date
2025-08-20
