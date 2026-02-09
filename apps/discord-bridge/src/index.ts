import { loadConfig } from "./config.js";
import { createDiscordClient } from "./discord-client.js";
import { createNatsTap } from "./nats-tap.js";
import { RoomManager } from "./room-manager.js";
import { createBridge } from "./bridge.js";

const logger = {
  info: (...args: unknown[]) => console.log(`[bridge]`, ...args),
  warn: (...args: unknown[]) => console.warn(`[bridge]`, ...args),
  error: (...args: unknown[]) => console.error(`[bridge]`, ...args),
};

async function main(): Promise<void> {
  const config = loadConfig();
  logger.info("Starting Discord bridge...");
  logger.info(`  Guild: ${config.guildId}`);
  logger.info(`  NATS: ${config.natsUrl}`);
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

  // 4. Connect to NATS and start tapping messages
  const natsTap = await createNatsTap(
    config.natsUrl,
    (subject, msg) => bridge.handleMessage(subject, msg),
    logger,
  );

  logger.info("Discord bridge is running");

  // Graceful shutdown
  const shutdown = async () => {
    logger.info("Shutting down...");
    bridge.stop();
    await natsTap.close();
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
