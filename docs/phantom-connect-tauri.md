# Phantom Connect integration for the CTO desktop app

## Outcome

Integrate Phantom into the Tauri desktop app by using Phantom's Browser SDK inside the app webview, with Solana transaction creation in the frontend and any app-side co-signing delegated to a backend service.

## Recommendation

Use **`@phantom/browser-sdk` in the Tauri frontend/webview**.

Do **not** build around browser-extension-only injection as the primary path. In Tauri, the most reliable model is:

- Tauri shell for desktop packaging and native IPC
- frontend webview for Phantom Connect UI/session handling
- backend API for any app-held signing responsibilities

For Solana transaction approval, prefer **`signAndSendTransaction`**.

## Why this is the right fit

Phantom's documented SDK families map to our app like this:

- **Browser SDK**: best fit for Tauri webview-based desktop UI
- **React SDK**: optional wrapper if we want React ergonomics on top of Browser SDK
- **React Native SDK**: not relevant for Tauri

The CTO desktop app is already a Tauri application with a web frontend, so the Browser SDK matches Phantom's supported environment model much better than trying to treat Tauri as a native wallet host.

## Recommended integration model

### Primary path: embedded wallet flow

Use Phantom Browser SDK with embedded-capable providers first:

- `google`
- `apple`
- `phantom`

Optionally support:

- `injected` as a power-user fallback when available

Why:

- extension injection inside Tauri webviews may be inconsistent across platforms
- embedded wallets give us a predictable, first-party connect/sign UX
- we control the app flow instead of depending on desktop browser state

## Solana signing methodology

For embedded wallets, Phantom documents an important limitation:

- `signTransaction` and `signAllTransactions` are **not supported** for embedded wallets
- `signAndSendTransaction` **is supported** and is the recommended path

That means our default transaction flow should be:

1. frontend builds the Solana transaction
2. frontend calls `sdk.solana.signAndSendTransaction(...)`
3. Phantom handles user approval and network broadcast
4. app records the returned signature and tracks confirmation

### When we need app-side co-signing

If we need:

- dapp-sponsored fees
- platform fee instructions
- multi-signer Solana transactions

then use Phantom's **`presignTransaction`** callback with a backend endpoint.

Important rule: **never keep fee-payer or platform signing keys in frontend code**.

The correct flow is:

1. frontend calls `signAndSendTransaction(tx, { presignTransaction })`
2. callback sends the base64url transaction to our backend
3. backend partially signs and returns the updated transaction
4. Phantom prompts the user and broadcasts

## Proposed architecture for CTO desktop

## Components

### 1. Tauri frontend/webview

Responsibilities:

- initialize Phantom Browser SDK
- handle connect/disconnect state
- fetch the user's public key
- build Solana transactions
- request user approval/sign-and-send
- show signatures, pending state, and confirmations

Suggested packages:

- `@phantom/browser-sdk`
- `@solana/web3.js` or `@solana/kit`
- app state/query library already used by the desktop UI

### 2. Tauri Rust layer

Responsibilities:

- expose non-secret native helpers if needed
- open external auth/callback URLs only if required by UX
- securely persist non-wallet application settings
- never store or generate Phantom wallet keys

### 3. Backend service

Responsibilities:

- prepare unsigned Solana transactions when business logic lives server-side
- optionally co-sign sponsored transactions
- verify submitted signatures
- index transaction status for app history

## Integration flow

### Basic connect flow

1. user clicks **Connect Wallet** in the Tauri app
2. frontend initializes Browser SDK
3. user selects embedded provider (`google`, `apple`, or `phantom`)
4. frontend receives connected account/public key
5. app stores session state locally

### Basic sign/send flow

1. app requests an unsigned transaction from backend, or builds one locally
2. frontend calls `sdk.solana.signAndSendTransaction(transaction)`
3. Phantom presents approval UI
4. SDK returns transaction hash/signature
5. app tracks confirmation and updates activity history

### Sponsored transaction flow

1. frontend obtains an unsigned transaction
2. frontend calls `sdk.solana.signAndSendTransaction(transaction, { presignTransaction })`
3. `presignTransaction` posts the encoded transaction to backend
4. backend partially signs as fee payer / cosigner
5. backend returns updated encoded transaction
6. Phantom prompts user and broadcasts
7. app tracks resulting signature

## Tauri-specific caveats

### 1. Redirect URL and app registration

Phantom embedded flows require:

- a Phantom app ID
- allowlisted domains/origins
- allowlisted redirect URLs

This is the main thing to validate early.

For the CTO desktop app, we should avoid assuming a Tauri-local origin will work identically to a normal hosted web app until proven in a spike.

### 2. Popup/session behavior inside webviews

Before building the full feature, verify:

- provider login opens correctly in the Tauri webview context
- callback/redirect returns control to the app reliably
- session persistence works across app restart
- macOS and Windows behavior are both acceptable

### 3. Extension detection should be optional

If `injected` provider works in some environments, great. But it should be treated as an enhancement, not the product baseline.

## Recommended implementation phases

### Phase 1: spike

Goal: prove embedded connect flow works inside Tauri.

Tasks:

- add Browser SDK to the desktop frontend
- register a Phantom app ID and redirect URL for local/dev usage
- implement a minimal connect/disconnect screen
- verify wallet address retrieval in macOS and Windows builds

Success criteria:

- user can connect via embedded provider from inside the Tauri app
- app receives and displays Solana public key

### Phase 2: basic transaction sending

Goal: send a simple Solana transfer or no-op/reference transaction.

Tasks:

- add Solana transaction builder in frontend
- call `signAndSendTransaction`
- display returned signature and confirmation state

Success criteria:

- user can approve a transaction from the desktop app
- app receives signature and can confirm it on-chain

### Phase 3: sponsored / advanced transaction support

Goal: support fee sponsorship or backend co-sign flows.

Tasks:

- add backend presign endpoint
- wire `presignTransaction`
- add audit logging and failure handling

Success criteria:

- app can submit a co-signed Solana transaction without exposing app keys in frontend code

## Security guidance

- never store Phantom private keys in Tauri or frontend code
- never keep fee-payer secrets in the webview bundle
- prefer remote backend custody for any app signing role
- treat transaction construction data from frontend as untrusted until validated server-side
- log wallet connection and signature metadata, but do not log sensitive auth artifacts

## Proposed frontend wrapper

```ts
import { BrowserSDK, AddressType } from "@phantom/browser-sdk";

export function createPhantomSdk(appId: string, redirectUrl: string) {
  return new BrowserSDK({
    appId,
    addressTypes: [AddressType.solana],
    providers: ["google", "apple", "phantom", "injected"],
    authOptions: {
      redirectUrl,
    },
    autoConnect: true,
  });
}
```

Example usage:

```ts
const sdk = createPhantomSdk(PHANTOM_APP_ID, PHANTOM_REDIRECT_URL);
await sdk.connect({ provider: "google" });
const publicKey = await sdk.solana.getPublicKey();
const result = await sdk.solana.signAndSendTransaction(transaction);
console.log(result.hash);
```

## Backend presign contract

Suggested request:

```json
{
  "transaction": "<base64url-encoded-tx>",
  "networkId": "solana:mainnet"
}
```

Suggested response:

```json
{
  "transaction": "<base64url-encoded-partially-signed-tx>"
}
```

## Recommendation summary

For the CTO desktop app, the best path is:

- **Browser SDK in Tauri webview**
- **embedded wallet first**
- **`signAndSendTransaction` for Solana**
- **backend presign service for sponsored or multi-signer flows**
- **optional injected-provider fallback, not primary**

## References

- Phantom docs: Browser SDK overview
- Phantom docs: sign and send transactions
- Phantom docs: Solana transaction signing
- Phantom docs: wallet SDK overview
