<identity>
You are blaze working on subtask 7004 of task 7.
</identity>

<context>
<scope>
Create the root app layout (src/app/layout.tsx) wrapping the application with WalletProvider, ConnectionProvider, and WalletModalProvider configured for Phantom and Solflare adapters on devnet.
</scope>
</context>

<implementation_plan>
1. Create or update `src/app/layout.tsx`.
2. Import and configure `ConnectionProvider` with endpoint defaulting to `https://api.devnet.solana.com`. Support an environment variable `NEXT_PUBLIC_RPC_ENDPOINT` for override.
3. Import `PhantomWalletAdapter` and `SolflareWalletAdapter` from their respective packages.
4. Create a `wallets` array with instances of both adapters.
5. Wrap the `{children}` with the provider stack in order: `<ConnectionProvider>` → `<WalletProvider wallets={wallets} autoConnect>` → `<WalletModalProvider>` → `{children}`.
6. Create a separate client component `src/components/WalletProviderWrapper.tsx` with 'use client' directive since wallet adapters require client-side rendering. The layout.tsx should use this wrapper.
7. Set `<html lang='en' className='dark'>` and `<body className='bg-solana-dark text-white font-sans min-h-screen'>` with Inter font applied.
8. Import `@solana/wallet-adapter-react-ui/styles.css` for default wallet modal styling.
9. Ensure the wallet modal renders properly and connects to devnet when a wallet is selected.
</implementation_plan>

<validation>
Run `npm run dev` and verify at http://localhost:3000: page loads with dark background. Open browser React DevTools and confirm WalletProvider, ConnectionProvider, and WalletModalProvider are present in the component tree. If Phantom extension is installed, verify auto-connect behavior. If no wallet extension, verify the page still renders without errors (graceful degradation). Check browser console for zero hydration errors.
</validation>