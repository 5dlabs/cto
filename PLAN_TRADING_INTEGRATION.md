# Trader Agent - Integration Plan

## Overview
Build OpenClaw plugin to execute Solana trades via Jupiter API, with Birdeye for analytics and Helios RPC.

## Skills Discovery (2026-02-05)
- ✅ `solana-trading` - Already installed in `~/.openclaw/skills/solana-trading/`
- 🔍 ClawHub.ai - JavaScript rendered, no specific crypto skills found
- ✅ OpenClaw docs reviewed - Plugin architecture confirmed as best approach

## Plugin Build Status
- ✅ **Phase 1: Plugin Scaffold** - COMPLETE
  - ✅ Created `~/.openclaw/extensions/trading-executor/`
  - ✅ `openclaw.plugin.json` - Manifest with config schema
  - ✅ `package.json` - Dependencies (@solana/web3.js, @solana/spl-token, bs58)
  - ✅ `index.ts` - Plugin entry point with all tools

## Installed Tools (6)
| Tool | Purpose |
|------|---------|
| `jupiter_quote` | Get swap quote from Jupiter |
| `jupiter_swap` | Execute swap (requires confirm) |
| `wallet_balance` | Check SOL and token balances |
| `birdeye_token` | Get token metrics |
| `birdeye_trending` | Get trending tokens |
| `birdeye_new_pairs` | Monitor new token pairs |

## RPC Methods
| Method | Purpose |
|--------|---------|
| `trading.status` | Check plugin status |

## CLI Commands
| Command | Purpose |
|---------|---------|
| `trading-status` | Show plugin status |
| `trading-balance <wallet>` | Check wallet balance |
| `trading-quote <input> <output> <amount>` | Get Jupiter quote |

## Next Steps
- [ ] **Phase 2: Wallet Integration - COMPLETE**
  - ✅ Added 1Password wallet loading (op:// references supported)
  - ✅ Added transaction signing capability
  - ✅ Implemented `jupiter_swap` with actual transaction execution
  - ✅ Added transaction confirmation and status
- [ ] Phase 3: Test with dry-run mode
- [ ] Phase 4: Configure OpenClaw to load the plugin

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  OPENCLAW PLUGIN (trading-executor)                   │
│  ├─ Agent Tools (for Trader agent in chat)            │
│  │   ├─ jupiter_quote(inputMint, outputMint, amount)  │
│  │   ├─ jupiter_swap(quoteResponse, userPublicKey)    │
│  │   ├─ wallet_balance(publicKey)                      │
│  │   ├─ birdeye_token(address)                         │
│  │   ├─ birdeye_trending()                             │
│  │   └─ birdeye_new_pairs()                           │
│  ├─ Gateway RPC Methods                                │
│  │   ├─ trading.quote()                               │
│  │   ├─ trading.execute()                             │
│  │   └─ trading.status()                              │
│  └─ CLI Commands (backup manual control)              │
└─────────────────┬───────────────────────────────────────┘
                  │
                  │ 1Password (wallet private key)
                  ▼
┌─────────────────────────────────────────────────────────┐
│  EXECUTION LAYER                                        │
│  ├─ Jupiter Swap API (https://api.jup.ag)             │
│  ├─ Helios RPC (Solana blockchain)                     │
│  └─ Birdeye API (analytics)                            │
└─────────────────────────────────────────────────────────┘
```

## Tasks

### Phase 1: Plugin Scaffold
- [ ] Create `~/.openclaw/extensions/trading-executor/`
- [ ] Create `openclaw.plugin.json` manifest
- [ ] Set up `package.json` with dependencies (@solana/web3.js, @solana/spl-token)
- [ ] Create plugin entry point (index.ts)

### Phase 2: Wallet Integration
- [ ] Create 1Password reference for wallet key
- [ ] Implement wallet load function
- [ ] Add balance checking functionality
- [ ] Test wallet connectivity

### Phase 3: Jupiter Integration
- [ ] Implement `jupiter_quote()` tool
- [ ] Implement `jupiter_swap()` tool
- [ ] Add slippage parameter support
- [ ] Add transaction signing
- [ ] Test with testnet/dry-run first

### Phase 4: Birdeye Integration
- [ ] Implement `birdeye_token()` tool
- [ ] Implement `birdeye_trending()` tool
- [ ] Implement `birdeye_new_pairs()` tool
- [ ] Add to agent tools

### Phase 5: Gateway RPC & CLI
- [ ] Register `trading.quote` RPC method
- [ ] Register `trading.execute` RPC method
- [ ] Register `trading.status` RPC method
- [ ] Add CLI commands for manual operations

### Phase 6: Testing & Documentation
- [ ] Dry-run trade test
- [ ] Write usage documentation
- [ ] Add safety checks (max slippage, confirmation)
- [ ] Document emergency stop procedures

## Skills to Add
- [ ] Review ClawHub for relevant skills
- [ ] Install any additional useful skills found

## Dependencies
- BIRDEYE_API_KEY (in .env or 1Password)
- HELIOS_RPC_URL (in .env)
- TRADER_WALLET (1Password reference)
- JUPITER_FEE_ACCOUNT (optional)

## Safety
- Always require confirmation for real trades
- Max slippage: configurable (default 5%)
- Dry-run mode for testing
- Emergency stop command

---
*Created: 2026-02-05*
*Last Updated: 2026-02-05*
