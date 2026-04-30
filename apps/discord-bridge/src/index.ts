import { loadConfig } from "./config.js";
import { createDiscordClient } from "./discord-client.js";
import { RoomManager } from "./room-manager.js";
import { createBridge } from "./bridge.js";
import { createDiscordElicitationHandler } from "./elicitation-handler.js";
import { createHttpServer } from "./http-server.js";
import { normalizeDiscordMessage } from "./discord-normalizer.js";
import { createPresenceFabric } from "./presence-fabric.js";
import { createPresenceRouter } from "./presence-router.js";

const logger = {
  info: (...args: unknown[]) => console.log(`[bridge]`, ...args),
  warn: (...args: unknown[]) => console.warn(`[bridge]`, ...args),
  error: (...args: unknown[]) => console.error(`[bridge]`, ...args),
};

async function main(): Promise<void> {
  const config = loadConfig();
  logger.info("Starting Discord bridge...");
  logger.info(`  Guild: ${config.guildId}`);
  logger.info(`  HTTP port: ${config.httpPort}`);
  logger.info(`  Linear bridge: ${config.linearBridgeUrl}`);
  logger.info(`  Inactivity timeout: ${config.inactivityTimeoutMs}ms`);
  if (config.presenceRouteStorePath) {
    logger.info(`  Presence route store: ${config.presenceRouteStorePath}`);
  }
  if (config.natsUrl) {
    logger.info(`  Presence NATS URL: ${config.natsUrl}`);
  }

  // 1. Connect to Discord and initialize rooms
  const discord = await createDiscordClient(config.discordToken, logger);
  const channelIds = await discord.initializeRooms(config.guildId, config.categoryName);
  logger.info(`Initialized ${channelIds.length} room channels`);

  // 2. Set up room manager
  const roomManager = new RoomManager();
  roomManager.initialize(channelIds);

  // 3. Create the bridge orchestrator
  const bridge = createBridge(config, discord, roomManager, logger);

  // 4. Create elicitation handler (uses HTTP for cross-cancel to linear-bridge)
  const elicitHandler = createDiscordElicitationHandler(discord, config.linearBridgeUrl, logger);

  // 5. Register interaction handler for buttons/select menus
  discord.onInteraction((interaction) => {
    elicitHandler.handleInteraction(interaction).catch((err) => {
      logger.error("Failed to handle interaction:", err);
    });
  });

  // 6. Create the runtime-neutral presence router foundation
  const presenceFabric = await createPresenceFabric(config.natsUrl, logger);
  const presence = createPresenceRouter(
    discord,
    logger,
    config.presenceRouteStorePath,
    presenceFabric,
    config.presenceSharedToken,
  );

  discord.onMessage((message) => {
    void (async () => {
      const event = normalizeDiscordMessage(message, {
        accountId: config.presenceAccountId,
        defaultAgentId: config.presenceDefaultAgentId,
        botUserId: message.client.user?.id,
      });
      if (!event) {
        return;
      }
      const result = await presence.routeDiscordEvent(event);
      if (result.deliveries.length > 0) {
        logger.info(`Presence routed Discord message ${event.discord.message_id ?? "unknown"} to ${result.deliveries.length} worker(s)`);
      }
    })().catch((err) => {
      logger.warn(`Presence Discord ingress failed: ${err instanceof Error ? err.message : String(err)}`);
    });
  });

  // 7. Start HTTP notification server
  const httpServer = createHttpServer(
    config.httpPort,
    bridge,
    elicitHandler,
    logger,
    config.deliberationChannelId,
    discord,
    presence,
    config.presenceSharedToken,
  );
  await httpServer.start();

  logger.info("Discord bridge is running");

  // Graceful shutdown
  const shutdown = async () => {
    logger.info("Shutting down...");
    bridge.stop();
    elicitHandler.destroy();
    await httpServer.stop();
    await presenceFabric.close();
    discord.destroy();
    process.exit(0);
  };

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

main().catch((err) => {
  logger.error("Fatal error:", err);
  process.exit(1);
});
