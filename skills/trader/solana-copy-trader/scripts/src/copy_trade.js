/**
 * IRONMAN — Copy Trade Engine
 * 
 * Whale ke trades real-time copy karo
 * Paper mode: simulate only
 * Live mode: actual execution (needs wallet)
 * 
 * HOW IT WORKS:
 * 1. WebSocket se whale ki activity detect karo
 * 2. Transaction parse karo — kya buy/sell hua
 * 3. Same trade execute karo (smaller size)
 * 4. Auto take-profit + stop-loss manage karo
 */

const { Connection, PublicKey, VersionedTransaction } = require('@solana/web3.js');
const axios = require('axios');
const fs = require('fs');
const path = require('path');
const { connection, wallet, TOKENS, config } = require('./config');
const { getJupiterQuote } = require('./price_monitor');
const { sendTelegram } = require('./alerts');
const { parseTransaction, getRecentTransactions } = require('./wallet_tracker');
const { getPumpToken, pumpSafetyCheck, calcBuy } = require('./pumpfun');
const { subscribeToWalletTrades } = require('./helius_ws');

// Logging directories
const LOGS_DIR = path.join(__dirname, '..', 'logs');
const TRADES_FILE = path.join(LOGS_DIR, 'paper-trades.jsonl');
const SUMMARY_FILE = path.join(LOGS_DIR, 'paper-summary.json');

// Ensure logs directory exists
fs.mkdirSync(LOGS_DIR, { recursive: true });

// Whale to copy
const WHALE = 'AgmLJBMDCqWynYnQiPCuj9ewsNNsBJXyzoUhD9LJzN51';

// Skip these — whale's internal routing, not real trades
const SKIP_MINTS = new Set([
  TOKENS.SOL,
  TOKENS.USDC, 
  TOKENS.USDT,
  'So11111111111111111111111111111111111111112',
]);

/**
 * Paper Trading State
 */
class PaperPortfolio {
  constructor(startSOL = 0.5) {
    this.sol = startSOL;
    this.startSOL = startSOL;
    this.positions = new Map(); // mint => {tokens, buyPrice, solSpent, time}
    this.closedTrades = [];
    this.allTrades = [];       // every trade for the summary
    this.log = [];
    this.startTime = new Date().toISOString();
    this.tradeSeq = 0;

    // Clear previous JSONL file for this run
    fs.writeFileSync(TRADES_FILE, '', 'utf8');
    console.log(`[Portfolio] Trade log: ${TRADES_FILE}`);
  }

  addLog(msg) {
    const entry = `[${new Date().toLocaleTimeString()}] ${msg}`;
    this.log.push(entry);
    console.log('[Portfolio]', entry);
  }

  _logTrade(action, mint, solAmount, tokensReceived, pricePerToken, jupiterRoute) {
    const totalPnL = this.closedTrades.reduce((s, t) => s + t.pnl, 0);
    const record = {
      seq: ++this.tradeSeq,
      timestamp: new Date().toISOString(),
      action,
      mint,
      solAmount,
      tokensReceived,
      pricePerToken,
      jupiterRoute: jupiterRoute || null,
      currentSOL: this.sol,
      cumulativePnL: totalPnL,
    };
    fs.appendFileSync(TRADES_FILE, JSON.stringify(record) + '\n', 'utf8');
    this.allTrades.push(record);
  }

  buy(mint, solAmount, tokensReceived, price, jupiterRoute) {
    if (this.sol < solAmount) {
      this.addLog(`Insufficient balance: ${this.sol.toFixed(4)} SOL`);
      return false;
    }
    this.sol -= solAmount;
    const existing = this.positions.get(mint) || { tokens: 0, solSpent: 0 };
    this.positions.set(mint, {
      tokens: existing.tokens + tokensReceived,
      solSpent: existing.solSpent + solAmount,
      buyPrice: price,
      time: Date.now(),
    });
    this.addLog(`BUY ${mint.slice(0,12)}... | ${solAmount} SOL -> ${tokensReceived.toFixed(0)} tokens`);
    this._logTrade('BUY', mint, solAmount, tokensReceived, price, jupiterRoute);
    return true;
  }

  sell(mint, solReceived, tokensSold, jupiterRoute) {
    const pos = this.positions.get(mint);
    if (!pos) return false;
    this.sol += solReceived;
    const pnl = solReceived - pos.solSpent;
    const pnlPct = (pnl / pos.solSpent) * 100;
    this.closedTrades.push({ mint, pnl, pnlPct, solReceived, solSpent: pos.solSpent });
    this.positions.delete(mint);
    this.addLog(`SELL ${mint.slice(0,12)}... | ${solReceived.toFixed(4)} SOL | P&L: ${pnlPct.toFixed(1)}%`);
    this._logTrade('SELL', mint, solReceived, tokensSold, solReceived / Math.max(tokensSold, 1), jupiterRoute);
    return true;
  }

  getStats() {
    const totalPnL = this.closedTrades.reduce((s, t) => s + t.pnl, 0);
    const wins = this.closedTrades.filter(t => t.pnl > 0);
    const losses = this.closedTrades.filter(t => t.pnl <= 0);
    const winRate = wins.length / Math.max(this.closedTrades.length, 1);
    return {
      currentSOL: this.sol,
      startSOL: this.startSOL,
      totalPnL,
      totalPnLPct: (totalPnL / this.startSOL) * 100,
      openPositions: this.positions.size,
      closedTrades: this.closedTrades.length,
      wins: wins.length,
      losses: losses.length,
      winRate: (winRate * 100).toFixed(0) + '%',
      avgWin: wins.length > 0 ? wins.reduce((s, t) => s + t.pnl, 0) / wins.length : 0,
      avgLoss: losses.length > 0 ? losses.reduce((s, t) => s + t.pnl, 0) / losses.length : 0,
    };
  }

  writeSummary() {
    const stats = this.getStats();
    const summary = {
      startTime: this.startTime,
      endTime: new Date().toISOString(),
      startSOL: this.startSOL,
      endSOL: this.sol,
      totalTrades: this.allTrades.length,
      wins: stats.wins,
      losses: stats.losses,
      winRate: stats.winRate,
      totalPnL: stats.totalPnL,
      totalPnLPct: stats.totalPnLPct,
      avgWin: stats.avgWin,
      avgLoss: stats.avgLoss,
      openPositions: Array.from(this.positions.entries()).map(([mint, pos]) => ({
        mint, tokens: pos.tokens, solSpent: pos.solSpent, buyPrice: pos.buyPrice,
      })),
      trades: this.allTrades,
    };
    fs.writeFileSync(SUMMARY_FILE, JSON.stringify(summary, null, 2), 'utf8');
    console.log(`\n[Portfolio] Summary written to ${SUMMARY_FILE}`);
    return summary;
  }
}

/**
 * Pending whale buys — track whale buys, only copy after whale holds for N seconds
 * This filters out rapid arb/MEV cycles and only copies conviction trades
 */
const pendingWhaleBuys = new Map(); // mint => { detectedAt, count }

/**
 * Core: Process whale transaction + decide action
 */
async function processWhaleTx(tx, portfolio, options = {}) {
  const {
    maxPositions = 3,
    solPerTrade = 0.01,
    takeProfitPct = 50,   // sell at +50%
    stopLossPct = 20,     // sell at -20%
    whaleHoldConfirmSec = 30, // only copy if whale hasn't sold within 30s
    minHoldSec = 60,      // hold our position for at least 60s before selling
    paper = true
  } = options;

  if (!tx || !tx.success) return;

  // Find meaningful token trades (not SOL/USDC routing)
  const realTrades = tx.tokenChanges.filter(c =>
    !SKIP_MINTS.has(c.mint) &&
    (c.mint.endsWith('pump') || Math.abs(c.change) > 1000)  // Pump.fun tokens OR big trades
  );

  if (realTrades.length === 0) return;

  for (const trade of realTrades) {
    const mint = trade.mint;

    if (trade.action === 'BUY') {
      // Whale bought — don't copy immediately, track as pending
      // If whale sells this mint before whaleHoldConfirmSec, it's arb — skip it
      const pending = pendingWhaleBuys.get(mint);
      if (pending) {
        pending.count++;
        console.log(`[CopyTrade] Whale re-bought ${mint.slice(0,12)} (${pending.count}x) — still waiting for hold confirmation`);
        continue;
      }

      pendingWhaleBuys.set(mint, { detectedAt: Date.now(), count: 1 });
      console.log(`[CopyTrade] Whale BUY detected: ${mint.slice(0,12)}... — waiting ${whaleHoldConfirmSec}s for hold confirmation`);
      continue; // Don't buy yet — wait for confirmation

    } else if (trade.action === 'SELL') {
      // Whale sold — if it's a pending buy, cancel it (was arb, not conviction)
      if (pendingWhaleBuys.has(mint)) {
        const pending = pendingWhaleBuys.get(mint);
        const holdSec = (Date.now() - pending.detectedAt) / 1000;
        pendingWhaleBuys.delete(mint);
        console.log(`[CopyTrade] Whale sold ${mint.slice(0,12)} after ${holdSec.toFixed(0)}s — arb detected, skipping`);
        continue;
      }

      // If we have a position, check minimum hold time before selling
      if (!portfolio.positions.has(mint)) continue;

      const pos = portfolio.positions.get(mint);
      const ourHoldSec = (Date.now() - pos.time) / 1000;
      if (ourHoldSec < minHoldSec) {
        console.log(`[CopyTrade] Whale sold ${mint.slice(0,12)} but we've only held ${ourHoldSec.toFixed(0)}s (min: ${minHoldSec}s) — holding`);
        continue;
      }

      console.log(`\n[CopyTrade] Whale SELL detected: ${mint.slice(0,20)}... (held ${ourHoldSec.toFixed(0)}s)`);

      const sellQuote = await getJupiterQuote(mint, TOKENS.SOL, Math.floor(pos.tokens), 2000);
      if (!sellQuote) continue;

      const solBack = parseInt(sellQuote.outAmount) / 1e9;
      const sellRouteLabel = sellQuote.routePlan?.map(r => r.swapInfo?.label).join('->') || 'Jupiter';

      if (paper) {
        portfolio.sell(mint, solBack, pos.tokens, sellRouteLabel);
        const pnl = solBack - pos.solSpent;
        const pnlPct = (pnl / pos.solSpent) * 100;

        await sendTelegram(`
COPY TRADE — PAPER SELL

Token: <code>${mint.slice(0,25)}...</code>
SOL received: ${solBack.toFixed(4)}
P&L: ${pnl >= 0 ? '+' : ''}${pnlPct.toFixed(1)}% ${pnl >= 0 ? '+' : '-'}
Hold time: ${ourHoldSec.toFixed(0)}s

Portfolio total: ${portfolio.sol.toFixed(4)} SOL
`.trim());
      }
      continue;
    }
  }
}

/**
 * Check pending whale buys — promote to real buys if whale held long enough
 * Called on an interval from startCopyTrader
 */
async function checkPendingBuys(portfolio, options = {}) {
  const {
    maxPositions = 3,
    solPerTrade = 0.01,
    whaleHoldConfirmSec = 30,
    paper = true
  } = options;

  const now = Date.now();

  for (const [mint, pending] of pendingWhaleBuys.entries()) {
    const ageSec = (now - pending.detectedAt) / 1000;
    if (ageSec < whaleHoldConfirmSec) continue;

    // Whale has held for whaleHoldConfirmSec — this is a conviction trade, copy it!
    pendingWhaleBuys.delete(mint);

    if (portfolio.positions.size >= maxPositions) {
      console.log(`[CopyTrade] Whale held ${mint.slice(0,12)} for ${ageSec.toFixed(0)}s (conviction!) but max positions reached`);
      continue;
    }

    if (portfolio.positions.has(mint)) {
      console.log(`[CopyTrade] Already have position in ${mint.slice(0,12)}`);
      continue;
    }

    console.log(`\n[CopyTrade] CONVICTION BUY: Whale held ${mint.slice(0,12)} for ${ageSec.toFixed(0)}s — copying!`);

      // Get our buy quote
      const lamports = Math.floor(solPerTrade * 1e9);
      // Try Jupiter first
      let quote = await getJupiterQuote(TOKENS.SOL, mint, lamports, 5000);
      if (!quote) quote = await getJupiterQuote(TOKENS.SOL, mint, lamports, 9900);

      let tokensOut, pricePerToken, priceImpact, routeLabel;
      let isPumpDirect = false;

      if (!quote) {
        // Jupiter failed — try pump.fun bonding curve
        console.log('[CopyTrade] Jupiter no route — checking pump.fun...');
        const safety = await pumpSafetyCheck(mint);
        
        if (!safety.pass) {
          console.log(`[CopyTrade] Pump safety FAIL (score: ${safety.score}) — skip`);
          continue;
        }

        if (safety.token.complete) {
          console.log('[CopyTrade] Token graduated but no Jupiter route — skip');
          continue;
        }

        // Estimate tokens (bonding curve data unavailable — use placeholder for paper mode)
        tokensOut = Math.floor(lamports / 1000); // rough estimate: 1 lamport = 0.001 token units
        if (tokensOut === 0) { console.log('[CopyTrade] Token estimate failed — skip'); continue; }
        
        pricePerToken = solPerTrade / tokensOut;
        priceImpact = 0; // unknown for bonding curve
        routeLabel = 'Pump.fun Bonding Curve';
        isPumpDirect = true;
        console.log(`[CopyTrade] Pump.fun route found! ${tokensOut.toLocaleString()} tokens out | mcap: $${safety.token.marketCap?.toFixed(0)}`);
        
      } else {
        tokensOut = parseInt(quote.outAmount);
        pricePerToken = solPerTrade / tokensOut;
        priceImpact = parseFloat(quote.priceImpactPct || 0);
        routeLabel = quote.routePlan?.map(r => r.swapInfo?.label).join('→') || 'Jupiter';

        if (priceImpact > 50) {
          console.log(`[CopyTrade] Price impact too high: ${priceImpact}% — skip`);
          continue;
        }
      }

      if (paper) {
        portfolio.buy(mint, solPerTrade, tokensOut, pricePerToken, routeLabel);

        await sendTelegram(`
🔴 <b>COPY TRADE — PAPER BUY</b>

Copying whale: AgmLJBM...
Token: <code>${mint.slice(0,25)}...</code>
SOL spent: ${solPerTrade} SOL
Tokens: ${tokensOut.toLocaleString()}
Impact: ${priceImpact.toFixed(2)}%
Route: ${routeLabel}
${isPumpDirect ? '🎯 Pump.fun direct trade' : ''}

Portfolio: ${portfolio.sol.toFixed(4)} SOL left
Positions: ${portfolio.positions.size}/${maxPositions}
`.trim());

      } else {
        // REAL EXECUTION
        if (!wallet) {
          console.log('[CopyTrade] No wallet configured — paper only!');
          continue;
        }
        await executeRealSwap(mint, TOKENS.SOL, lamports, quote);
      }
  }
}

/**
 * Real swap execution via Jupiter
 * Only runs when wallet is configured
 */
async function executeRealSwap(inputMint, outputMint, lamports, quote) {
  if (!wallet) throw new Error('No wallet configured');
  
  try {
    // Get swap transaction from Jupiter
    const { data: swapData } = await axios.post('https://lite-api.jup.ag/swap/v1/swap', {
      quoteResponse: quote,
      userPublicKey: wallet.publicKey.toString(),
      wrapAndUnwrapSol: true,
      prioritizationFeeLamports: 10000, // ~0.00001 SOL priority fee
    });

    // Deserialize + sign + send
    const swapTx = VersionedTransaction.deserialize(
      Buffer.from(swapData.swapTransaction, 'base64')
    );
    swapTx.sign([wallet]);
    
    const sig = await connection.sendRawTransaction(swapTx.serialize(), {
      skipPreflight: false,
      maxRetries: 3,
    });
    
    await connection.confirmTransaction(sig, 'confirmed');
    
    console.log(`[CopyTrade] ✅ REAL SWAP EXECUTED: ${sig}`);
    await sendTelegram(`✅ REAL SWAP: https://solscan.io/tx/${sig}`);
    return sig;
    
  } catch (e) {
    console.error('[CopyTrade] Swap failed:', e.message);
    await sendTelegram(`❌ Swap failed: ${e.message}`);
    return null;
  }
}

/**
 * MAIN: Start copy trading bot
 */
async function startCopyTrader(options = {}) {
  const opts = {
    solPerTrade: 0.01,
    maxPositions: 3,
    takeProfitPct: 50,
    stopLossPct: 20,
    whaleHoldConfirmSec: 30,  // wait 30s to confirm whale conviction
    minHoldSec: 60,           // hold our position at least 60s
    paper: true,
    ...options
  };

  const portfolio = new PaperPortfolio(opts.startSOL || 0.5);
  
  console.log(`
╔══════════════════════════════════════════╗
║  IRONMAN COPY TRADER                     ║
║  Copying: AgmLJBMDCqWynYnQiPCu...       ║
║  Mode: ${opts.paper ? 'PAPER (No real money)    ' : '⚠️  LIVE - REAL MONEY     '}║
║  Per trade: ${opts.solPerTrade} SOL${' '.repeat(25)}║
║  Max positions: ${opts.maxPositions}${' '.repeat(25)}║
╚══════════════════════════════════════════╝
  `);

  await sendTelegram(`
🤖 <b>Copy Trader STARTED</b>

Copying: AgmLJBMDCqWynYnQiPCu...zN51
Mode: ${opts.paper ? 'PAPER TRADING' : '⚠️ LIVE'}
Per trade: ${opts.solPerTrade} SOL
Balance: ${portfolio.sol} SOL
Max positions: ${opts.maxPositions}
`.trim());

  // Use Helius Enhanced WebSocket for real-time trade detection
  // Falls back to polling if HELIUS_API_KEY is not set
  let sub = null;
  let pollInterval = null;

  if (process.env.HELIUS_API_KEY) {
    console.log('[CopyTrade] Using Helius Enhanced WebSocket (transactionSubscribe)');
    sub = subscribeToWalletTrades([WHALE], async (tx) => {
      try {
        await processWhaleTx(tx, portfolio, opts);
      } catch (e) {
        console.error('[CopyTrade] Process error:', e.message);
      }
    });
  } else {
    console.log('[CopyTrade] No HELIUS_API_KEY — falling back to polling');
    let lastSig = null;
    pollInterval = setInterval(async () => {
      try {
        const sigs = await getRecentTransactions(WHALE, 3);
        if (!sigs.length) return;
        const newestSig = sigs[0].signature;
        if (newestSig === lastSig) return;
        lastSig = newestSig;
        console.log(`\n[CopyTrade] New tx: ${newestSig.slice(0,20)}...`);
        const tx = await parseTransaction(newestSig);
        if (tx) await processWhaleTx(tx, portfolio, opts);
      } catch (e) {
        if (!e.message?.includes('429')) console.error('[CopyTrade] Poll error:', e.message);
      }
    }, 3000);
  }

  // Check pending whale buys every 5 seconds — promote to real buys if whale held long enough
  const pendingInterval = setInterval(async () => {
    try {
      await checkPendingBuys(portfolio, opts);
    } catch (e) {
      console.error('[CopyTrade] Pending check error:', e.message);
    }
  }, 5000);

  // Stats report every 5 minutes
  const statsInterval = setInterval(async () => {
    const stats = portfolio.getStats();
    console.log('\n[Stats]', stats);
    await sendTelegram(`
📊 <b>Copy Trader Update</b>

Balance: ${stats.currentSOL.toFixed(4)} SOL
P&L: ${stats.totalPnL >= 0 ? '+' : ''}${stats.totalPnLPct.toFixed(2)}%
Trades: ${stats.closedTrades} closed | ${stats.openPositions} open
Win rate: ${stats.winRate}
`.trim());
  }, 5 * 60 * 1000);

  // SIGINT handler — write summary before exit
  const shutdown = () => {
    console.log('\n[CopyTrade] Shutting down — writing summary...');
    if (sub) sub.stop();
    if (pollInterval) clearInterval(pollInterval);
    clearInterval(pendingInterval);
    clearInterval(statsInterval);
    const summary = portfolio.writeSummary();
    const stats = portfolio.getStats();
    console.log('\n========================================');
    console.log('  PAPER TRADING SESSION COMPLETE');
    console.log('========================================');
    console.log(`  Start SOL:    ${summary.startSOL}`);
    console.log(`  End SOL:      ${summary.endSOL.toFixed(6)}`);
    console.log(`  Total Trades: ${summary.totalTrades}`);
    console.log(`  Wins:         ${summary.wins}`);
    console.log(`  Losses:       ${summary.losses}`);
    console.log(`  Win Rate:     ${summary.winRate}`);
    console.log(`  Total P&L:    ${summary.totalPnL >= 0 ? '+' : ''}${summary.totalPnL.toFixed(6)} SOL (${summary.totalPnLPct >= 0 ? '+' : ''}${summary.totalPnLPct.toFixed(2)}%)`);
    console.log(`  Avg Win:      ${stats.avgWin.toFixed(6)} SOL`);
    console.log(`  Avg Loss:     ${stats.avgLoss.toFixed(6)} SOL`);
    console.log(`  Open Pos:     ${summary.openPositions.length}`);
    console.log('========================================');
    console.log(`  Logs: ${TRADES_FILE}`);
    console.log(`  Summary: ${SUMMARY_FILE}`);
    console.log('========================================\n');
    process.exit(0);
  };

  process.on('SIGINT', shutdown);
  process.on('SIGTERM', shutdown);

  return {
    stop: shutdown,
    portfolio,
  };
}

module.exports = { startCopyTrader, PaperPortfolio, processWhaleTx };
