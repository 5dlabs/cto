import {
  Client,
  GatewayIntentBits,
  ChannelType,
  EmbedBuilder,
  type Guild,
  type TextChannel,
  type CategoryChannel,
} from "discord.js";
import { NUM_ROOMS, availableChannelName } from "./types.js";

export interface DiscordHandle {
  /** Ensure the category and 5 room channels exist; returns channel IDs */
  initializeRooms(guildId: string, categoryName: string): Promise<string[]>;
  /** Rename a channel */
  renameChannel(channelId: string, name: string): Promise<void>;
  /** Post a message embed to a channel */
  postEmbed(channelId: string, embed: EmbedBuilder): Promise<void>;
  /** Disconnect from Discord */
  destroy(): void;
}

export async function createDiscordClient(
  token: string,
  logger: { info: Function; warn: Function; error: Function },
): Promise<DiscordHandle> {
  const client = new Client({
    intents: [GatewayIntentBits.Guilds],
  });

  await client.login(token);
  logger.info("Discord client logged in");

  // Wait for the client to be ready
  await new Promise<void>((resolve) => {
    if (client.isReady()) {
      resolve();
    } else {
      client.once("ready", () => resolve());
    }
  });

  logger.info(`Discord client ready as ${client.user?.tag}`);

  return {
    async initializeRooms(guildId: string, categoryName: string): Promise<string[]> {
      const guild: Guild = await client.guilds.fetch(guildId);
      const channels = await guild.channels.fetch();

      // Find or create the category
      let category = channels.find(
        (c): c is CategoryChannel =>
          c !== null && c.type === ChannelType.GuildCategory && c.name === categoryName,
      ) as CategoryChannel | undefined;

      if (!category) {
        category = await guild.channels.create({
          name: categoryName,
          type: ChannelType.GuildCategory,
        });
        logger.info(`Created category "${categoryName}" (${category.id})`);
      }

      // Find or create room channels under the category
      const roomChannelIds: string[] = [];

      for (let i = 0; i < NUM_ROOMS; i++) {
        const defaultName = availableChannelName(i);
        // Check existing channels under the category
        const existing = channels.find(
          (c): c is TextChannel =>
            c !== null &&
            c.type === ChannelType.GuildText &&
            c.parentId === category!.id &&
            (c.name === defaultName || c.name.startsWith(`room-${i}-`)),
        );

        if (existing) {
          roomChannelIds.push(existing.id);
          logger.info(`Found existing room channel: ${existing.name} (${existing.id})`);
        } else {
          const newChannel = await guild.channels.create({
            name: defaultName,
            type: ChannelType.GuildText,
            parent: category.id,
          });
          roomChannelIds.push(newChannel.id);
          logger.info(`Created room channel: ${defaultName} (${newChannel.id})`);
        }
      }

      return roomChannelIds;
    },

    async renameChannel(channelId: string, name: string): Promise<void> {
      try {
        const channel = await client.channels.fetch(channelId);
        if (channel && channel.type === ChannelType.GuildText) {
          await (channel as TextChannel).setName(name);
        }
      } catch (err) {
        logger.warn(`Failed to rename channel ${channelId} to "${name}": ${err}`);
      }
    },

    async postEmbed(channelId: string, embed: EmbedBuilder): Promise<void> {
      try {
        const channel = await client.channels.fetch(channelId);
        if (channel && channel.type === ChannelType.GuildText) {
          await (channel as TextChannel).send({ embeds: [embed] });
        }
      } catch (err) {
        logger.warn(`Failed to post embed to channel ${channelId}: ${err}`);
      }
    },

    destroy(): void {
      client.destroy();
      logger.info("Discord client destroyed");
    },
  };
}
