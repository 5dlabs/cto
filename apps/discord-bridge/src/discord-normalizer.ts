import type { PresenceDiscordEvent } from "./presence-types.js";

export interface DiscordNormalizerOptions {
  accountId: string;
  defaultAgentId: string;
  botUserId?: string;
}

interface DiscordLikeUser {
  id?: string;
  username?: string;
  globalName?: string | null;
  bot?: boolean;
}

interface DiscordLikeAttachment {
  id?: string;
  url?: string;
  contentType?: string | null;
  content_type?: string;
  name?: string | null;
  filename?: string;
  size?: number;
  spoiler?: boolean;
}

interface DiscordLikeChannel {
  id?: string;
  type?: unknown;
  parentId?: string | null;
  isThread?: () => boolean;
  isDMBased?: () => boolean;
}

interface DiscordLikeMessage {
  id?: string;
  content?: string | null;
  author?: DiscordLikeUser | null;
  guildId?: string | null;
  channelId?: string;
  channel?: DiscordLikeChannel | null;
  mentions?: {
    users?: unknown;
  };
  attachments?: unknown;
  reference?: {
    messageId?: string | null;
    channelId?: string | null;
    guildId?: string | null;
  } | null;
}

function valuesFromCollection<T>(collection: unknown): T[] {
  if (!collection) return [];
  if (Array.isArray(collection)) return collection as T[];
  if (collection instanceof Map) return [...collection.values()] as T[];
  if (typeof collection === "object" && "values" in collection && typeof (collection as { values: unknown }).values === "function") {
    return [...((collection as { values: () => Iterable<T> }).values())];
  }
  return [];
}

function channelIsThread(channel: DiscordLikeChannel | null | undefined): boolean {
  return Boolean(channel?.isThread?.()) || String(channel?.type ?? "").toLowerCase().includes("thread");
}

function channelIsDm(channel: DiscordLikeChannel | null | undefined, guildId: string | null | undefined): boolean {
  return !guildId || Boolean(channel?.isDMBased?.()) || String(channel?.type ?? "").toLowerCase().includes("dm");
}

function normalizeAttachments(attachments: unknown): PresenceDiscordEvent["attachments"] {
  return valuesFromCollection<DiscordLikeAttachment>(attachments)
    .map((attachment) => ({
      url: attachment.url ?? "",
      id: attachment.id,
      content_type: attachment.contentType ?? attachment.content_type,
      filename: attachment.name ?? attachment.filename,
      size: attachment.size,
      spoiler: attachment.spoiler,
    }))
    .filter((attachment) => attachment.url);
}

function normalizeAgentMention(value: string | null | undefined): string | undefined {
  const normalized = value?.trim().toLowerCase();
  return normalized || undefined;
}

function normalizeMentionedAgentIds(message: DiscordLikeMessage, options: DiscordNormalizerOptions): string[] | undefined {
  const mentioned = valuesFromCollection<DiscordLikeUser>(message.mentions?.users)
    .filter((user) => user.id !== options.botUserId)
    .flatMap((user) => [user.id, normalizeAgentMention(user.username), normalizeAgentMention(user.globalName)])
    .filter((id): id is string => Boolean(id));
  return mentioned.length ? [...new Set(mentioned)] : undefined;
}

export function normalizeDiscordMessage(
  message: DiscordLikeMessage,
  options: DiscordNormalizerOptions,
): PresenceDiscordEvent | undefined {
  if (message.author?.bot) {
    return undefined;
  }

  const channel = message.channel;
  const rawChannelId = message.channelId ?? channel?.id;
  if (!rawChannelId) {
    throw new Error("Discord message is missing channel id");
  }

  const isThread = channelIsThread(channel);
  const isDm = channelIsDm(channel, message.guildId);
  const channelId = isThread ? (channel?.parentId ?? rawChannelId) : rawChannelId;
  const authorName = message.author?.globalName ?? message.author?.username ?? undefined;

  return {
    schema: "cto.presence.v1",
    event_type: "message",
    discord: {
      account_id: options.accountId,
      guild_id: message.guildId ?? undefined,
      channel_id: channelId,
      thread_id: isThread ? rawChannelId : undefined,
      message_id: message.id,
      reference_message_id: message.reference?.messageId ?? undefined,
      reference_channel_id: message.reference?.channelId ?? undefined,
      reference_guild_id: message.reference?.guildId ?? undefined,
      user_id: message.author?.id,
      user_name: authorName,
      chat_type: isDm ? "dm" : isThread ? "thread" : "group",
      parent_channel_id: isThread ? (channel?.parentId ?? undefined) : undefined,
      mentioned_agent_ids: normalizeMentionedAgentIds(message, options),
    },
    text: message.content ?? undefined,
    attachments: normalizeAttachments(message.attachments),
  };
}
