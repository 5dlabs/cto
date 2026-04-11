# Entry Point Analyzer

Systematically identify all **state-changing** entry points in a smart contract codebase to guide security audits.

## When to Use

Use this skill when:

- Starting a smart contract security audit to map the attack surface
- Asked to find entry points, external functions, or audit flows
- Analyzing access control patterns across a codebase
- Identifying privileged operations and role-restricted functions
- Building an understanding of which functions can modify contract state

## When NOT to Use

Do NOT use this skill for:

- Vulnerability detection (use audit-context-building or domain-specific-audits)
- Writing exploit POCs
- Code quality or gas optimization analysis
- Non-smart-contract codebases
- Analyzing read-only functions (this skill excludes them)

## Scope: State-Changing Functions Only

This skill focuses exclusively on functions that can modify state. **Excluded:**

| Language | Excluded Patterns |
| --- | --- |
| Solidity | `view`, `pure` functions |
| Vyper | `@view`, `@pure` functions |
| Solana | Functions without `mut` account references |
| Move | Non-entry `public fun` (module-callable only) |
| CosmWasm | `query` entry point and its handlers |

**Why exclude read-only functions?** They cannot directly cause loss of funds or state corruption.

## Workflow

1. **Detect Language** - Identify contract language(s) from file extensions and syntax
2. **Use Tooling (if available)** - For Solidity, check if Slither is available and use it
3. **Locate Contracts** - Find all contract/module files
4. **Extract Entry Points** - Parse each file for externally callable, state-changing functions
5. **Classify Access** - Categorize each function by access level
6. **Generate Report** - Output structured markdown report

## Slither Integration (Solidity)

For Solidity codebases, Slither can automatically extract entry points:

```bash
# Check if Slither is available
which slither

# If detected, run entry points printer
slither . --print entry-points
```

## Language Detection

| Extension | Language |
| --- | --- |
| `.sol` | Solidity |
| `.vy` | Vyper |
| `.rs` + `Cargo.toml` with `solana-program` | Solana (Rust) |
| `.move` | Move (Aptos/Sui) |
| `.fc`, `.func`, `.tact` | TON (FunC/Tact) |
| `.rs` + `Cargo.toml` with `cosmwasm-std` | CosmWasm |

## Access Classification

Classify each state-changing entry point into one of these categories:

### 1. Public (Unrestricted)

Functions callable by anyone without restrictions.

### 2. Role-Restricted

Functions limited to specific roles. Common patterns to detect:

- Explicit role names: `admin`, `owner`, `governance`, `guardian`, `operator`, `manager`, `minter`, `pauser`, `keeper`, `relayer`
- Role-checking patterns: `onlyRole`, `hasRole`, `require(msg.sender == X)`, `assert_owner`, `#[access_control]`

### 3. Contract-Only (Internal Integration Points)

Functions callable only by other contracts, not by EOAs. Indicators:

- Callbacks: `onERC721Received`, `uniswapV3SwapCallback`, `flashLoanCallback`
- Interface implementations with contract-caller checks
- Functions that revert if `tx.origin == msg.sender`
- Cross-contract hooks

## Output Format

Generate a markdown report with this structure:

```markdown
# Entry Point Analysis: [Project Name]

**Analyzed**: [timestamp]
**Scope**: [directories analyzed or "full codebase"]
**Languages**: [detected languages]
**Focus**: State-changing functions only (view/pure excluded)

## Summary

| Category | Count |
|----------|-------|
| Public (Unrestricted) | X |
| Role-Restricted | X |
| Contract-Only | X |
| **Total** | **X** |

---

## Public Entry Points (Unrestricted)

State-changing functions callable by anyone—prioritize for attack surface analysis.

| Function | File | Notes |
|----------|------|-------|
| `functionName(params)` | `path/to/file.sol:L42` | Brief note |

---

## Role-Restricted Entry Points

### Admin / Owner
| Function | File | Restriction |
|----------|------|-------------|
| `setFee(uint256)` | `Config.sol:L15` | `onlyOwner` |

---

## Contract-Only (Internal Integration Points)

| Function | File | Expected Caller |
|----------|------|-----------------|
| `onFlashLoan(...)` | `Vault.sol:L200` | Flash loan provider |
```

## Common Role Patterns by Protocol Type

| Protocol Type | Common Roles |
| --- | --- |
| DEX | `owner`, `feeManager`, `pairCreator` |
| Lending | `admin`, `guardian`, `liquidator`, `oracle` |
| Governance | `proposer`, `executor`, `canceller`, `timelock` |
| NFT | `minter`, `admin`, `royaltyReceiver` |
| Bridge | `relayer`, `guardian`, `validator`, `operator` |
| Vault/Yield | `strategist`, `keeper`, `harvester`, `manager` |

## Analysis Guidelines

1. **Be thorough**: Don't skip files. Every state-changing externally callable function matters.
2. **Be conservative**: When uncertain about access level, flag for review rather than miscategorize.
3. **Skip read-only**: Exclude `view`, `pure`, and equivalent read-only functions.
4. **Note inheritance**: If a function's access control comes from a parent contract, note this.
5. **Track modifiers**: List all access-related modifiers/decorators applied to each function.
