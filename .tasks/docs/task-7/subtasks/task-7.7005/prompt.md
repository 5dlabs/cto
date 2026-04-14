<identity>
You are blaze working on subtask 7005 of task 7.
</identity>

<context>
<scope>
Create the Header component displaying the CTO/5D Labs logo on the left, a 'Devnet' network indicator badge in the center area, and the WalletMultiButton on the right with a solana-green glow effect when connected.
</scope>
</context>

<implementation_plan>
1. Create `src/components/Header.tsx` as a client component ('use client').
2. Layout: flex row with items-center, justify-between. Full width with horizontal padding and a subtle bottom border (border-solana-card or similar).
3. Left section: CTO / 5D Labs logo. Use a text-based logo for now (e.g., styled 'CTO' in solana-purple with 'Settlement Protocol' subtitle), or an SVG if available.
4. Center section: shadcn `Badge` component showing 'Devnet' with `bg-solana-purple/20 text-solana-purple border-solana-purple` styling.
5. Right section: `<WalletMultiButton />` from `@solana/wallet-adapter-react-ui`. Apply custom className to style it with Solana colors.
6. Use the `useWallet()` hook to detect `connected` state. When connected, add a CSS glow effect (box-shadow with solana-green #14F195) around the wallet button or add a green dot indicator.
7. Apply responsive styles: on mobile (<768px), reduce padding and potentially hide the network badge or shrink the logo text.
8. Export the Header component and import it in the layout or page as appropriate (likely in layout.tsx below the providers or in page.tsx).
</implementation_plan>

<validation>
Visually verify at http://localhost:3000: (1) Header spans full width with CTO logo on left, Devnet badge in center area, wallet button on right. (2) 'Devnet' badge is visible with purple styling. (3) Clicking the wallet button opens the wallet selection modal. (4) After connecting with Phantom on devnet, verify the green glow effect appears. (5) On 375px viewport, header elements remain accessible and don't overflow.
</validation>