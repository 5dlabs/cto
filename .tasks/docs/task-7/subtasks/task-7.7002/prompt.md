<identity>
You are blaze working on subtask 7002 of task 7.
</identity>

<context>
<scope>
Set up the custom Tailwind CSS theme with Solana-specific brand colors, dark mode class strategy, Inter font, and map accent colors to shadcn CSS variables in globals.css.
</scope>
</context>

<implementation_plan>
1. Edit `tailwind.config.ts` to set `darkMode: 'class'`.
2. Extend the theme colors with: `'solana-purple': '#9945FF'`, `'solana-green': '#14F195'`, `'solana-dark': '#0E0E1A'`, `'solana-card': '#1A1A2E'`.
3. Edit `src/app/globals.css` to map shadcn CSS variables in the `:root` and `.dark` selectors. Set primary to solana-purple (#9945FF), accent/success to solana-green (#14F195), background to solana-dark (#0E0E1A), card to solana-card (#1A1A2E).
4. Import Inter font from `next/font/google` and apply it to the body.
5. Ensure the `<html>` element has `class='dark'` by default in layout.tsx (just the class, not the full layout — that comes in 7004).
6. Create a test page or component that renders sample text, buttons, and cards using the custom colors to visually verify the theme.
</implementation_plan>

<validation>
Visually verify at http://localhost:3000 that the page background is #0E0E1A (solana-dark). Create a temporary test div with `bg-solana-purple` and `text-solana-green` classes and confirm they render correctly. Verify Inter font loads by inspecting computed styles in browser DevTools. Confirm shadcn Button component renders with solana-purple primary color.
</validation>