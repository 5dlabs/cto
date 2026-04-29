import {
  Client,
  GatewayIntentBits,
  ChannelType,
  EmbedBuilder,
  ActionRowBuilder,
  type Guild,
  type TextChannel,
  type CategoryChannel,
  type Message,
  type Interaction,
  type MessageActionRowComponentBuilder,
} from "discord.js";
import { NUM_ROOMS, availableChannelName } from "./types.js";

/** Discord thread name max length */
const THREAD_NAME_MAX = 100;

function sanitizeThreadName(sessionId: string): string {
  const base = `intake-${sessionId}`.replace(/[^a-zA-Z0-9_-]+/g, "-").replace(/^-+|-+$/g, "");
  return base.length <= THREAD_NAME_MAX ? base || "intake-session" : base.slice(0, THREAD_NAME_MAX);
}

export interface DiscordHandle {
  /** Ensure the category and 5 room channels exist; returns channel IDs */
  initializeRooms(guildId: string, categoryName: string): Promise<string[]>;
  /** Rename a channel */
  renameChannel(channelId: string, name: string): Promise<void>;
  /**
   * Get or create a public thread under parentChannelId for this intake session.
   * Returns the thread channel id (usable with postEmbed / postElicitation / updateMessage).
   */
  getOrCreateSessionThread(parentChannelId: string, sessionId: string): Promise<string>;
  /** Post a message embed to a text channel or thread */
  postEmbed(channelId: string, embed: EmbedBuilder): Promise<void>;
  /** Post an embed with interactive components (buttons/select menus) */
  postElicitation(
    channelId: string,
    embed: EmbedBuilder,
    components: ActionRowBuilder<MessageActionRowComponentBuilder>[],
    extraEmbeds?: EmbedBuilder[],
  ): Promise<Message>;
  /** Update an existing message's embed and components */
  updateMessage(
    channelId: string,
    messageId: string,
    embed: EmbedBuilder,
    components?: ActionRowBuilder<MessageActionRowComponentBuilder>[],
  ): Promise<void>;
  /** Send a plain message to a text channel or thread and return its Discord message id. */
  postMessage(channelId: string, content: string): Promise<string>;
  /** Edit a plain message in a text channel or thread. */
  editPlainMessage(channelId: string, messageId: string, content: string): Promise<void>;
  /** Add a reaction to a message in a text channel or thread. */
  addReaction(channelId: string, messageId: string, emoji: string): Promise<void>;
  /** Send a Discord typing indicator to a text channel or thread. */
  sendTyping(channelId: string): Promise<void>;
  /** Register a handler for interaction events (buttons, select menus) */
  onInteraction(handler: (interaction: Interaction) => void): void;
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
      client.once("clientReady", () => resolve());
    }
  });

  logger.info(`Discord client ready as ${client.user?.tag}`);

  const resolvedSessionThreads = new Map<string, string>();
  const inFlightSessionThreads = new Map<string, Promise<string>>();

  async function getOrCreateSessionThreadImpl(
    parentChannelId: string,
    sessionId: string,
  ): Promise<string> {
    const key = `${parentChannelId}:${sessionId}`;
    const cached = resolvedSessionThreads.get(key);
    if (cached) {
      const ch = await client.channels.fetch(cached).catch(() => null);
      if (ch && ch.isThread()) return cached;
      resolvedSessionThreads.delete(key);
    }

    const parent = await client.channels.fetch(parentChannelId);
    if (!parent || parent.type !== ChannelType.GuildText) {
      throw new Error(`Parent channel ${parentChannelId} not found or not a text channel`);
    }
    const textParent = parent as TextChannel;
    const threadName = sanitizeThreadName(sessionId);

    const active = await textParent.threads.fetchActive();
    const existing = active.threads.find((t) => t.name === threadName);
    if (existing) {
      resolvedSessionThreads.set(key, existing.id);
      logger.info(`Session thread reuse: ${threadName} (${existing.id})`);
      return existing.id;
    }

    const created = await textParent.threads.create({
      name: threadName,
      autoArchiveDuration: 10_080,
      reason: `Intake deliberation session ${sessionId}`,
    });
    resolvedSessionThreads.set(key, created.id);
    logger.info(`Session thread created: ${threadName} (${created.id})`);
    return created.id;
  }

  return {
    async getOrCreateSessionThread(parentChannelId: string, sessionId: string): Promise<string> {
      const key = `${parentChannelId}:${sessionId}`;
      let p = inFlightSessionThreads.get(key);
      if (p) return p;
      p = getOrCreateSessionThreadImpl(parentChannelId, sessionId);
      inFlightSessionThreads.set(key, p);
      try {
        return await p;
      } finally {
        inFlightSessionThreads.delete(key);
      }
    },

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
        if (!channel) return;
        if (channel.type === ChannelType.GuildText) {
          await (channel as TextChannel).send({ embeds: [embed] });
        } else if (channel.isThread()) {
          await channel.send({ embeds: [embed] });
        }
      } catch (err) {
        logger.warn(`Failed to post embed to channel ${channelId}: ${err}`);
      }
    },

    async postElicitation(channelId, embed, components, extraEmbeds): Promise<Message> {
      const embeds = extraEmbeds ?? [embed];
      const channel = await client.channels.fetch(channelId);
      if (!channel) {
        throw new Error(`Channel ${channelId} not found`);
      }
      if (channel.type === ChannelType.GuildText) {
        return (channel as TextChannel).send({ embeds, components });
      }
      if (channel.isThread()) {
        return channel.send({ embeds, components });
      }
      throw new Error(`Channel ${channelId} is not a guild text channel or thread`);
    },

    async updateMessage(channelId, messageId, embed, components): Promise<void> {
      try {
        const channel = await client.channels.fetch(channelId);
        if (!channel) return;
        if (channel.type === ChannelType.GuildText) {
          const message = await (channel as TextChannel).messages.fetch(messageId);
          await message.edit({
            embeds: [embed],
            components: components ?? [],
          });
        } else if (channel.isThread()) {
          const message = await channel.messages.fetch(messageId);
          await message.edit({
            embeds: [embed],
            components: components ?? [],
          });
        }
      } catch (err) {
        logger.warn(`Failed to update message ${messageId} in ${channelId}: ${err}`);
      }
    },

    async postMessage(channelId, content): Promise<string> {
      const channel = await client.channels.fetch(channelId);
      if (!channel) {
        throw new Error(`Channel ${channelId} not found`);
      }
      if (channel.type === ChannelType.GuildText) {
        const sent = await (channel as TextChannel).send({ content });
        return sent.id;
      }
      if (channel.isThread()) {
        const sent = await channel.send({ content });
        return sent.id;
      }
      throw new Error(`Channel ${channelId} is not a guild text channel or thread`);
    },

    async editPlainMessage(channelId, messageId, content): Promise<void> {
      const channel = await client.channels.fetch(channelId);
      if (!channel) {
        throw new Error(`Channel ${channelId} not found`);
      }
      if (channel.type === ChannelType.GuildText) {
        const message = await (channel as TextChannel).messages.fetch(messageId);
        await message.edit({ content });
        return;
      }
      if (channel.isThread()) {
        const message = await channel.messages.fetch(messageId);
        await message.edit({ content });
        return;
      }
      throw new Error(`Channel ${channelId} is not a guild text channel or thread`);
    },

    async addReaction(channelId, messageId, emoji): Promise<void> {
      const channel = await client.channels.fetch(channelId);
      if (!channel) {
        throw new Error(`Channel ${channelId} not found`);
      }
      if (channel.type === ChannelType.GuildText) {
        const message = await (channel as TextChannel).messages.fetch(messageId);
        await message.react(emoji);
        return;
      }
      if (channel.isThread()) {
        const message = await channel.messages.fetch(messageId);
        await message.react(emoji);
        return;
      }
      throw new Error(`Channel ${channelId} is not a guild text channel or thread`);
    },

    async sendTyping(channelId): Promise<void> {
      const channel = await client.channels.fetch(channelId);
      if (!channel) {
        throw new Error(`Channel ${channelId} not found`);
      }
      if (channel.type === ChannelType.GuildText) {
        await (channel as TextChannel).sendTyping();
        return;
      }
      if (channel.isThread()) {
        await channel.sendTyping();
        return;
      }
      throw new Error(`Channel ${channelId} is not a guild text channel or thread`);
    },

    onInteraction(handler): void {
      client.on("interactionCreate", handler);
    },

    destroy(): void {
      client.destroy();
      logger.info("Discord client destroyed");
    },
  };
}
