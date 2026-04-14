# EVM Smart Contract Development

## Overview
Comprehensive skill for developing, testing, deploying, and auditing smart contracts on Ethereum and EVM-compatible chains (Base, Arbitrum, Optimism, Polygon, Avalanche, BSC).

## Frameworks

### Foundry (Preferred)
```bash
forge init my-project
forge install OpenZeppelin/openzeppelin-contracts
forge build
forge test -vvv
forge test --fuzz-runs 10000          # fuzz testing
forge script script/Deploy.s.sol --rpc-url $RPC --broadcast --verify
```

**Project Structure:**
```
src/           # Contracts
test/          # Tests (*.t.sol)
script/        # Deploy scripts (*.s.sol)
lib/           # Dependencies (git submodules)
foundry.toml   # Config
```

**Testing Patterns:**
```solidity
import {Test, console2} from "forge-std/Test.sol";

contract MyTest is Test {
    function setUp() public {
        // deploy contracts, set state
    }

    function test_basicFlow() public {
        // unit test
    }

    function testFuzz_deposit(uint256 amount) public {
        amount = bound(amount, 1e6, 1e24);  // constrain fuzz input
        // fuzz test
    }

    function invariant_totalSupply() public view {
        // invariant test — called after random sequences
    }
}
```

**Useful Cast Commands:**
```bash
cast call $CONTRACT "balanceOf(address)" $ADDR --rpc-url $RPC
cast send $CONTRACT "transfer(address,uint256)" $TO $AMT --rpc-url $RPC --private-key $KEY
cast sig "transfer(address,uint256)"    # get function selector
cast abi-decode "balanceOf(address)(uint256)" $DATA
cast estimate $CONTRACT "mint(uint256)" 100 --rpc-url $RPC
```

### Hardhat (Legacy / JS-heavy projects)
```bash
npx hardhat init
npx hardhat compile
npx hardhat test
npx hardhat run scripts/deploy.ts --network base
```

## Solidity Patterns

### Access Control
```solidity
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
// Ownable for simple admin, AccessControl for role-based
```

### Upgradeable Contracts
```solidity
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

contract MyContract is Initializable, UUPSUpgradeable {
    function initialize(address admin) public initializer {
        __UUPSUpgradeable_init();
        // no constructor — use initializer
    }
    function _authorizeUpgrade(address) internal override onlyOwner {}
}
```

### Token Standards
| Standard | Use Case | OpenZeppelin Base |
|----------|----------|-------------------|
| ERC-20 | Fungible tokens | `ERC20.sol` |
| ERC-721 | NFTs (unique) | `ERC721.sol` |
| ERC-1155 | Multi-token (fungible + NFT) | `ERC1155.sol` |
| ERC-4626 | Tokenized vaults | `ERC4626.sol` |
| ERC-2612 | Gasless approvals (permit) | `ERC20Permit.sol` |

### DeFi Integrations

**Uniswap v3 Swap:**
```solidity
ISwapRouter.ExactInputSingleParams memory params = ISwapRouter.ExactInputSingleParams({
    tokenIn: tokenA,
    tokenOut: tokenB,
    fee: 3000,  // 0.3%
    recipient: msg.sender,
    deadline: block.timestamp + 300,
    amountIn: amount,
    amountOutMinimum: calculateMinOut(amount, slippageBps),
    sqrtPriceLimitX96: 0
});
router.exactInputSingle(params);
```

**Aave v3 Supply:**
```solidity
IERC20(asset).approve(address(pool), amount);
pool.supply(asset, amount, onBehalfOf, 0);
```

**Chainlink Price Feed:**
```solidity
AggregatorV3Interface feed = AggregatorV3Interface(feedAddress);
(, int256 price,,,) = feed.latestRoundData();
require(price > 0, "invalid price");
```

## Security Checklist

1. **Reentrancy** — Use `ReentrancyGuard`, follow checks-effects-interactions
2. **Access control** — Verify msg.sender, never tx.origin
3. **Integer safety** — Solidity 0.8+ has built-in overflow checks; use unchecked only when safe
4. **External calls** — Validate return values, use SafeERC20
5. **Flash loan resistance** — Don't rely on balances/prices in single tx without TWAPs
6. **Front-running** — Use commit-reveal, deadline params, slippage protection
7. **Upgrade safety** — No storage layout collisions, initializer guards
8. **Event emission** — Emit events for all state changes (critical for indexers)

## Gas Optimization

- Pack storage variables into 32-byte slots
- Use `calldata` over `memory` for read-only external params
- Prefer custom errors (`error InsufficientBalance()`) over require strings
- Use `immutable` for constructor-set values, `constant` for compile-time
- Batch operations where possible
- Minimize cold SLOAD — cache in memory variables

## Chain-Specific Notes

| Chain | Block Time | Gas Token | Key RPCs |
|-------|-----------|-----------|----------|
| Ethereum | 12s | ETH | Alchemy, Infura |
| Base | 2s | ETH | Alchemy, QuickNode |
| Arbitrum | 250ms | ETH | Alchemy, QuickNode |
| Optimism | 2s | ETH | Alchemy, QuickNode |
| Polygon | 2s | MATIC/POL | Alchemy, QuickNode |
| Avalanche | 2s | AVAX | Public RPC |
| BSC | 3s | BNB | Public RPC |
