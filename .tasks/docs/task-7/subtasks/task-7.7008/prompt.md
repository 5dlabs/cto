<identity>
You are blaze working on subtask 7008 of task 7.
</identity>

<context>
<scope>
Create the useCustomerBalance() hook that fetches and parses the CustomerBalance PDA account data for a given wallet, and the useOperatorConfig() hook that fetches the OperatorConfig account returning mint, treasury, and paused state.
</scope>
</context>

<implementation_plan>
1. Create `src/hooks/useCustomerBalance.ts`:
   - Accept a `wallet: PublicKey | null` parameter.
   - Use `useProgram()` to get the Anchor Program instance.
   - Use `usePdaDerivation()` to derive the CustomerBalance PDA for the given wallet.
   - Fetch the account using `program.account.customerBalance.fetchNullable(pda)` (returns null if account doesn't exist).
   - Return an object with: `{ data: CustomerBalance | null, loading: boolean, error: Error | null, refetch: () => void }`.
   - Use `useState` and `useEffect` for the async fetch, or consider `useCallback` for the refetch function.
   - Handle the case where wallet is null (return null data, no fetch).
   - Handle the case where the PDA account does not exist (new user — return null data gracefully).
2. Create `src/hooks/useOperatorConfig.ts`:
   - Use `useProgram()` and `usePdaDerivation().deriveOperatorConfig()` to get the PDA.
   - Fetch using `program.account.operatorConfig.fetchNullable(pda)`.
   - Return `{ data: OperatorConfig | null, loading: boolean, error: Error | null, refetch: () => void }`.
   - This hook takes no parameters since there's a single OperatorConfig.
3. Both hooks should re-fetch when the program instance changes (wallet reconnect, etc.).
4. Export both hooks from `src/hooks/index.ts`.
</implementation_plan>

<validation>
Connect Phantom wallet on devnet. Call useCustomerBalance(wallet.publicKey) — if no on-chain account exists yet, verify it returns `{ data: null, loading: false, error: null }` without throwing. Call useOperatorConfig() — verify it returns either valid parsed data or null without errors. Verify that passing null wallet to useCustomerBalance returns null data immediately. Verify refetch() triggers a new fetch by checking loading state transitions. Run `npx tsc --noEmit` — zero type errors.
</validation>