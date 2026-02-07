import {
  connect,
  type NatsConnection,
  type Subscription,
  type Msg,
  StringCodec,
} from "nats";
import type { NatsConfig, AgentMessage } from "./types";

const sc = StringCodec();

export interface NatsClientHandle {
  publish(subject: string, msg: AgentMessage): void;
  request(subject: string, msg: AgentMessage, timeoutMs: number): Promise<AgentMessage>;
  close(): Promise<void>;
  isConnected(): boolean;
}

export async function createNatsClient(
  config: NatsConfig,
  onMessage: (subject: string, msg: AgentMessage) => void,
  logger: { info: Function; warn: Function; error: Function },
): Promise<NatsClientHandle> {
  let nc: NatsConnection | null = null;
  const subs: Subscription[] = [];

  const doConnect = async (): Promise<NatsConnection> => {
    const conn = await connect({
      servers: config.url,
      name: config.agentName,
      reconnectTimeWait: config.reconnectWaitMs,
      maxReconnectAttempts: config.maxReconnectAttempts,
    });

    logger.info(`Connected to NATS at ${config.url} as "${config.agentName}"`);

    // Monitor connection events
    (async () => {
      for await (const s of conn.status()) {
        switch (s.type) {
          case "reconnecting":
            logger.warn(`NATS reconnecting...`);
            break;
          case "reconnect":
            logger.info(`NATS reconnected to ${s.data}`);
            break;
          case "disconnect":
            logger.warn(`NATS disconnected`);
            break;
          case "error":
            logger.error(`NATS error: ${s.data}`);
            break;
        }
      }
    })();

    return conn;
  };

  nc = await doConnect();

  // Subscribe to configured subjects
  for (const subj of config.subjects) {
    const sub = nc.subscribe(subj);
    subs.push(sub);

    (async () => {
      for await (const msg of sub) {
        try {
          const data: AgentMessage = JSON.parse(sc.decode(msg.data));
          // Skip messages from self
          if (data.from === config.agentName) continue;

          // Handle request-reply: if the inbound message has a reply subject,
          // attach it so the processor can respond
          if (msg.reply) {
            data.replyTo = msg.reply;
          }

          onMessage(msg.subject, data);
        } catch (err) {
          logger.warn(`Failed to parse message on ${msg.subject}: ${err}`);
        }
      }
    })();
  }

  return {
    publish(subject: string, msg: AgentMessage): void {
      if (!nc) throw new Error("NATS client not connected");
      nc.publish(subject, sc.encode(JSON.stringify(msg)));
    },

    async request(
      subject: string,
      msg: AgentMessage,
      timeoutMs: number,
    ): Promise<AgentMessage> {
      if (!nc) throw new Error("NATS client not connected");
      const reply = await nc.request(
        subject,
        sc.encode(JSON.stringify(msg)),
        { timeout: timeoutMs },
      );
      return JSON.parse(sc.decode(reply.data));
    },

    async close(): Promise<void> {
      for (const sub of subs) {
        sub.unsubscribe();
      }
      if (nc) {
        await nc.drain();
        nc = null;
      }
    },

    isConnected(): boolean {
      return nc !== null && !nc.isClosed();
    },
  };
}
