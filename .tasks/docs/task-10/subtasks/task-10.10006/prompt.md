<identity>
You are atlas working on subtask 10006 of task 10.
</identity>

<context>
<scope>
Optimize CI caching for Rust target directory, Bun node_modules, and Solana CLI binaries to keep pipeline under 10 minutes. Configure Anchor.toml devnet program ID section. Write docs/ci-cd.md with pipeline overview and branch protection recommendations.
</scope>
</context>

<implementation_plan>
1. **Caching optimization** — Review all CI jobs and add/optimize caching:
   - Rust target dir: `actions/cache@v4` with key `rust-${{ hashFiles('**/Cargo.lock') }}`. Restore key: `rust-`.
   - Bun node_modules: cache key `bun-${{ hashFiles('**/bun.lockb') }}` for each package directory.
   - Solana CLI binaries: cache `~/.local/share/solana/` with key `solana-v1.18.26`.
   - Anchor CLI: cache `~/.avm/` with key `anchor-v0.30.1`.
   - Measure cache hit rates and ensure cold-start CI completes under 15 minutes, warm-start under 10 minutes.
2. **Anchor.toml configuration**:
   - Add `[programs.devnet]` section with the deployed program ID.
   - Structure: `cto_billing = "<PROGRAM_ID>"` under `[programs.devnet]`.
   - Add `[provider]` section with `cluster = "localnet"` as default, `wallet = "~/.config/solana/id.json"`.
   - Ensure `[scripts]` section has `test = "bun test"` or equivalent.
3. **Documentation** — Create `docs/ci-cd.md` with:
   - Pipeline architecture diagram (text-based: Stage 1 → Stage 2 → Stage 3 → Stage 4).
   - Job descriptions and their purposes.
   - Required secrets list: `OPERATOR_KEYPAIR` (format, how to generate), `SOLANA_DEVNET_RPC_URL`.
   - How to trigger manual deployment.
   - Branch protection recommendations: require CI pass on PRs, protect main branch, require at least 1 approval.
   - Local development: reference Makefile targets.
   - Troubleshooting: common CI failures (airdrop flaky, Solana CLI version mismatch, cache invalidation).
4. Verify overall pipeline timing: trigger full CI run and measure total wall-clock time. If over 15 minutes, identify bottleneck and optimize (e.g., pre-built Docker image with Solana toolchain, parallel test sharding).
</implementation_plan>

<validation>
CI pipeline with caching completes under 15 minutes on cold start, under 10 minutes on warm cache (second run). Cache hit is logged for Rust target, Bun modules, Solana CLI, and Anchor CLI. Anchor.toml has valid `[programs.devnet]` section. docs/ci-cd.md exists with pipeline overview, secrets documentation, branch protection recommendations, and troubleshooting section. Document is at least 500 words.
</validation>