# Solana DeFi

Expertise in Solana DeFi protocols, token standards, and ecosystem integrations.

## Token Standards

### SPL Token (Original)
- Standard fungible token program: `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`
- Associated Token Account (ATA) program for deterministic token accounts

### Token-2022 (Token Extensions)
- Extended token program: `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`
- Extensions: transfer fees, confidential transfers, transfer hooks, metadata, non-transferable, interest-bearing, permanent delegate, CPI guard, group/member pointers
- Use when you need any extension feature; otherwise use standard SPL Token

### Metaplex Standards
- **Token Metadata** — NFT/SFT metadata (name, symbol, URI, creators, royalties)
- **Bubblegum** — compressed NFTs (cNFTs) using Merkle trees
- **Core** — next-gen asset standard (simpler, cheaper than Token Metadata)

## Major DeFi Protocols

### Jupiter (Aggregator)
- **Swap API**: `https://quote-api.jup.ag/v6/quote` → `https://quote-api.jup.ag/v6/swap`
- **DCA**: Dollar-cost averaging program
- **Limit Orders**: On-chain limit order book
- **Perps**: Perpetual futures trading
```typescript
// Jupiter swap example
const quote = await fetch(
  `https://quote-api.jup.ag/v6/quote?inputMint=${inputMint}&outputMint=${outputMint}&amount=${amount}&slippageBps=50`
).then(r => r.json());

const swap = await fetch('https://quote-api.jup.ag/v6/swap', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ quoteResponse: quote, userPublicKey: wallet.publicKey.toString() })
}).then(r => r.json());
```

### Raydium
- **AMM v4** — Constant product AMM (legacy)
- **CLMM** — Concentrated liquidity (like Uniswap v3)
- **CPMM** — New constant product with OpenBook integration
- **AcceleRaytor** — Launchpad

### Orca
- **Whirlpools** — Concentrated liquidity pools
- **Whirlpools SDK** (`@orca-so/whirlpools-sdk`) for programmatic interaction

### Marinade Finance
- **mSOL** — Liquid staking token
- Native staking, delayed unstake, instant unstake via liquidity pool

### Jito
- **JitoSOL** — MEV-powered liquid staking
- **Jito Tips** — priority transaction bundles for MEV protection
- **Jito Bundles** — atomic transaction bundles

### Kamino Finance
- **Lending/Borrowing** — Solana lending protocol
- **Liquidity Vaults** — Automated CLMM management
- **Multiply/Long/Short** — Leveraged strategies

### Drift Protocol
- **Perps** — Decentralized perpetual futures
- **Spot** — Spot trading with margin
- **Borrow/Lend** — Interest rate markets

### Tensor
- **NFT marketplace** — trading, listings, bids
- **cNFT support** — compressed NFT trading

## Common Patterns

### Token Swap Integration
```typescript
import { Jupiter } from '@jup-ag/core';

const jupiter = await Jupiter.load({ connection, cluster: 'mainnet-beta', user: wallet });
const routes = await jupiter.computeRoutes({ inputMint, outputMint, amount, slippageBps: 50 });
const { execute } = await jupiter.exchange({ routeInfo: routes.routesInfos[0] });
const result = await execute();
```

### Creating a Token (Token-2022 with metadata)
```typescript
import { createMint, createInitializeMetadataPointerInstruction } from '@solana/spl-token';

// Token-2022 mint with embedded metadata
const mint = await createMint(
  connection, payer, mintAuthority, freezeAuthority, decimals,
  undefined, undefined, TOKEN_2022_PROGRAM_ID
);
```

### Reading DeFi State
```typescript
// Fetch Raydium pool state
const poolInfo = await Raydium.load({ connection, owner: wallet });
const pools = await poolInfo.api.fetchPoolByMints({ mint1, mint2 });

// Fetch Orca whirlpool
const whirlpool = await fetcher.getPool(whirlpoolAddress);
const price = PriceMath.sqrtPriceX64ToPrice(whirlpool.sqrtPrice, decimalsA, decimalsB);
```

## Solana Program Library (SPL) Programs
| Program | Address | Purpose |
|---------|---------|---------|
| Token | `TokenkegQfeZy...` | Fungible/NFT tokens |
| Token-2022 | `TokenzQdBNbLq...` | Extended token features |
| Associated Token | `ATokenGPvbdGV...` | Deterministic token accounts |
| Memo | `MemoSq4gqABAX...` | Attach memos to transactions |
| Name Service | `namesLPneVptA...` | .sol domain names (Bonfida) |
| Stake Pool | `SPoo1Ku8WFXoN...` | Stake pool management |

## MEV & Transaction Optimization
- Use **Jito bundles** for atomic multi-instruction transactions
- Set **compute unit price** for priority (`ComputeBudgetProgram.setComputeUnitPrice`)
- Set **compute unit limit** to actual usage (`ComputeBudgetProgram.setComputeUnitLimit`)
- Use **address lookup tables** (ALTs) for transactions with many accounts
- **Preflight simulation** before sending to catch errors early

## Security Considerations for DeFi
- Always verify token mint addresses against known registries
- Check for **rug pull indicators**: freeze authority, mint authority still active
- Use **slippage protection** on all swaps
- Verify **oracle prices** aren't stale (check `lastUpdatedSlot`)
- Watch for **sandwich attacks** — use Jito bundles or private mempools
- Validate **decimals** when converting amounts (SOL = 9, USDC = 6, etc.)
