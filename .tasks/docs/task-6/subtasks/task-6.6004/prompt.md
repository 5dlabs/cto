<identity>
You are nova working on subtask 6004 of task 6.
</identity>

<context>
<scope>
Create separate command files in cli/src/commands/ for each Solana program operation, wired up with Commander, so users can invoke operations individually beyond the scripted demo flow.
</scope>
</context>

<implementation_plan>
1. Create `cli/src/index.ts` as the Commander-based CLI entry point:
   ```typescript
   import { program } from 'commander';
   program.name('cto-billing').version('0.1.0');
   // Register subcommands
   import './commands/init-operator';
   import './commands/register-agent';
   import './commands/create-customer';
   import './commands/deposit';
   import './commands/settle';
   import './commands/withdraw';
   import './commands/verify-receipt';
   import './commands/demo';
   program.parse();
   ```
2. Create command files in `cli/src/commands/`:
   - `init-operator.ts`: `cto-billing init-operator --treasury <pubkey> --fee-bps <number> --keypair <path>`
   - `register-agent.ts`: `cto-billing register-agent --package-id <string> --split-bps <number> --source-uri <string> --keypair <path>`
   - `create-customer.ts`: `cto-billing create-customer --max-per-task <number> --max-per-day <number> --keypair <path>`
   - `deposit.ts`: `cto-billing deposit --amount <number> --keypair <path>`
   - `settle.ts`: `cto-billing settle --task-id <string> --amount <number> --receipt <json-or-path> --quality-met --keypair <path>`
   - `withdraw.ts`: `cto-billing withdraw --amount <number> --keypair <path>`
   - `verify-receipt.ts`: `cto-billing verify-receipt --task-id <string>` — fetches on-chain receipt and Arweave data, verifies hash.
   - `demo.ts`: `cto-billing demo` — runs the full demo flow from subtask 6002/6003.
3. Each command should:
   - Accept `--rpc <url>` and `--program-id <pubkey>` global options.
   - Use shared utils from `src/utils/`.
   - Print colored output with explorer links on success.
   - Print descriptive error messages with transaction links on failure.
4. Add to `package.json`: `"bin": { "cto-billing": "src/index.ts" }` and scripts: `"cli": "bun run src/index.ts"`, `"demo": "bun run src/index.ts demo"`.
5. Add `--help` output for each command with examples.
</implementation_plan>

<validation>
Run `bun run cli --help` — prints available commands list without error. Run `bun run cli init-operator --help` — prints usage with all options. Run `bun run cli deposit --amount 10 --keypair ./test-keypair.json --rpc https://api.devnet.solana.com` against devnet (with a pre-created customer account) — deposit succeeds and prints balance. Run each command with `--help` — all print valid usage information. Run `bun run cli demo` — equivalent to `bun run demo`, completes full E2E flow.
</validation>