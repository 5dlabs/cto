/**
 * Elicitation Handler — Discord Bridge
 *
 * Bidirectional human-in-the-loop decision flow:
 *   HTTP POST /elicitation  → Discord embed + buttons/select menu
 *   Discord interaction     → HTTP callback to linear-bridge + cross-cancel
 *   HTTP POST /elicitation/cancel → Update Discord message (disable components)
 */

import {
  EmbedBuilder,
  ActionRowBuilder,
  ButtonBuilder,
  StringSelectMenuBuilder,
  ButtonStyle,
  type Interaction,
  type MessageActionRowComponentBuilder,
} from "discord.js";
import type { DiscordHandle } from "./discord-client.js";
import type {
  ElicitationRequest,
  ElicitationCancel,
  ElicitationResponse,
} from "./elicitation-types.js";
import {
  createElicitationResponse,
  createElicitationCancel,
} from "./elicitation-types.js";

// =============================================================================
// Types
// =============================================================================

interface PendingElicitation {
  request: ElicitationRequest;
  channelId: string;
  messageId: string;
  timeoutTimer?: ReturnType<typeof setTimeout>;
}

// =============================================================================
// Constants
// =============================================================================

const MAX_BUTTONS_PER_ROW = 5;
const REDELIBERATE_PREFIX = "elicit_redelib:";
const SELECT_PREFIX = "elicit:";
const DISCORD_CUSTOM_ID_MAX = 100;

/** Truncate a custom ID to Discord's 100-char limit */
function safeCustomId(id: string): string {
  return id.length <= DISCORD_CUSTOM_ID_MAX ? id : id.slice(0, DISCORD_CUSTOM_ID_MAX);
}

// =============================================================================
// Handler
// =============================================================================

export interface DiscordElicitationHandler {
  handleRequest(request: ElicitationRequest): Promise<void>;
  handleInteraction(interaction: Interaction): Promise<void>;
  handleCancel(cancel: ElicitationCancel): Promise<void>;
  destroy(): void;
}

export function createDiscordElicitationHandler(
  discord: DiscordHandle,
  linearBridgeUrl: string,
  logger: { info: Function; warn: Function; error: Function },
): DiscordElicitationHandler {
  const pending = new Map<string, PendingElicitation>();

  // ─── HTTP helpers ─────────────────────────────────────────────────────

  async function postToLinearBridge(path: string, data: unknown): Promise<void> {
    try {
      await fetch(`${linearBridgeUrl}${path}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });
    } catch (err) {
      logger.warn(`Failed to POST to linear-bridge ${path}: ${err}`);
    }
  }

  // ─── Build embed ────────────────────────────────────────────────────────

  function buildEmbed(req: ElicitationRequest, resolved?: string): EmbedBuilder {
    const embed = new EmbedBuilder()
      .setTitle(resolved ? `Resolved: ${req.question}` : `Decision Required: ${req.question}`)
      .setColor(resolved ? 0x57f287 : req.vote_summary.escalated ? 0xed4245 : 0x5865f2)
      .setTimestamp(new Date(req.timestamp));

    // Vote breakdown
    const voteLines = req.options.map((opt) => {
      const count = opt.vote_count ?? req.vote_summary.tally[opt.value] ?? 0;
      const isWinner = opt.value === req.recommended_option;
      return `${isWinner ? "**" : ""}${opt.label}: ${count} vote${count !== 1 ? "s" : ""}${isWinner ? " (recommended)**" : ""}`;
    });
    embed.addFields({ name: "Vote Breakdown", value: voteLines.join("\n"), inline: false });

    // Consensus
    const { consensus_strength, total_voters, escalated } = req.vote_summary;
    embed.addFields({
      name: "Consensus",
      value: `${Math.round(consensus_strength * 100)}% (${total_voters} voters)${escalated ? " — **ESCALATED**" : ""}`,
      inline: true,
    });

    embed.addFields({
      name: "Category",
      value: req.category,
      inline: true,
    });

    // Voter reasoning
    if (req.vote_summary.voter_notes?.length) {
      const notes = req.vote_summary.voter_notes
        .map((n) => `**${n.voter_id}** → ${n.chose}: ${n.reasoning}`)
        .join("\n");
      embed.addFields({
        name: "Voter Reasoning",
        value: notes.length > 1024 ? notes.slice(0, 1021) + "..." : notes,
      });
    }

    if (resolved) {
      embed.addFields({ name: "Resolution", value: resolved });
      embed.setFooter({ text: `Session: ${req.session_id}` });
    } else {
      embed.setFooter({ text: `Session: ${req.session_id} | Decision: ${req.decision_id}` });
    }

    return embed;
  }

  // ─── Build components ───────────────────────────────────────────────────

  function buildComponents(
    req: ElicitationRequest,
  ): ActionRowBuilder<MessageActionRowComponentBuilder>[] {
    const rows: ActionRowBuilder<MessageActionRowComponentBuilder>[] = [];

    if (req.options.length <= MAX_BUTTONS_PER_ROW) {
      const row = new ActionRowBuilder<MessageActionRowComponentBuilder>();
      for (const opt of req.options) {
        const isRecommended = opt.value === req.recommended_option;
        row.addComponents(
          new ButtonBuilder()
            .setCustomId(safeCustomId(`${SELECT_PREFIX}${req.elicitation_id}:${opt.value}`))
            .setLabel(opt.label)
            .setStyle(isRecommended ? ButtonStyle.Success : ButtonStyle.Secondary),
        );
      }
      rows.push(row);
    } else {
      const menu = new StringSelectMenuBuilder()
        .setCustomId(safeCustomId(`${SELECT_PREFIX}${req.elicitation_id}`))
        .setPlaceholder("Select an option...")
        .addOptions(
          req.options.map((opt) => ({
            label: opt.label,
            value: opt.value,
            default: opt.value === req.recommended_option,
            description: opt.description?.slice(0, 100),
          })),
        );
      rows.push(new ActionRowBuilder<MessageActionRowComponentBuilder>().addComponents(menu));
    }

    // Re-deliberation button (separate row)
    if (req.allow_redeliberation) {
      const redelibRow = new ActionRowBuilder<MessageActionRowComponentBuilder>().addComponents(
        new ButtonBuilder()
          .setCustomId(safeCustomId(`${REDELIBERATE_PREFIX}${req.elicitation_id}`))
          .setLabel("Re-deliberate")
          .setStyle(ButtonStyle.Danger),
      );
      rows.push(redelibRow);
    }

    return rows;
  }

  // ─── Resolve helper (sends response + cross-cancel via HTTP) ──────────

  async function resolveElicitation(
    elicitationId: string,
    response: ElicitationResponse,
  ): Promise<void> {
    // Post response to linear-bridge callback (if run is registered, this triggers Lobster resume)
    const req = pending.get(elicitationId)?.request;
    const runId = req?.metadata?.run_id;
    if (runId) {
      await postToLinearBridge(`/runs/${runId}/callback`, response);
    }

    // Cross-cancel: tell linear-bridge this elicitation is resolved
    const cancel = createElicitationCancel(
      elicitationId,
      'discord',
      response.selected_option,
    );
    await postToLinearBridge('/elicitation/cancel', cancel);
  }

  // ─── HTTP → Discord ────────────────────────────────────────────────────

  async function handleRequest(request: ElicitationRequest): Promise<void> {
    const { elicitation_id, discord_channel_id } = request;

    if (!discord_channel_id) {
      logger.warn(`Elicitation ${elicitation_id}: no discord_channel_id — skipping Discord rendering`);
      return;
    }

    const embed = buildEmbed(request);

    if (request.informational) {
      await discord.postEmbed(discord_channel_id, embed);
      logger.info(`Elicitation ${elicitation_id}: posted informational embed`);
      return;
    }

    // Interactive: post with buttons/select menu
    const components = buildComponents(request);
    let message;
    try {
      message = await discord.postElicitation(discord_channel_id, embed, components);
    } catch (err) {
      logger.error(`Elicitation ${elicitation_id}: failed to post — ${err}`);
      return;
    }

    // Set up timeout
    let timeoutTimer: ReturnType<typeof setTimeout> | undefined;
    if (request.timeout_seconds > 0 && request.recommended_option) {
      const graceMs = 5000;
      timeoutTimer = setTimeout(async () => {
        const p = pending.get(elicitation_id);
        if (!p) return;
        pending.delete(elicitation_id);

        logger.info(`Elicitation ${elicitation_id}: timeout — auto-selecting "${request.recommended_option}"`);

        const response = createElicitationResponse(elicitation_id, "discord", "system:timeout", {
          selectedOption: request.recommended_option,
        });
        await resolveElicitation(elicitation_id, response);

        const resolvedEmbed = buildEmbed(request, `Timeout — auto-selected: ${request.recommended_option}`);
        await discord.updateMessage(p.channelId, p.messageId, resolvedEmbed);
      }, request.timeout_seconds * 1000 + graceMs);
    }

    pending.set(elicitation_id, {
      request,
      channelId: discord_channel_id,
      messageId: message.id,
      timeoutTimer,
    });

    logger.info(`Elicitation ${elicitation_id}: posted interactive message ${message.id}`);
  }

  // ─── Discord → HTTP ────────────────────────────────────────────────────

  async function handleInteraction(interaction: Interaction): Promise<void> {
    // Handle button clicks
    if (interaction.isButton()) {
      const customId = interaction.customId;

      // Re-deliberation button
      if (customId.startsWith(REDELIBERATE_PREFIX)) {
        const elicitationId = customId.slice(REDELIBERATE_PREFIX.length);
        const entry = pending.get(elicitationId);

        if (!entry) {
          await interaction.reply({ content: "This decision has already been resolved.", ephemeral: true });
          return;
        }

        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        pending.delete(elicitationId);

        await interaction.deferUpdate();

        const response = createElicitationResponse(elicitationId, "discord", interaction.user.id, {
          userContext: "Human requested re-deliberation via Discord",
        });
        await resolveElicitation(elicitationId, response);

        const resolvedEmbed = buildEmbed(entry.request, `Re-deliberation requested by <@${interaction.user.id}>`);
        await discord.updateMessage(entry.channelId, entry.messageId, resolvedEmbed);

        logger.info(`Elicitation ${elicitationId}: re-deliberation requested by ${interaction.user.tag}`);
        return;
      }

      // Selection button
      if (customId.startsWith(SELECT_PREFIX)) {
        const parts = customId.slice(SELECT_PREFIX.length);
        const colonIdx = parts.indexOf(":");
        if (colonIdx === -1) return;

        const elicitationId = parts.slice(0, colonIdx);
        const selectedValue = parts.slice(colonIdx + 1);
        const entry = pending.get(elicitationId);

        if (!entry) {
          await interaction.reply({ content: "This decision has already been resolved.", ephemeral: true });
          return;
        }

        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        pending.delete(elicitationId);

        await interaction.deferUpdate();

        const response = createElicitationResponse(elicitationId, "discord", interaction.user.id, {
          selectedOption: selectedValue,
        });
        await resolveElicitation(elicitationId, response);

        const optLabel = entry.request.options.find((o) => o.value === selectedValue)?.label ?? selectedValue;
        const resolvedEmbed = buildEmbed(entry.request, `${optLabel} — selected by <@${interaction.user.id}>`);
        await discord.updateMessage(entry.channelId, entry.messageId, resolvedEmbed);

        logger.info(`Elicitation ${elicitationId}: "${selectedValue}" selected by ${interaction.user.tag}`);
        return;
      }
    }

    // Handle select menu
    if (interaction.isStringSelectMenu() && interaction.customId.startsWith(SELECT_PREFIX)) {
      const elicitationId = interaction.customId.slice(SELECT_PREFIX.length);
      const selectedValue = interaction.values[0];
      const entry = pending.get(elicitationId);

      if (!entry) {
        await interaction.reply({ content: "This decision has already been resolved.", ephemeral: true });
        return;
      }

      if (!selectedValue) return;

      if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
      pending.delete(elicitationId);

      await interaction.deferUpdate();

      const response = createElicitationResponse(elicitationId, "discord", interaction.user.id, {
        selectedOption: selectedValue,
      });
      await resolveElicitation(elicitationId, response);

      const optLabel = entry.request.options.find((o) => o.value === selectedValue)?.label ?? selectedValue;
      const resolvedEmbed = buildEmbed(entry.request, `${optLabel} — selected by <@${interaction.user.id}>`);
      await discord.updateMessage(entry.channelId, entry.messageId, resolvedEmbed);

      logger.info(`Elicitation ${elicitationId}: "${selectedValue}" selected by ${interaction.user.tag}`);
    }
  }

  // ─── Cancel from Linear ─────────────────────────────────────────────────

  async function handleCancel(cancel: ElicitationCancel): Promise<void> {
    const entry = pending.get(cancel.elicitation_id);
    if (!entry) return;

    if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
    pending.delete(cancel.elicitation_id);

    const resolution = cancel.selected_option
      ? `Resolved via Linear: ${cancel.selected_option}`
      : "Resolved via Linear (re-deliberation requested)";

    const resolvedEmbed = buildEmbed(entry.request, resolution);
    await discord.updateMessage(entry.channelId, entry.messageId, resolvedEmbed);

    logger.info(`Elicitation ${cancel.elicitation_id}: cancelled — ${resolution}`);
  }

  // ─── Cleanup ────────────────────────────────────────────────────────────

  function destroy(): void {
    for (const entry of pending.values()) {
      if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
    }
    pending.clear();
  }

  return { handleRequest, handleInteraction, handleCancel, destroy };
}
