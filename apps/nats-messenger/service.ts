import type { NatsConfig } from "./types";
import { createNatsClient, type NatsClientHandle } from "./client";
import { processInboundMessage } from "./processor";
import { deliverToAgent } from "./actions";

export interface NatsServiceResult {
  /** Background service descriptor for api.registerService() */
  service: {
    id: string;
    name: string;
    start(): Promise<void>;
    stop(): Promise<void>;
  };
  /** Client handle for the tool to publish/request */
  handle: () => NatsClientHandle | null;
}

export function createService(
  config: NatsConfig,
  runtime: any,
  logger: { info: Function; warn: Function; error: Function },
): NatsServiceResult {
  let client: NatsClientHandle | null = null;

  const service = {
    id: "nats-messenger-service",
    name: "NATS Messenger",

    async start(): Promise<void> {
      if (!config.enabled) {
        logger.info("NATS messenger disabled by config");
        return;
      }

      logger.info(`Starting NATS messenger for agent "${config.agentName}"`);
      logger.info(`Subscribing to: ${config.subjects.join(", ")}`);

      client = await createNatsClient(
        config,
        (subject, msg) => {
          const processed = processInboundMessage(
            subject,
            msg,
            config.agentName,
          );
          deliverToAgent(runtime, processed, logger);
        },
        logger,
      );
    },

    async stop(): Promise<void> {
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
  };
}
