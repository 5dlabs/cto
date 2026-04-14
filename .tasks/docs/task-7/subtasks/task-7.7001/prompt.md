<identity>
You are blaze working on subtask 7001 of task 7.
</identity>

<context>
<scope>
Scaffold the Next.js 14 application at demo/web/ using create-next-app with App Router, TypeScript, and Tailwind CSS. Install all Solana SDK packages, wallet adapter packages, and initialize shadcn/ui with required components.
</scope>
</context>

<implementation_plan>
1. Run `npx create-next-app@14 demo/web --typescript --tailwind --app --src-dir` to scaffold the project.
2. cd into demo/web/ and install Solana packages: `npm install @coral-xyz/anchor @solana/web3.js @solana/spl-token`.
3. Install wallet adapter packages: `npm install @solana/wallet-adapter-react @solana/wallet-adapter-react-ui @solana/wallet-adapter-phantom @solana/wallet-adapter-solflare @solana/wallet-adapter-base`.
4. Initialize shadcn/ui: `npx shadcn-ui@latest init` — select dark theme, New York style, and configure the components.json.
5. Install required shadcn components one by one: `npx shadcn-ui@latest add button card badge table dialog input tabs separator toast`.
6. Verify `npm run dev` starts without errors and the default Next.js page renders.
7. Ensure tsconfig.json has strict mode enabled and path aliases are configured for `@/` pointing to `src/`.
</implementation_plan>

<validation>
Run `npm run dev` — dev server starts without errors. Run `npm run build` — production build succeeds. Run `npx tsc --noEmit` — zero type errors. Verify node_modules contains @coral-xyz/anchor, @solana/web3.js, @solana/wallet-adapter-react. Verify shadcn components exist in src/components/ui/ (button.tsx, card.tsx, badge.tsx, table.tsx, dialog.tsx, input.tsx, tabs.tsx, separator.tsx, toast.tsx).
</validation>