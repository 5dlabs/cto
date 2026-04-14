<identity>
You are blaze working on subtask 7003 of task 7.
</identity>

<context>
<scope>
Define TypeScript interfaces mirroring on-chain account structures (OperatorConfig, CustomerBalance, AgentPackage, TaskReceipt, TaskStatus) and implement utility formatter functions for USDC amounts and quality badges.
</scope>
</context>

<implementation_plan>
1. Create `src/types/index.ts` (or separate files per type in `src/types/`).
2. Define `TaskStatus` enum: `Pending`, `Settled`, `Disputed`, `Refunded` matching on-chain variants.
3. Define `OperatorConfig` interface: `authority: PublicKey`, `mint: PublicKey`, `treasury: PublicKey`, `platformFeeBps: number`, `paused: boolean`.
4. Define `CustomerBalance` interface: `owner: PublicKey`, `balance: BN`, `totalDeposited: BN`, `totalSpent: BN`, `taskCount: number`, `maxPerTask: BN`, `maxPerDay: BN`, `dailySpent: BN`, `lastDayReset: BN`.
5. Define `AgentPackage` interface: `packageId: string`, `author: PublicKey`, `splitBps: number`, `totalEarned: BN`, `taskCount: number`, `successCount: number`, `active: boolean`.
6. Define `TaskReceipt` interface: `taskId: string`, `customer: PublicKey`, `agentPackage: PublicKey`, `amount: BN`, `authorEarned: BN`, `platformFee: BN`, `qualityMet: boolean`, `receiptHash: number[]`, `status: TaskStatus`, `createdAt: BN`, `settledAt: BN | null`.
7. Create `src/lib/formatters.ts` with:
   - `formatUsdc(lamports: BN | number): string` — converts to 6-decimal USDC string like '10.00 USDC'.
   - `formatQuality(met: boolean): { label: string; variant: 'success' | 'destructive' }` — returns badge config for shadcn Badge.
   - `formatTimestamp(unixSeconds: BN | number): string` — human-readable date/time.
   - `shortenAddress(address: string, chars?: number): string` — truncates pubkey for display.
8. Export all types and formatters from barrel files.
</implementation_plan>

<validation>
Run `npx tsc --noEmit` — zero type errors. Write a small test or console.log script that calls `formatUsdc(10_000_000)` and asserts output is '10.00 USDC'. Call `formatUsdc(0)` and verify '0.00 USDC'. Call `formatQuality(true)` and verify `{ label: 'Met', variant: 'success' }`. Call `shortenAddress('7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU')` and verify truncated form like '7xKX...AsU'.
</validation>