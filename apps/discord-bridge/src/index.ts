import { loadConfig } from "./config.js";
import { createDiscordClient } from "./discord-client.js";
import { RoomManager } from "./room-manager.js";
import { createBridge } from "./bridge.js";
import { createDiscordElicitationHandler } from "./elicitation-handler.js";
import { createHttpServer } from "./http-server.js";

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

  // 6. Start HTTP notification server
  const httpServer = createHttpServer(
    config.httpPort,
    bridge,
    elicitHandler,
    logger,
    config.deliberationChannelId,
    discord,
  );
  await httpServer.start();

  logger.info("Discord bridge is running");

  // Graceful shutdown
  const shutdown = async () => {
    logger.info("Shutting down...");
    bridge.stop();
    elicitHandler.destroy();
    await httpServer.stop();
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
