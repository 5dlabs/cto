import type { NatsConfig, PingPongState } from "./types";
import { createNatsClient, type NatsClientHandle } from "./client";
import { processInboundMessage } from "./processor";
import { deliverToAgent } from "./actions";

export interface PingPongCheck {
  allowed: boolean;
  count: number;
  limit: number;
}

export interface NatsServiceResult {
  /** Background service descriptor for api.registerService() */
  service: {
    id: string;
    name: string;
    start(ctx?: any): Promise<void>;
    stop(ctx?: any): Promise<void>;
  };
  /** Client handle for the tool to publish/request */
  handle: () => NatsClientHandle | null;
  /** Check if a message to peer is within ping-pong limits */
  checkPingPong: (peer: string) => PingPongCheck;
  /** Record an outbound message to peer */
  recordPingPong: (peer: string) => void;
  /** Get the most recent NATS reply subject for a peer (for request-reply) */
  getReplyContext: (peer: string) => string | undefined;
}

export function createService(
  config: NatsConfig,
  runtime: any,
  logger: { info: Function; warn: Function; error: Function },
): NatsServiceResult {
  let client: NatsClientHandle | null = null;

  // Ping-pong guard state
  const pingPongMap = new Map<string, PingPongState>();
  const maxTurns = config.maxPingPongTurns ?? 10;
  const windowMs = config.pingPongWindowMs ?? 300000;
  let gcInterval: ReturnType<typeof setInterval> | null = null;

  // Reply context: maps peer name -> most recent NATS reply subject
  const replyContextMap = new Map<string, { subject: string; timestamp: number }>();

  function getReplyContext(peer: string): string | undefined {
    const ctx = replyContextMap.get(peer);
    if (!ctx) return undefined;
    // Expire stale reply contexts (same window as ping-pong)
    if (Date.now() - ctx.timestamp > windowMs) {
      replyContextMap.delete(peer);
      return undefined;
    }
    return ctx.subject;
  }

  function getOrCreateState(peer: string): PingPongState {
    let state = pingPongMap.get(peer);
    if (!state) {
      state = { count: 0, lastReset: Date.now() };
      pingPongMap.set(peer, state);
    }
    return state;
  }

  function checkPingPong(peer: string): PingPongCheck {
    const state = getOrCreateState(peer);
    // Auto-reset if window expired
    if (Date.now() - state.lastReset > windowMs) {
      state.count = 0;
      state.lastReset = Date.now();
    }
    return { allowed: state.count < maxTurns, count: state.count, limit: maxTurns };
  }

  function recordPingPong(peer: string): void {
    const state = getOrCreateState(peer);
    if (Date.now() - state.lastReset > windowMs) {
      state.count = 0;
      state.lastReset = Date.now();
    }
    state.count++;
  }

  const service = {
    id: "nats-messenger-service",
    name: "NATS Messenger",

    async start(_ctx?: any): Promise<void> {
      if (!config.enabled) {
        logger.info("NATS messenger disabled by config");
        return;
      }

      logger.info(`Starting NATS messenger for agent "${config.agentName}"`);
      logger.info(`Subscribing to: ${config.subjects.join(", ")}`);

      client = await createNatsClient(
        config,
        (subject, msg) => {
          // Inbound ping-pong guard
          const check = checkPingPong(msg.from);
          if (!check.allowed) {
            logger.warn(
              `Ping-pong limit reached for ${msg.from} (${check.count}/${check.limit}), dropping message`,
            );
            return;
          }
          recordPingPong(msg.from);

          // Store reply context so the agent can reply to request-reply messages
          if (msg.replyTo) {
            replyContextMap.set(msg.from, { subject: msg.replyTo, timestamp: Date.now() });
          }

          const processed = processInboundMessage(
            subject,
            msg,
            config.agentName,
          );
          deliverToAgent(runtime, processed, logger);
        },
        logger,
      );

      // Periodic GC for stale ping-pong and reply context entries
      gcInterval = setInterval(() => {
        const now = Date.now();
        for (const [peer, state] of pingPongMap) {
          if (now - state.lastReset > windowMs) {
            pingPongMap.delete(peer);
          }
        }
        for (const [peer, ctx] of replyContextMap) {
          if (now - ctx.timestamp > windowMs) {
            replyContextMap.delete(peer);
          }
        }
      }, windowMs);
    },

    async stop(_ctx?: any): Promise<void> {
      if (gcInterval) {
        clearInterval(gcInterval);
        gcInterval = null;
      }
      if (client) {
        logger.info("Stopping NATS messenger");
        await client.close();
        client = null;
      }
    },
  };

  return {
    service,
    handle: () => client,
    checkPingPong,
    recordPingPong,
    getReplyContext,
  };
}
