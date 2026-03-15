const express = require('express');
const path = require('path');
const fs = require('fs');

// Load shared .env from parent trader directory
require('dotenv').config({ path: path.resolve(__dirname, '..', '.env') });

const app = express();
app.use(express.json());
app.use(express.static(path.join(__dirname, 'public')));

const PORT = process.env.DASHBOARD_PORT || 3847;

// ─── Paths ───────────────────────────────────────────────────────────────────
const TRADER_ROOT = path.resolve(__dirname, '..');

const PATHS = {
  copyTrader: {
    summary: path.join(TRADER_ROOT, 'solana-copy-trader/scripts/logs/paper-summary.json'),
    trades: path.join(TRADER_ROOT, 'solana-copy-trader/scripts/logs/paper-trades.jsonl'),
  },
  moltiumv2: {
    runs: path.join(TRADER_ROOT, 'moltiumv2/tools/moltium/local/autostrategy/runs/paper-safe.jsonl'),
    state: path.join(TRADER_ROOT, 'moltiumv2/tools/moltium/local/autostrategy/state/paper-safe.json'),
    events: path.join(TRADER_ROOT, 'moltiumv2/tools/moltium/local/autostrategy/events'),
    strategy: path.join(TRADER_ROOT, 'moltiumv2/tools/moltium/local/autostrategy/strategies/paper-safe/strategy.json'),
  },
  polymarket: {
    config: path.join(TRADER_ROOT, 'polymarket-copytrading/config.json'),
  },
  temporalArb: {
    logs: path.join(TRADER_ROOT, 'temporal-arb/logs'),
  },
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

function readJsonSafe(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf-8'));
  } catch {
    return null;
  }
}

function readJsonlSafe(filePath, limit = 200) {
  try {
    const content = fs.readFileSync(filePath, 'utf-8').trim();
    if (!content) return [];
    const lines = content.split('\n');
    const recent = lines.slice(-limit);
    return recent.map(line => {
      try { return JSON.parse(line); } catch { return null; }
    }).filter(Boolean);
  } catch {
    return [];
  }
}

function fileModTime(filePath) {
  try {
    return fs.statSync(filePath).mtime.toISOString();
  } catch {
    return null;
  }
}

function isRecentlyActive(filePath, thresholdMs = 120_000) {
  try {
    const stat = fs.statSync(filePath);
    return Date.now() - stat.mtime.getTime() < thresholdMs;
  } catch {
    return false;
  }
}

// ─── Strategy Readers ────────────────────────────────────────────────────────

function getCopyTraderStatus() {
  const summary = readJsonSafe(PATHS.copyTrader.summary);
  if (!summary) {
    return {
      name: 'Solana Copy Trader',
      id: 'copy-trader',
      status: 'stopped',
      mode: 'paper',
      balance: 0,
      invested: 0,
      netPnl: 0,
      netPnlPct: 0,
      openPositions: 0,
      closedTrades: 0,
      winRate: '0%',
      wins: 0,
      losses: 0,
      lastTradeTime: null,
    };
  }

  const openCount = (summary.openPositions || []).length;
  const investedInOpen = (summary.openPositions || []).reduce((s, p) => s + (p.solSpent || 0), 0);
  const sells = (summary.trades || []).filter(t => t.action === 'SELL');
  const lastTrade = summary.trades && summary.trades.length > 0
    ? summary.trades[summary.trades.length - 1]
    : null;

  return {
    name: 'Solana Copy Trader',
    id: 'copy-trader',
    status: isRecentlyActive(PATHS.copyTrader.summary) ? 'running' : 'stopped',
    mode: 'paper',
    balance: summary.endSOL || 0,
    startBalance: summary.startSOL || 0,
    invested: investedInOpen,
    netPnl: summary.totalPnL || 0,
    netPnlPct: summary.totalPnLPct || 0,
    openPositions: openCount,
    closedTrades: sells.length,
    totalTrades: summary.totalTrades || 0,
    wins: summary.wins || 0,
    losses: summary.losses || 0,
    winRate: summary.winRate || '0%',
    lastTradeTime: lastTrade ? lastTrade.timestamp : null,
    unit: 'SOL',
  };
}

function getMoltiumv2Status() {
  const state = readJsonSafe(PATHS.moltiumv2.state);
  const strategy = readJsonSafe(PATHS.moltiumv2.strategy);
  const runs = readJsonlSafe(PATHS.moltiumv2.runs, 100);

  const isDryRun = strategy?.execution?.dryRun !== false;
  const positions = state?.positions || {};
  const openCount = Object.keys(positions).length;

  // Parse runs for trade actions
  const tradeRuns = runs.filter(r => r.actions && r.actions.length > 0);
  const allActions = runs.flatMap(r => (r.actions || []).map(a => ({ ...a, runTime: r.t })));
  const buys = allActions.filter(a => a.action === 'buy' || a.type === 'buy');
  const sells = allActions.filter(a => a.action === 'sell' || a.type === 'sell');

  // Calculate PnL from sell actions
  let totalPnl = 0;
  let wins = 0;
  let losses = 0;
  for (const sell of sells) {
    const pnl = sell.pnl || sell.profit || 0;
    totalPnl += pnl;
    if (pnl > 0) wins++;
    else losses++;
  }

  const closedCount = sells.length;
  const winRate = closedCount > 0 ? ((wins / closedCount) * 100).toFixed(0) + '%' : '0%';

  // Last activity
  const lastRun = runs.length > 0 ? runs[runs.length - 1] : null;
  const lastTickTime = state?.lastTickAt ? new Date(state.lastTickAt).toISOString() : null;

  return {
    name: 'Moltiumv2 Sniper',
    id: 'moltiumv2',
    status: isRecentlyActive(PATHS.moltiumv2.runs) ? 'running' : 'stopped',
    mode: isDryRun ? 'paper' : 'live',
    balance: lastRun?.solBal || 0,
    startBalance: 0,
    invested: 0,
    netPnl: totalPnl,
    netPnlPct: 0,
    openPositions: openCount,
    closedTrades: closedCount,
    totalTrades: buys.length + sells.length,
    wins,
    losses,
    winRate,
    lastTradeTime: lastTickTime,
    unit: 'SOL',
  };
}

function getPolymarketStatus() {
  const config = readJsonSafe(PATHS.polymarket.config);

  return {
    name: 'Polymarket Copy Trading',
    id: 'polymarket',
    status: 'stopped',
    mode: 'paper',
    balance: 0,
    startBalance: 0,
    invested: 0,
    netPnl: 0,
    netPnlPct: 0,
    openPositions: 0,
    closedTrades: 0,
    totalTrades: 0,
    wins: 0,
    losses: 0,
    winRate: '0%',
    lastTradeTime: null,
    unit: 'USDC',
    note: config ? `Tracking ${(config.wallets || '').split(',').length} wallets` : 'Not configured',
  };
}

function getTemporalArbStatus() {
  let trades = [];
  let lastTradeTime = null;

  try {
    const logDir = PATHS.temporalArb.logs;
    if (fs.existsSync(logDir)) {
      const files = fs.readdirSync(logDir).filter(f => f.endsWith('.jsonl') || f.endsWith('.json'));
      for (const file of files) {
        const entries = readJsonlSafe(path.join(logDir, file), 50);
        trades.push(...entries);
      }
      if (trades.length > 0) {
        const last = trades[trades.length - 1];
        lastTradeTime = last.timestamp || last.t || null;
      }
    }
  } catch {}

  let totalPnl = 0;
  let wins = 0;
  let losses = 0;
  for (const t of trades) {
    const pnl = t.pnl || t.profit || 0;
    totalPnl += pnl;
    if (pnl > 0) wins++;
    else if (pnl < 0) losses++;
  }
  const closedCount = wins + losses;
  const winRate = closedCount > 0 ? ((wins / closedCount) * 100).toFixed(0) + '%' : '0%';

  return {
    name: 'Temporal Arbitrage',
    id: 'temporal-arb',
    status: 'stopped',
    mode: 'paper',
    balance: 0,
    startBalance: 0,
    invested: 0,
    netPnl: totalPnl,
    netPnlPct: 0,
    openPositions: 0,
    closedTrades: closedCount,
    totalTrades: trades.length,
    wins,
    losses,
    winRate,
    lastTradeTime,
    unit: 'USDC',
    note: 'CEX-to-Polymarket latency arb',
  };
}

// ─── API Routes ──────────────────────────────────────────────────────────────

app.get('/api/status', (req, res) => {
  const strategies = [
    getCopyTraderStatus(),
    getMoltiumv2Status(),
    getPolymarketStatus(),
    getTemporalArbStatus(),
  ];

  const totalPnl = strategies.reduce((s, st) => s + st.netPnl, 0);
  const totalOpen = strategies.reduce((s, st) => s + st.openPositions, 0);
  const totalClosed = strategies.reduce((s, st) => s + st.closedTrades, 0);
  const totalWins = strategies.reduce((s, st) => s + st.wins, 0);
  const totalLosses = strategies.reduce((s, st) => s + st.losses, 0);
  const overallWinRate = (totalWins + totalLosses) > 0
    ? ((totalWins / (totalWins + totalLosses)) * 100).toFixed(0) + '%'
    : '0%';

  res.json({
    timestamp: new Date().toISOString(),
    portfolio: {
      totalPnl,
      openPositions: totalOpen,
      closedTrades: totalClosed,
      winRate: overallWinRate,
      totalWins,
      totalLosses,
    },
    strategies,
  });
});

app.get('/api/trades', (req, res) => {
  const limit = parseInt(req.query.limit) || 50;
  const allTrades = [];

  // Copy trader trades
  const ctTrades = readJsonlSafe(PATHS.copyTrader.trades, 100);
  for (const t of ctTrades) {
    allTrades.push({
      strategy: 'Solana Copy Trader',
      strategyId: 'copy-trader',
      timestamp: t.timestamp,
      action: t.action,
      asset: t.mint ? (t.mint.slice(0, 6) + '...' + t.mint.slice(-4)) : 'Unknown',
      assetFull: t.mint,
      amount: t.solAmount,
      unit: 'SOL',
      route: t.jupiterRoute || '',
      pnl: t.cumulativePnL || 0,
    });
  }

  // Moltiumv2 runs with actions
  const mRuns = readJsonlSafe(PATHS.moltiumv2.runs, 100);
  for (const run of mRuns) {
    for (const action of (run.actions || [])) {
      allTrades.push({
        strategy: 'Moltiumv2 Sniper',
        strategyId: 'moltiumv2',
        timestamp: new Date(run.t).toISOString(),
        action: (action.action || action.type || 'unknown').toUpperCase(),
        asset: action.mint ? (action.mint.slice(0, 6) + '...' + action.mint.slice(-4)) : 'Unknown',
        assetFull: action.mint || action.tokenAddress,
        amount: action.solAmount || action.amount || 0,
        unit: 'SOL',
        route: action.source || '',
        pnl: action.pnl || 0,
      });
    }
  }

  // Sort by timestamp descending
  allTrades.sort((a, b) => {
    const ta = new Date(a.timestamp || 0).getTime();
    const tb = new Date(b.timestamp || 0).getTime();
    return tb - ta;
  });

  res.json({ trades: allTrades.slice(0, limit) });
});

app.get('/api/positions', (req, res) => {
  const positions = [];

  // Copy trader open positions
  const summary = readJsonSafe(PATHS.copyTrader.summary);
  if (summary?.openPositions) {
    for (const pos of summary.openPositions) {
      positions.push({
        strategy: 'Solana Copy Trader',
        strategyId: 'copy-trader',
        mint: pos.mint,
        mintShort: pos.mint ? (pos.mint.slice(0, 6) + '...' + pos.mint.slice(-4)) : 'Unknown',
        tokens: pos.tokens,
        invested: pos.solSpent,
        buyPrice: pos.buyPrice,
        unit: 'SOL',
      });
    }
  }

  // Moltiumv2 open positions
  const state = readJsonSafe(PATHS.moltiumv2.state);
  if (state?.positions) {
    for (const [mint, pos] of Object.entries(state.positions)) {
      positions.push({
        strategy: 'Moltiumv2 Sniper',
        strategyId: 'moltiumv2',
        mint,
        mintShort: mint ? (mint.slice(0, 6) + '...' + mint.slice(-4)) : 'Unknown',
        tokens: pos.tokens || pos.amount || 0,
        invested: pos.solSpent || pos.cost || 0,
        buyPrice: pos.buyPrice || pos.entryPrice || 0,
        unit: 'SOL',
      });
    }
  }

  res.json({ positions });
});

app.post('/api/toggle/:strategy', (req, res) => {
  const { strategy } = req.params;
  const { confirm } = req.body;

  if (strategy === 'moltiumv2') {
    const strategyData = readJsonSafe(PATHS.moltiumv2.strategy);
    if (!strategyData) {
      return res.status(404).json({ error: 'Strategy config not found' });
    }

    const currentDryRun = strategyData.execution?.dryRun !== false;
    const newDryRun = !currentDryRun;

    // Require confirmation to go live
    if (!newDryRun && !confirm) {
      return res.json({
        requireConfirm: true,
        message: 'Switching Moltiumv2 to LIVE mode. Real funds will be traded. Confirm?',
        currentMode: currentDryRun ? 'paper' : 'live',
        newMode: 'live',
      });
    }

    strategyData.execution.dryRun = newDryRun;
    try {
      fs.writeFileSync(PATHS.moltiumv2.strategy, JSON.stringify(strategyData, null, 2));
      return res.json({
        success: true,
        strategy: 'moltiumv2',
        mode: newDryRun ? 'paper' : 'live',
      });
    } catch (err) {
      return res.status(500).json({ error: 'Failed to write strategy config', detail: err.message });
    }
  }

  if (strategy === 'copy-trader') {
    // Copy trader doesn't have a strategy.json toggle yet -- return info
    return res.json({
      success: false,
      message: 'Copy trader paper/live toggle requires editing the launch command. Use --paper flag when starting.',
      currentMode: 'paper',
    });
  }

  if (strategy === 'polymarket' || strategy === 'temporal-arb') {
    return res.json({
      success: false,
      message: `${strategy} paper/live toggle not yet implemented.`,
    });
  }

  res.status(404).json({ error: `Unknown strategy: ${strategy}` });
});

// SPA fallback
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, 'public', 'index.html'));
});

app.listen(PORT, () => {
  console.log(`\n  IRONMAN Trading Dashboard`);
  console.log(`  http://localhost:${PORT}\n`);
  console.log(`  Monitoring strategies in: ${TRADER_ROOT}\n`);
});
