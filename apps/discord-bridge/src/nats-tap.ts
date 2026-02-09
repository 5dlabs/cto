import { connect, StringCodec, type NatsConnection, type Subscription } from "nats";
import type { AgentMessage } from "./types.js";

const sc = StringCodec();

export interface NatsTap {
  close(): Promise<void>;
}

export type MessageHandler = (subject: string, msg: AgentMessage) => void;

/**
 * Subscribe to the `agent.>` wildcard on NATS and emit parsed AgentMessages.
 * Discovery pings/pongs are filtered out -- we only care about real messages.
 */
export async function createNatsTap(
  natsUrl: string,
  onMessage: MessageHandler,
  logger: { info: Function; warn: Function; error: Function },
): Promise<NatsTap> {
  const nc: NatsConnection = await connect({
    servers: natsUrl,
    name: "discord-bridge",
  });

  logger.info(`NATS tap connected to ${natsUrl}`);

  const sub: Subscription = nc.subscribe("agent.>");

  (async () => {
    for await (const msg of sub) {
      try {
        const data: AgentMessage = JSON.parse(sc.decode(msg.data));

        // Skip discovery protocol messages
        if (data.type === "discovery_ping" || data.type === "discovery_pong") {
          continue;
        }

        onMessage(msg.subject, data);
      } catch {
        // Skip malformed messages
      }
    }
  })();

  return {
    async close() {
      sub.unsubscribe();
      await nc.drain();
    },
  };
}
