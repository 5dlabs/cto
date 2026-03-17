/**
 * NATS Status Publishing Library for Trading Agents
 *
 * Provides a simple interface for agents to publish status updates
 * and trade events to NATS subjects for dashboard consumption.
 *
 * Subjects:
 *   trader.status.{agentId}  — periodic status heartbeats
 *   trader.trades.{agentId}  — individual trade events
 *
 * Usage:
 *   import { publishStatus, publishTrade, disconnect } from "../lib/nats-status.js";
 *
 *   await publishStatus("info-arb-geo-aggressive", {
 *     strategy: "info-arb",
 *     mode: "paper",
 *     cyclesRun: 42,
 *     signalsFound: 3,
 *     tradesExecuted: 1,
 *     pnl: 12.50,
 *   });
 *
 *   await publishTrade("info-arb-geo-aggressive", {
 *     side: "yes",
 *     amount: 5.00,
 *     market: "Will BTC exceed 75K?",
 *     confidence: 0.72,
 *     paper: true,
 *   });
 */

let nc = null;
let sc = null;

async function getConnection() {
  if (nc) return { nc, sc };

  const natsUrl = process.env.NATS_URL || "nats://localhost:4222";

  try {
    const nats = await import("nats");
    sc = nats.StringCodec();
    nc = await nats.connect({
      servers: natsUrl,
      reconnect: true,
      maxReconnectAttempts: -1,
      reconnectTimeWait: 2000,
    });
    return { nc, sc };
  } catch (err) {
    // NATS not available — fail silently (local dev, no cluster)
    console.error(`[nats-status] Connection failed: ${err.message}`);
    return { nc: null, sc: null };
  }
}

/**
 * Publish a status heartbeat for this agent.
 *
 * @param {string} agentId - Agent identifier (e.g., "info-arb-geo-aggressive")
 * @param {Object} status - Status data (strategy, mode, metrics, etc.)
 */
export async function publishStatus(agentId, status) {
  const { nc, sc } = await getConnection();
  if (!nc) return;

  const payload = JSON.stringify({
    agentId,
    timestamp: new Date().toISOString(),
    type: "status",
    ...status,
  });

  nc.publish(`trader.status.${agentId}`, sc.encode(payload));
}

/**
 * Publish a trade event for this agent.
 *
 * @param {string} agentId - Agent identifier
 * @param {Object} trade - Trade details (side, amount, market, etc.)
 */
export async function publishTrade(agentId, trade) {
  const { nc, sc } = await getConnection();
  if (!nc) return;

  const payload = JSON.stringify({
    agentId,
    timestamp: new Date().toISOString(),
    type: "trade",
    ...trade,
  });

  nc.publish(`trader.trades.${agentId}`, sc.encode(payload));
}

/**
 * Gracefully disconnect from NATS.
 */
export async function disconnect() {
  if (nc) {
    await nc.drain();
    nc = null;
  }
}
