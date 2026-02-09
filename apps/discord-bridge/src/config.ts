export interface BridgeConfig {
  /** Discord bot token */
  discordToken: string;
  /** Discord guild (server) ID */
  guildId: string;
  /** NATS server URL */
  natsUrl: string;
  /** Inactivity timeout in ms before a conversation room is freed (default: 1 hour) */
  inactivityTimeoutMs: number;
  /** Category name for bot conversation channels */
  categoryName: string;
}

export function loadConfig(): BridgeConfig {
  const discordToken = process.env.DISCORD_BRIDGE_TOKEN;
  if (!discordToken) {
    throw new Error("DISCORD_BRIDGE_TOKEN environment variable is required");
  }

  return {
    discordToken,
    guildId: process.env.GUILD_ID ?? "1409006087331512342",
    natsUrl: process.env.NATS_URL ?? "nats://localhost:4222",
    inactivityTimeoutMs: parseInt(process.env.INACTIVITY_TIMEOUT_MS ?? "3600000", 10),
    categoryName: process.env.CATEGORY_NAME ?? "Bot Conversations",
  };
}
