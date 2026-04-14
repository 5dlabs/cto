## Hackathon Demo Web Dashboard - Core Setup & Wallet Integration (Blaze - React/Next.js/TypeScript)

### Objective
Set up the Next.js 14 web dashboard with Solana wallet adapter integration, dark theme with Solana brand colors, shadcn/ui components, and the split-panel layout skeleton. This is the visual foundation for the hackathon demo (PRD Section 11, Deliverable #3).

### Ownership
- Agent: blaze
- Stack: React/Next.js 14/TypeScript/Tailwind CSS
- Priority: medium
- Status: pending
- Dependencies: 1, 2

### Implementation Details
1. Initialize Next.js 14 app at `demo/web/` with App Router, TypeScript, Tailwind CSS:
   ```bash
   npx create-next-app@14 demo/web --typescript --tailwind --app --src-dir
   ```

2. Install dependencies:
   - `@coral-xyz/anchor`, `@solana/web3.js`, `@solana/spl-token`
   - `@solana/wallet-adapter-react`, `@solana/wallet-adapter-react-ui`, `@solana/wallet-adapter-phantom`, `@solana/wallet-adapter-solflare` (D10)
   - `shadcn/ui` via `npx shadcn-ui@latest init` — select dark theme, New York style (D14)
   - Install shadcn components: `button`, `card`, `badge`, `table`, `dialog`, `input`, `tabs`, `separator`, `toast`

3. Configure Tailwind theme in `tailwind.config.ts`:
   - Dark mode: `class` strategy, default to dark.
   - Custom colors: `solana-purple: '#9945FF'`, `solana-green: '#14F195'`, `solana-dark: '#0E0E1A'`, `solana-card: '#1A1A2E'`.
   - Accent colors mapped to shadcn CSS variables in `globals.css`.

4. Create app layout (`src/app/layout.tsx`):
   - `<WalletProvider>` wrapping the app with Phantom + Solflare adapters.
   - `<ConnectionProvider>` with configurable endpoint (default devnet).
   - `<WalletModalProvider>` for the connect dialog.
   - Dark background (`bg-solana-dark`), Inter font.

5. Create header component (`src/components/Header.tsx`):
   - CTO / 5D Labs logo (left).
   - Network indicator badge: 'Devnet' in solana-purple.
   - Wallet connect button (right) using `<WalletMultiButton />` from wallet-adapter-react-ui.
   - Solana-green glow effect on connected state.

6. Create split-panel layout (D12) in `src/app/page.tsx`:
   - Left panel (60% width): Settlement feed area (placeholder for Task 8).
   - Right panel (40% width): Customer balance area (placeholder for Task 8).
   - Responsive: stack vertically on mobile, side-by-side on desktop.
   - Optimize for 1920×1080 screen recording.

7. Create shared hooks (`src/hooks/`):
   - `useProgram()` — returns Anchor Program instance initialized with connected wallet and IDL. Import IDL from `../../target/idl/cto_billing.json` (or copy to `src/idl/`).
   - `usePdaDerivation()` — PDA derivation helpers using SHA-256 (D2): `deriveCustomerBalance(wallet)`, `deriveAgentPackage(packageId)`, `deriveTaskReceipt(taskId)`, `deriveVault(mint)`, `deriveOperatorConfig()`.
   - `useCustomerBalance(wallet)` — fetches CustomerBalance PDA account data, returns parsed balance/caps/stats.
   - `useOperatorConfig()` — fetches OperatorConfig, returns mint/treasury/paused state.

8. Create types (`src/types/`):
   - Mirror on-chain types: OperatorConfig, CustomerBalance, AgentPackage, TaskReceipt, TaskStatus.
   - Utility formatters: `formatUsdc(lamports: number)` → '10.00 USDC', `formatQuality(met: boolean)` → badge JSX.

9. Create a minimal 'Not Connected' state for the main page that shows the CTO branding and prompts wallet connection.

10. Verify the app runs with `npm run dev` and displays correctly in dark mode with Solana branding at 1920×1080.

### Subtasks
- [ ] Initialize Next.js 14 project and install all dependencies: Scaffold the Next.js 14 application at demo/web/ using create-next-app with App Router, TypeScript, and Tailwind CSS. Install all Solana SDK packages, wallet adapter packages, and initialize shadcn/ui with required components.
- [ ] Configure Tailwind theme with Solana brand colors and dark mode: Set up the custom Tailwind CSS theme with Solana-specific brand colors, dark mode class strategy, Inter font, and map accent colors to shadcn CSS variables in globals.css.
- [ ] Create TypeScript type definitions and utility formatters: Define TypeScript interfaces mirroring on-chain account structures (OperatorConfig, CustomerBalance, AgentPackage, TaskReceipt, TaskStatus) and implement utility formatter functions for USDC amounts and quality badges.
- [ ] Integrate Solana wallet providers in app layout: Create the root app layout (src/app/layout.tsx) wrapping the application with WalletProvider, ConnectionProvider, and WalletModalProvider configured for Phantom and Solflare adapters on devnet.
- [ ] Build Header component with logo, network badge, and wallet button: Create the Header component displaying the CTO/5D Labs logo on the left, a 'Devnet' network indicator badge in the center area, and the WalletMultiButton on the right with a solana-green glow effect when connected.
- [ ] Create split-panel responsive layout with 'Not Connected' state: Implement the main page split-panel layout with 60/40 width ratio for settlement feed and customer balance areas, responsive stacking on mobile, optimized for 1920×1080 screen recording, and a 'Not Connected' fallback state.
- [ ] Implement useProgram() and usePdaDerivation() hooks: Create the useProgram() hook that returns an initialized Anchor Program instance using the connected wallet and IDL, and the usePdaDerivation() hook providing PDA derivation helpers using SHA-256 for all program accounts.
- [ ] Implement useCustomerBalance() and useOperatorConfig() data-fetching hooks: Create the useCustomerBalance() hook that fetches and parses the CustomerBalance PDA account data for a given wallet, and the useOperatorConfig() hook that fetches the OperatorConfig account returning mint, treasury, and paused state.