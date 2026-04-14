# Cross-Chain Development

## Overview
Patterns and protocols for building cross-chain applications — bridging assets, passing messages, and deploying contracts across multiple blockchains.

## Bridging Protocols

### Circle CCTP (USDC Native Bridge)
The gold standard for bridging USDC — burn on source, mint on destination. No wrapped assets.

**Supported Chains:** Ethereum, Base, Arbitrum, Optimism, Polygon, Avalanche, Solana, Noble (Cosmos)

**Flow:**
1. Approve USDC to TokenMessenger contract
2. Call `depositForBurn(amount, destinationDomain, mintRecipient, usdc)`
3. Poll Circle Attestation API for signed attestation
4. Call `receiveMessage(message, attestation)` on destination MessageTransmitter

**Domain IDs:** Ethereum=0, Avalanche=1, Optimism=2, Arbitrum=3, Base=6, Polygon=7, Solana=5

```solidity
// Source chain
IERC20(usdc).approve(address(tokenMessenger), amount);
tokenMessenger.depositForBurn(amount, destinationDomain, bytes32(uint256(uint160(recipient))), usdc);
```

```typescript
// Attestation polling
const response = await fetch(`https://iris-api.circle.com/attestations/${messageHash}`);
const { attestation } = await response.json();
// Then call receiveMessage on destination
```

### Wormhole
General-purpose cross-chain messaging + token bridge. Supports 30+ chains.

**SDK:** `@wormhole-foundation/sdk`

```typescript
import { wormhole } from "@wormhole-foundation/sdk";
import evm from "@wormhole-foundation/sdk/evm";
import solana from "@wormhole-foundation/sdk/solana";

const wh = await wormhole("Mainnet", [evm, solana]);
const srcChain = wh.getChain("Ethereum");
const dstChain = wh.getChain("Solana");

// Token transfer
const xfer = await wh.tokenTransfer(
  { chain: "Ethereum", address: tokenAddress },
  amount,
  srcAddress,
  dstAddress,
  false // not automatic
);
```

### LayerZero v2
Messaging protocol with OApp (messaging) and OFT (fungible token) patterns.

**OApp Pattern (Cross-chain messaging):**
```solidity
import { OApp } from "@layerzerolabs/oapp-evm/contracts/oapp/OApp.sol";

contract MyOApp is OApp {
    function send(uint32 _dstEid, bytes calldata _payload, bytes calldata _options) external payable {
        _lzSend(_dstEid, _payload, _options, MessagingFee(msg.value, 0), payable(msg.sender));
    }

    function _lzReceive(Origin calldata, bytes32, bytes calldata _payload, address, bytes calldata) internal override {
        // handle incoming message
    }
}
```

**OFT Pattern (Cross-chain token):**
```solidity
import { OFT } from "@layerzerolabs/oft-evm/contracts/OFT.sol";

contract MyToken is OFT {
    constructor(address _lzEndpoint, address _delegate)
        OFT("MyToken", "MTK", _lzEndpoint, _delegate) {
        _mint(msg.sender, 1_000_000e18);
    }
}
```

### Axelar
General Message Passing (GMP) across EVM + Cosmos chains.

```solidity
import { AxelarExecutable } from "@axelar-network/axelar-gmp-sdk-solidity/contracts/executable/AxelarExecutable.sol";

contract MyApp is AxelarExecutable {
    function sendMessage(string calldata destChain, string calldata destAddress, bytes calldata payload) external payable {
        gateway.callContract(destChain, destAddress, payload);
    }

    function _execute(string calldata sourceChain, string calldata sourceAddress, bytes calldata payload) internal override {
        // handle
    }
}
```

## Cross-Chain Design Patterns

### Message Verification
- Always verify the source chain and sender address
- Use nonces or message IDs for replay protection
- Implement timeouts — messages can be delayed

### Asset Bridging Strategy
| Asset | Best Bridge | Why |
|-------|------------|-----|
| USDC | CCTP | Native mint/burn, no wrapped tokens |
| ETH | Native bridges (L2→L1) | Canonical, trust-minimized |
| ERC-20 | Wormhole or LayerZero OFT | Wide chain support |
| NFTs | Wormhole NFT bridge | Established standard |
| Arbitrary data | LayerZero or Wormhole | Message passing |

### Multi-Chain Deployment
```bash
# Deploy same contract to multiple chains
for CHAIN in base arbitrum optimism; do
  forge script script/Deploy.s.sol \
    --rpc-url "${CHAIN}_RPC_URL" \
    --broadcast --verify
done
```

**Address consistency:** Use CREATE2 or deterministic deployment proxies for same address across chains.

### Security Considerations

1. **Source verification** — Always check message origin (chain + address)
2. **Replay protection** — Track processed message IDs
3. **Rate limiting** — Cap bridge volume per time window
4. **Timeout handling** — Messages can fail or be delayed; implement retry/refund
5. **Oracle trust** — Understand the trust model (Wormhole guardians, LayerZero DVNs, etc.)
6. **Finality** — Wait for sufficient confirmations before acting on cross-chain messages
7. **Partial failures** — Design for atomic execution or implement recovery mechanisms
