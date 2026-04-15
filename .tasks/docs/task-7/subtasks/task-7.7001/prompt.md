<identity>
You are blaze working on subtask 7001 of task 7.
</identity>

<context>
<scope>
Initialize the Next.js 14 project with App Router, install all Solana and UI dependencies, configure SolanaProvider with wallet adapters, create the useProgram hook for Anchor IDL integration, implement PDA derivation utilities, and set up environment configuration.
</scope>
</context>

<implementation_plan>
1. Run `npx create-next-app@latest --typescript --tailwind --app --src-dir` in `app/dashboard/`.
2. Install dependencies: `@coral-xyz/anchor` (0.30.x), `@solana/web3.js` (^1.95), `@solana/wallet-adapter-react` (^0.15), `@solana/wallet-adapter-wallets` (^0.19), `@solana/wallet-adapter-react-ui` (^0.9), `@tanstack/react-query` (^5), `lucide-react`, `recharts`.
3. Create `src/providers/SolanaProvider.tsx`: wrap WalletProvider with Phantom, Solflare, Backpack adapters. Include WalletModalProvider. Support network toggle (devnet/mainnet) via env var.
4. Create `src/providers/QueryProvider.tsx`: wrap TanStack QueryClientProvider.
5. Create `src/hooks/useProgram.ts`: initialize Anchor program from IDL (import from `target/idl/cto_billing.json`). Return the program instance and connection.
6. Create `src/lib/pda.ts`: PDA derivation functions for all account types (operatorConfig, customerBalance, taskReceipt, agentPackage) using program ID and seeds from the Anchor program.
7. Create `src/lib/constants.ts`: USDC mint address (devnet), program ID from env.
8. Configure `.env.local` with `NEXT_PUBLIC_SOLANA_RPC_URL`, `NEXT_PUBLIC_PROGRAM_ID`, `NEXT_PUBLIC_SOLANA_NETWORK=devnet`.
9. Configure `next.config.js` with `output: 'export'` for static hosting.
10. Wire providers in `src/app/layout.tsx`: SolanaProvider → QueryProvider → children.
</implementation_plan>

<validation>
Run `bun run dev` — app starts without errors. Import useProgram in a test page and verify it returns a valid Anchor Program instance when wallet is connected. Verify PDA derivation functions produce deterministic addresses matching the Anchor program's PDA seeds. Environment variables are accessible via `process.env.NEXT_PUBLIC_*`.
</validation>