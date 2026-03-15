/**
 * Helius Enhanced WebSocket — transactionSubscribe
 *
 * Uses Helius Business+ Enhanced WebSocket for real-time
 * transaction monitoring with auto-parsed token changes.
 *
 * Much more reliable than onAccountChange + getRecentTransactions
 * because the parsed data comes directly in the notification.
 */

const WebSocket = require('ws');

const HELIUS_API_KEY = process.env.HELIUS_API_KEY;
const WS_URL = `wss://mainnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}`;

// Known stablecoin/SOL mints to skip
const SKIP_MINTS = new Set([
  'So11111111111111111111111111111111111111112',
  'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC
  'Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB', // USDT
]);

/**
 * Subscribe to a wallet's transactions via Helius Enhanced WebSocket.
 * Returns parsed token changes directly — no extra RPC calls needed.
 *
 * @param {string[]} wallets - Array of wallet addresses to monitor
 * @param {function} onTrade - Callback: (trade) => void
 *   trade = { signature, wallet, tokenChanges: [{mint, change, action, owner}], solChange, dex, timestamp }
 * @returns {{ ws, stop }} - WebSocket instance and stop function
 */
function subscribeToWalletTrades(wallets, onTrade) {
  if (!HELIUS_API_KEY) {
    throw new Error('HELIUS_API_KEY not set — required for Enhanced WebSocket');
  }

  let ws;
  let pingInterval;
  let reconnectTimeout;
  let stopped = false;

  function connect() {
    if (stopped) return;

    ws = new WebSocket(WS_URL);

    ws.on('open', () => {
      console.log(`[HeliusWS] Connected — subscribing to ${wallets.length} wallet(s)`);

      // transactionSubscribe with account filter
      ws.send(JSON.stringify({
        jsonrpc: '2.0',
        id: 420,
        method: 'transactionSubscribe',
        params: [
          {
            accountInclude: wallets,
            accountExclude: [],
            accountRequired: [],
          },
          {
            commitment: 'confirmed',
            encoding: 'jsonParsed',
            transactionDetails: 'full',
            showRewards: false,
            maxSupportedTransactionVersion: 0,
          },
        ],
      }));

      // Keep-alive ping every 30s
      pingInterval = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.ping();
        }
      }, 30000);
    });

    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data.toString());

        // Subscription confirmation
        if (msg.id === 420 && msg.result !== undefined) {
          console.log(`[HeliusWS] Subscription active (id: ${msg.result})`);
          return;
        }

        // Transaction notification
        if (msg.method === 'transactionNotification' && msg.params?.result) {
          const result = msg.params.result;
          const tx = result.transaction;
          const meta = tx?.meta;
          const sig = result.signature;

          if (!meta || meta.err) return; // Skip failed transactions

          // Parse token balance changes
          const preBalances = meta.preTokenBalances || [];
          const postBalances = meta.postTokenBalances || [];
          const changes = [];

          for (const post of postBalances) {
            if (SKIP_MINTS.has(post.mint)) continue;

            const pre = preBalances.find(
              (p) => p.mint === post.mint && p.accountIndex === post.accountIndex
            );

            const preAmt = pre ? parseFloat(pre.uiTokenAmount?.uiAmount || 0) : 0;
            const postAmt = parseFloat(post.uiTokenAmount?.uiAmount || 0);
            const change = postAmt - preAmt;

            if (Math.abs(change) > 0.001) {
              changes.push({
                mint: post.mint,
                change,
                action: change > 0 ? 'BUY' : 'SELL',
                owner: post.owner,
              });
            }
          }

          if (changes.length === 0) return; // No meaningful token trades

          // SOL change
          const solChange =
            meta.postBalances && meta.preBalances
              ? (meta.postBalances[0] - meta.preBalances[0]) / 1e9
              : 0;

          // Detect DEX used
          const accountKeys = tx.transaction?.message?.accountKeys?.map(
            (k) => (typeof k === 'object' ? k.pubkey : k)
          ) || [];

          const DEX_MAP = {
            JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4: 'JUPITER',
            'JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB': 'JUPITER_V4',
            '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8': 'RAYDIUM',
            whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc: 'ORCA',
            '6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P': 'PUMP_FUN',
            PSwapMdSai8tjrEXcxFeQth87xC4rRsa4VA5mhGhXkP: 'PUMPSWAP',
            Eo7WjKq67rjJQDd81bBQXXZMCFNiPQWASbcYxKBF3Rn: 'METEORA',
          };

          let dex = 'UNKNOWN';
          for (const key of accountKeys) {
            if (DEX_MAP[key]) {
              dex = DEX_MAP[key];
              break;
            }
          }

          // Find which monitored wallet is involved
          const involvedWallet = wallets.find((w) => accountKeys.includes(w));

          const trade = {
            signature: sig,
            wallet: involvedWallet || 'unknown',
            tokenChanges: changes,
            solChange,
            dex,
            timestamp: Date.now(),
            success: true,
          };

          console.log(
            `[HeliusWS] Trade detected: ${dex} | ${changes.map((c) => `${c.action} ${c.mint.slice(0, 12)}...`).join(', ')}`
          );
          onTrade(trade);
        }
      } catch (e) {
        // Parse errors are non-fatal
      }
    });

    ws.on('close', (code) => {
      console.log(`[HeliusWS] Disconnected (code: ${code})`);
      clearInterval(pingInterval);
      if (!stopped) {
        console.log('[HeliusWS] Reconnecting in 3s...');
        reconnectTimeout = setTimeout(connect, 3000);
      }
    });

    ws.on('error', (err) => {
      console.error(`[HeliusWS] Error: ${err.message}`);
    });
  }

  connect();

  return {
    get ws() { return ws; },
    stop: () => {
      stopped = true;
      clearInterval(pingInterval);
      clearTimeout(reconnectTimeout);
      if (ws) ws.close();
    },
  };
}

module.exports = { subscribeToWalletTrades };
