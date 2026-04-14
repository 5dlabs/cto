<identity>
You are blaze working on subtask 8005 of task 8.
</identity>

<context>
<scope>
Create the receipt detail view that expands when a TaskReceipt row is selected in the Settlement Feed, showing the full receipt hash, on-chain Explorer link, quality_met explanation, all amounts, and transaction metadata.
</scope>
</context>

<implementation_plan>
1. Create `src/components/ReceiptDetail.tsx` as a client component.
2. Accept a `receipt: TaskReceipt | null` prop (selected from SettlementFeed).
3. When receipt is null, show a placeholder: 'Select a settlement to view details'.
4. When a receipt is selected, display in a shadcn `Card` or slide-over panel:
   - **Task ID**: full string (not truncated).
   - **Status**: large Badge with color coding per TaskStatus.
   - **Amount breakdown**: total amount, author earned, platform fee — each formatted with `formatUsdc()` and displayed in a clear breakdown layout.
   - **Quality Assessment**: large badge for qualityMet (green 'Quality Met ✓' / red 'Quality Not Met ✗') with prominent visual styling.
   - **Receipt Hash**: display as hex string (convert `number[]` to hex). Truncated with a 'Copy' button.
   - **Timestamps**: `createdAt` and `settledAt` formatted as human-readable dates.
   - **Customer**: shortened PublicKey with copy button.
   - **Agent Package**: shortened PublicKey with copy button.
   - **On-chain link**: 'View on Explorer →' button that opens the receipt PDA address in a Solana block explorer (construct URL as `https://explorer.solana.com/address/{pda}?cluster=devnet` or configurable).
5. State management: use React state in the parent page or a context to share the selected receipt between SettlementFeed (8001) and ReceiptDetail.
6. Create a shared context or callback pattern: `src/contexts/SelectedReceiptContext.tsx` with provider in page.tsx, consumed by both SettlementFeed and ReceiptDetail.
7. Position the ReceiptDetail component: either as a slide-over/drawer from the right, or as a section below the settlement feed in the left panel, or as a shadcn Dialog modal. Choose the approach that works best for 1920×1080 demo recording (modal is clean and focused).
8. Add a close/dismiss action to deselect the receipt.
</implementation_plan>

<validation>
Click a row in the SettlementFeed DataTable. Verify the ReceiptDetail view opens and shows all fields: Task ID, Status badge, Amount breakdown (total, author earned, platform fee all summing correctly), Quality badge (green or red matching the row's qualityMet), Receipt Hash in hex, timestamps, shortened addresses with copy buttons, and 'View on Explorer' link. Click the Explorer link — verify it opens the correct Solana Explorer URL with the PDA address and ?cluster=devnet. Click 'Copy' on the receipt hash — verify clipboard contains the full hex string. Verify the placeholder message shows when no receipt is selected. Verify dismiss/close returns to the placeholder state.
</validation>