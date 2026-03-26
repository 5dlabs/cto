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
  DesignReviewRequest,
  DesignReviewResponse,
} from "./elicitation-types.js";
import {
  createElicitationResponse,
  createElicitationCancel,
} from "./elicitation-types.js";
import { createHash } from "node:crypto";
import { createBridgeStateDb, defaultBridgeStateDbPath, defaultWorkspaceRoot } from "./state/bridge-state-db.js";

// =============================================================================
// Types
// =============================================================================

interface PendingElicitation {
  request: ElicitationRequest;
  channelId: string;
  messageId: string;
  interactionKey: string;
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

/** Stable short key to keep Discord custom IDs well under limits. */
function interactionKeyFor(elicitationId: string): string {
  return createHash("sha1").update(elicitationId).digest("hex").slice(0, 16);
}

function describeInteraction(interaction: Interaction): string {
  const kind = interaction.isButton()
    ? "button"
    : interaction.isStringSelectMenu()
      ? "select"
      : interaction.type.toString();
  const customId =
    interaction.isButton() || interaction.isStringSelectMenu()
      ? interaction.customId
      : "n/a";
  return `${kind} customId=${customId} deferred=${interaction.deferred} replied=${interaction.replied}`;
}

// =============================================================================
// Handler
// =============================================================================

export interface DiscordElicitationHandler {
  handleRequest(request: ElicitationRequest): Promise<void>;
  handleDesignReview(request: DesignReviewRequest): Promise<void>;
  handleInteraction(interaction: Interaction): Promise<void>;
  handleCancel(cancel: ElicitationCancel): Promise<void>;
  getStatus(elicitationId: string): { active: boolean; known: boolean };
  getDecisionHistory(sessionId?: string, limit?: number): unknown[];
  getSessionHistory(limit?: number, status?: string): unknown[];
  getWaitingSessions(limit?: number): unknown[];
  getDecisionAudit(elicitationId: string, bridge?: string): unknown;
  getDesignHistory(sessionId?: string, limit?: number): unknown[];
  recordDesignSnapshot(snapshot: Record<string, unknown>): void;
  destroy(): void;
}

export function createDiscordElicitationHandler(
  discord: DiscordHandle,
  linearBridgeUrl: string,
  logger: { info: Function; warn: Function; error: Function },
): DiscordElicitationHandler {
  const pending = new Map<string, PendingElicitation>();
  const pendingByInteractionKey = new Map<string, string>();
  const resolved = new Set<string>();
  const acknowledgedInteractionIds = new Set<string>();
  const stateDb = createBridgeStateDb(defaultBridgeStateDbPath());
  const stateFile = `${defaultWorkspaceRoot()}/.intake/discord-elicitation-state.json`;

  async function persistState(): Promise<void> {
    for (const [elicitationId, entry] of pending.entries()) {
      stateDb.saveElicitationPending({
        bridge: "discord",
        elicitationId,
        request: entry.request,
        status: "active",
      });
      stateDb.upsertProviderMessageRef("discord", elicitationId, {
        channelId: entry.channelId,
        messageId: entry.messageId,
        interactionKey: entry.interactionKey,
      });
      stateDb.setSessionStatus(entry.request.session_id, "waiting_user");
    }
    for (const id of resolved) {
      stateDb.markElicitationResolved("discord", id, undefined, undefined, undefined, "discord", "resolved");
    }
  }

  async function loadState(): Promise<void> {
    stateDb.importLegacyDiscordStateJson(stateFile, logger);
    const active = stateDb.getActiveElicitations("discord");
    for (const row of active) {
      const interactionKey = row.interactionKey ?? interactionKeyFor(row.elicitationId);
      pending.set(row.elicitationId, {
        request: row.request,
        channelId: row.channelId ?? row.request.discord_channel_id ?? "",
        messageId: row.messageId ?? "",
        interactionKey,
      });
      pendingByInteractionKey.set(interactionKey, row.elicitationId);
    }
    if (active.length > 0) {
      logger.info(`Recovered ${active.length} pending elicitation cards after restart`);
    }
  }

  void loadState();

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

  async function acknowledgeComponentInteraction(interaction: Interaction): Promise<void> {
    if (!(interaction.isButton() || interaction.isStringSelectMenu())) return;
    const interactionId = "id" in interaction ? interaction.id : undefined;
    const token = "token" in interaction ? interaction.token : undefined;
    if (!interactionId || !token) return;
    if (acknowledgedInteractionIds.has(interactionId) || interaction.deferred || interaction.replied) return;
    logger.info(`Ack start: ${describeInteraction(interaction)}`);
    const res = await fetch(
      `https://discord.com/api/v10/interactions/${interactionId}/${token}/callback`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ type: 6 }),
      },
    );
    if (res.ok) {
      acknowledgedInteractionIds.add(interactionId);
      logger.info(`Ack success (${res.status}): ${describeInteraction(interaction)}`);
      return;
    }
    const body = await res.text().catch(() => "");
    if (res.status === 400 && body.includes("40060")) {
      acknowledgedInteractionIds.add(interactionId);
      logger.warn(`Ack already acknowledged (${res.status}): ${describeInteraction(interaction)} body=${body}`);
      return;
    }
    throw new Error(`Ack failed (${res.status}): ${body || "no response body"}`);
  }

  async function finalizeInteractionUpdate(
    interaction: Interaction,
    channelId: string,
    messageId: string,
    embed: EmbedBuilder,
    components: ActionRowBuilder<MessageActionRowComponentBuilder>[] = [],
  ): Promise<void> {
    await discord.updateMessage(channelId, messageId, embed, components);
  }

  async function safeEphemeralReply(interaction: Interaction, content: string): Promise<void> {
    const interactionId = "id" in interaction ? interaction.id : undefined;
    const token = "token" in interaction ? interaction.token : undefined;
    const applicationId = "applicationId" in interaction ? interaction.applicationId : undefined;
    const alreadyAcknowledged =
      (interactionId && acknowledgedInteractionIds.has(interactionId)) ||
      ((interaction.isButton() || interaction.isStringSelectMenu()) && (interaction.deferred || interaction.replied));
    if (alreadyAcknowledged && token && applicationId) {
      try {
        const res = await fetch(
          `https://discord.com/api/v10/webhooks/${applicationId}/${token}`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ content, flags: 64 }),
          },
        );
        if (res.ok) {
          logger.info(`Ephemeral webhook follow-up success: ${describeInteraction(interaction)}`);
          return;
        }
        logger.warn(`Ephemeral webhook follow-up rejected (${res.status}): ${describeInteraction(interaction)}`);
      } catch (err) {
        logger.warn(`Ephemeral webhook follow-up failed: ${err}`);
      }
    }
    if (!interactionId || !token) return;
    try {
      const res = await fetch(
        `https://discord.com/api/v10/interactions/${interactionId}/${token}/callback`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            type: 4,
            data: { content, flags: 64 },
          }),
        },
      );
      if (res.ok) {
        logger.info(`Ephemeral callback success: ${describeInteraction(interaction)}`);
        return;
      }
      logger.warn(`Ephemeral callback rejected (${res.status}): ${describeInteraction(interaction)}`);
    } catch (err) {
      logger.warn(`Ephemeral callback threw: ${err}`);
      // Continue into defer fallback below.
    }

    try {
      await acknowledgeComponentInteraction(interaction);
      logger.info(`Ephemeral fallback ack success: ${describeInteraction(interaction)}`);
    } catch (err) {
      logger.warn(`Ephemeral fallback ack failed: ${err}`);
      // Best-effort; interaction may have already expired.
    }
  }

  function buildExpiredCardEmbed(message: string): EmbedBuilder {
    return new EmbedBuilder()
      .setColor(0x80848e)
      .setTitle("Decision Card Expired")
      .setDescription(message)
      .setTimestamp(new Date());
  }

  async function expireInteractionCard(interaction: Interaction, message: string): Promise<void> {
    if (!(interaction.isButton() || interaction.isStringSelectMenu())) {
      await safeEphemeralReply(interaction, message);
      return;
    }
    const expiredEmbed = buildExpiredCardEmbed(message);
    await finalizeInteractionUpdate(
      interaction,
      interaction.channelId,
      interaction.message.id,
      expiredEmbed,
      [],
    );
  }

  // ─── Build embed ────────────────────────────────────────────────────────

  function truncField(text: string, max = 1024): string {
    return text.length <= max ? text : text.slice(0, max - 3) + "...";
  }

  function labelForChoice(req: ElicitationRequest, choice: string): string {
    const opt = req.options.find((o) => o.value === choice || o.label === choice);
    if (opt?.label) return opt.label;
    const text = choice.toLowerCase();
    if (text.includes("optimist")) {
      return req.options.find((o) => o.value === "optimist_option")?.label ?? choice;
    }
    if (text.includes("pessimist")) {
      return req.options.find((o) => o.value === "pessimist_option")?.label ?? choice;
    }
    return choice;
  }

  function voteBreakdownLines(req: ElicitationRequest): string[] {
    const notes = req.vote_summary.voter_notes ?? [];
    if (notes.length === 0) return ["• No per-voter notes were recorded."];
    return notes.map((n) => `• ${n.voter_id} -> ${truncField(labelForChoice(req, n.chose), 90)}`);
  }

  function buildEmbed(req: ElicitationRequest, resolved?: string): EmbedBuilder {
    const { consensus_strength, total_voters, escalated } = req.vote_summary;
    const consensusPct = Math.round(consensus_strength * 100);

    const embed = new EmbedBuilder()
      .setTimestamp(new Date(req.timestamp));

    if (resolved) {
      // ── Resolved state: show the decision prominently ──
      embed
        .setTitle(truncField(req.question, 256))
        .setColor(0x57f287);

      embed.addFields({
        name: "Decision",
        value: `**${resolved}**`,
      });

      embed.addFields({
        name: "Consensus",
        value: `${consensusPct}% agreement (${total_voters} voters)`,
        inline: true,
      });

      embed.addFields({
        name: "Category",
        value: req.category,
        inline: true,
      });

      embed.addFields({
        name: "Committee votes (debug)",
        value: truncField(voteBreakdownLines(req).join("\n"), 1024),
      });

      embed.setFooter({ text: `Session: ${req.session_id}` });
    } else {
      // ── Decision Required: show options with pros/cons at a glance ──
      embed
        .setTitle(truncField(`Decision Required: ${req.question}`, 256))
        .setColor(escalated ? 0xed4245 : 0x5865f2);

      // Per-option summaries: show advocate, votes, and rationale
      for (const opt of req.options) {
        const count = opt.vote_count ?? req.vote_summary.tally[opt.value] ?? 0;
        const isWinner = opt.value === req.recommended_option;
        const advocate = opt.advocated_by ? ` (${opt.advocated_by})` : "";

        const lines: string[] = [];
        lines.push(`${isWinner ? "**Recommended** — " : ""}${count} vote${count !== 1 ? "s" : ""}${advocate}`);

        if (opt.description) {
          lines.push(truncField(opt.description, 300));
        }

        const supporters = req.vote_summary.voter_notes
          ?.filter((n) => n.chose === opt.value || n.chose === opt.label)
          .slice(0, 3)
          .map((n) => `> **${n.voter_id}**: ${truncField(n.reasoning, 150)}`);
        if (supporters?.length) {
          lines.push(...supporters);
        }

        const fieldName = `${isWinner ? "▶ " : ""}${truncField(opt.label, 250)}`;
        embed.addFields({
          name: fieldName,
          value: truncField(lines.join("\n")),
        });
      }

      // Consensus bar
      embed.addFields({
        name: "Consensus",
        value: `${consensusPct}% agreement (${total_voters} voters)${escalated ? " — **ESCALATED**" : ""}`,
        inline: true,
      });

      embed.addFields({
        name: "Category",
        value: req.category,
        inline: true,
      });

      embed.addFields({
        name: "Status",
        value: "Waiting for user input",
      });

      embed.addFields({
        name: "Committee votes (debug)",
        value: truncField(voteBreakdownLines(req).join("\n"), 1024),
      });

      embed.setFooter({ text: `Session: ${req.session_id} | Decision: ${req.decision_id}` });
    }

    return embed;
  }

  function buildDecisionOutcomeEmbed(
    req: ElicitationRequest,
    selected: { value: string; label: string },
    userId: string,
  ): EmbedBuilder {
    const consensusPct = Math.round((req.vote_summary.consensus_strength ?? 0) * 100);
    const selectedVotes = req.vote_summary.tally[selected.value] ?? 0;
    const reasons = (req.vote_summary.voter_notes ?? [])
      .filter((n) => n.chose === selected.value || n.chose === selected.label)
      .slice(0, 3)
      .map((n) => `• ${truncField(n.reasoning, 140)}`);
    const whyLines = reasons.length > 0
      ? reasons
      : [`• Highest support in committee tally (${selectedVotes} votes).`];
    const alternatives = req.options
      .filter((o) => o.value !== selected.value)
      .slice(0, 3)
      .map((o) => {
        const votes = req.vote_summary.tally[o.value] ?? 0;
        return `• ${truncField(o.label, 90)} — ${votes} vote${votes !== 1 ? "s" : ""}`;
      });

    return new EmbedBuilder()
      .setColor(0x57f287)
      .setTitle(truncField(`Decision Outcome: ${req.question}`, 256))
      .setDescription(`**Selected:** ${truncField(selected.label, 180)}\nSelected by <@${userId}>`)
      .addFields(
        {
          name: "Why this was chosen",
          value: truncField(whyLines.join("\n"), 1024),
        },
        {
          name: "Other options considered",
          value: truncField(
            alternatives.length > 0
              ? alternatives.join("\n")
              : "• No alternatives were provided.",
            1024,
          ),
        },
        {
          name: "Consensus",
          value: `${consensusPct}% agreement (${req.vote_summary.total_voters} voters)`,
          inline: true,
        },
        {
          name: "Category",
          value: req.category,
          inline: true,
        },
        {
          name: "Committee votes (debug)",
          value: truncField(voteBreakdownLines(req).join("\n"), 1024),
        },
      )
      .setFooter({ text: `Session: ${req.session_id} | Decision: ${req.decision_id}` })
      .setTimestamp(new Date());
  }

  async function postMainChannelOutcome(
    req: ElicitationRequest,
    selected: { value: string; label: string },
    userId: string,
  ): Promise<void> {
    const mainChannelId = req.metadata?.main_channel_id;
    if (!mainChannelId) return;
    const outcome = buildDecisionOutcomeEmbed(req, selected, userId);
    await discord.postEmbed(mainChannelId, outcome);
  }

  // ─── Build components ───────────────────────────────────────────────────

  function buildComponents(
    req: ElicitationRequest,
    interactionKey: string,
  ): ActionRowBuilder<MessageActionRowComponentBuilder>[] {
    const rows: ActionRowBuilder<MessageActionRowComponentBuilder>[] = [];

    if (req.options.length <= MAX_BUTTONS_PER_ROW) {
      const row = new ActionRowBuilder<MessageActionRowComponentBuilder>();
      for (let i = 0; i < req.options.length; i++) {
        const opt = req.options[i];
        const isRecommended = opt.value === req.recommended_option;
        row.addComponents(
          new ButtonBuilder()
            .setCustomId(`${SELECT_PREFIX}${interactionKey}:${i}`)
            .setLabel(truncField(opt.label, 80))
            .setStyle(isRecommended ? ButtonStyle.Success : ButtonStyle.Secondary),
        );
      }
      rows.push(row);
    } else {
      const menu = new StringSelectMenuBuilder()
        .setCustomId(safeCustomId(`${SELECT_PREFIX}${interactionKey}`))
        .setPlaceholder("Select an option...")
        .addOptions(
          req.options.map((opt, i) => ({
            label: truncField(opt.label, 100),
            value: String(i),
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
          .setCustomId(safeCustomId(`${REDELIBERATE_PREFIX}${interactionKey}`))
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
      stateDb.markElicitationResolved("discord", elicitation_id, undefined, undefined, undefined, "discord", "skipped-no-channel");
      return;
    }
    stateDb.setSessionStatus(request.session_id, "waiting_user");
    stateDb.appendProviderEvent("discord", "elicitation_received", request, request.session_id, elicitation_id, request.metadata?.run_id);

    const embed = buildEmbed(request);

    if (request.informational) {
      await discord.postEmbed(discord_channel_id, embed);
      logger.info(`Elicitation ${elicitation_id}: posted informational embed`);
      stateDb.appendProviderEvent("discord", "informational_posted", { channelId: discord_channel_id }, request.session_id, elicitation_id, request.metadata?.run_id);
      return;
    }

    // Interactive: post with buttons/select menu
    const interactionKey = interactionKeyFor(elicitation_id);
    const components = buildComponents(request, interactionKey);
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

        logger.info(`Elicitation ${elicitation_id}: timeout — auto-selecting "${request.recommended_option}"`);

        const response = createElicitationResponse(elicitation_id, "discord", "system:timeout", {
          selectedOption: request.recommended_option,
        });
        await resolveElicitation(elicitation_id, response);
        pendingByInteractionKey.delete(p.interactionKey);
        pending.delete(elicitation_id);
        resolved.add(elicitation_id);
        stateDb.markElicitationResolved(
          "discord",
          elicitation_id,
          request.recommended_option,
          request.options.find((o) => o.value === request.recommended_option)?.label,
          "system:timeout",
          "discord",
          "timeout",
        );
        stateDb.setSessionStatus(request.session_id, "decision_made");
        stateDb.appendProviderEvent("discord", "timeout_auto_select", { selectedOption: request.recommended_option }, request.session_id, elicitation_id, request.metadata?.run_id);
        void persistState();

        const resolvedEmbed = buildEmbed(request, `Timeout — auto-selected: ${request.recommended_option}`);
        await discord.updateMessage(p.channelId, p.messageId, resolvedEmbed);
      }, request.timeout_seconds * 1000 + graceMs);
    }

    pending.set(elicitation_id, {
      request,
      channelId: discord_channel_id,
      messageId: message.id,
      interactionKey,
      timeoutTimer,
    });
    pendingByInteractionKey.set(interactionKey, elicitation_id);
    resolved.delete(elicitation_id);
    stateDb.upsertProviderMessageRef("discord", elicitation_id, {
      channelId: discord_channel_id,
      messageId: message.id,
      interactionKey,
    });
    stateDb.appendProviderEvent("discord", "interactive_card_posted", { channelId: discord_channel_id, messageId: message.id }, request.session_id, elicitation_id, request.metadata?.run_id);
    void persistState();

    logger.info(`Elicitation ${elicitation_id}: posted interactive message ${message.id}`);
  }

  // ─── Discord → HTTP ────────────────────────────────────────────────────

  async function handleInteraction(interaction: Interaction): Promise<void> {
    // ACK component clicks immediately so Discord never treats a slow path as
    // an expired interaction while we are still resolving the decision.
    if (interaction.isButton() || interaction.isStringSelectMenu()) {
      try {
        await acknowledgeComponentInteraction(interaction);
      } catch (err) {
        logger.warn(`Initial interaction ack failed: ${err}`);
        // Best effort: if this fails, stale-card handling below still tries to
        // update the message directly.
      }
    }

    // Handle button clicks
    if (interaction.isButton()) {
      const customId = interaction.customId;

      // Re-deliberation button
      if (customId.startsWith(REDELIBERATE_PREFIX)) {
        const interactionKey = customId.slice(REDELIBERATE_PREFIX.length);
        const elicitationId = pendingByInteractionKey.get(interactionKey);
        if (!elicitationId) {
          await expireInteractionCard(interaction, "This decision is no longer active. Please use the latest decision card.");
          return;
        }
        const entry = pending.get(elicitationId);
        if (!entry) {
          await expireInteractionCard(interaction, "This decision has already been resolved. Please use the latest decision card.");
          return;
        }

        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        pendingByInteractionKey.delete(entry.interactionKey);
        pending.delete(elicitationId);
        resolved.add(elicitationId);
        stateDb.markElicitationResolved("discord", elicitationId, undefined, undefined, interaction.user.id, "discord", "redeliberate");
        stateDb.setSessionStatus(entry.request.session_id, "decision_made");
        stateDb.appendProviderEvent("discord", "redeliberate_selected", { userId: interaction.user.id }, entry.request.session_id, elicitationId, entry.request.metadata?.run_id);
        void persistState();

        const resolvedEmbed = buildEmbed(entry.request, `Re-deliberation requested by <@${interaction.user.id}>`);
        await finalizeInteractionUpdate(interaction, entry.channelId, entry.messageId, resolvedEmbed);

        const response = createElicitationResponse(elicitationId, "discord", interaction.user.id, {
          userContext: "Human requested re-deliberation via Discord",
        });
        await resolveElicitation(elicitationId, response);

        logger.info(`Elicitation ${elicitationId}: re-deliberation requested by ${interaction.user.tag}`);
        return;
      }

      // Selection button
      if (customId.startsWith(SELECT_PREFIX)) {
        const parts = customId.slice(SELECT_PREFIX.length);
        const colonIdx = parts.indexOf(":");
        if (colonIdx === -1) {
          await expireInteractionCard(interaction, "This decision card is malformed or expired. Please use the latest decision card.");
          return;
        }

        const interactionKey = parts.slice(0, colonIdx);
        const optIdx = parseInt(parts.slice(colonIdx + 1), 10);
        const elicitationId = pendingByInteractionKey.get(interactionKey);
        if (!elicitationId) {
          await expireInteractionCard(interaction, "This decision is no longer active. Please use the latest decision card.");
          return;
        }
        const entry = pending.get(elicitationId);
        if (!entry) {
          await expireInteractionCard(interaction, "This decision has already been resolved. Please use the latest decision card.");
          return;
        }

        const opt = entry.request.options[optIdx];
        if (!opt) {
          await expireInteractionCard(interaction, "This option is no longer valid. Please use the latest decision card.");
          return;
        }

        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        pendingByInteractionKey.delete(entry.interactionKey);
        pending.delete(elicitationId);
        resolved.add(elicitationId);
        stateDb.markElicitationResolved("discord", elicitationId, opt.value, opt.label, interaction.user.id, "discord", "selected");
        stateDb.setSessionStatus(entry.request.session_id, "decision_made");
        stateDb.appendProviderEvent("discord", "option_selected", { userId: interaction.user.id, selectedOption: opt.value }, entry.request.session_id, elicitationId, entry.request.metadata?.run_id);
        void persistState();

        const resolvedEmbed = buildEmbed(entry.request, `${truncField(opt.label, 120)} — selected by <@${interaction.user.id}>`);
        await finalizeInteractionUpdate(interaction, entry.channelId, entry.messageId, resolvedEmbed);

        const response = createElicitationResponse(elicitationId, "discord", interaction.user.id, {
          selectedOption: opt.value,
        });
        await resolveElicitation(elicitationId, response);
        await postMainChannelOutcome(entry.request, { value: opt.value, label: opt.label }, interaction.user.id);

        logger.info(`Elicitation ${elicitationId}: "${opt.value.slice(0, 80)}" selected by ${interaction.user.tag}`);
        return;
      }
    }

    // Handle select menu
    if (interaction.isStringSelectMenu() && interaction.customId.startsWith(SELECT_PREFIX)) {
      const interactionKey = interaction.customId.slice(SELECT_PREFIX.length);
      const selectedValue = interaction.values[0];
      const elicitationId = pendingByInteractionKey.get(interactionKey);
      if (!elicitationId) {
        await expireInteractionCard(interaction, "This decision is no longer active. Please use the latest decision card.");
        return;
      }
      const entry = pending.get(elicitationId);
      if (!entry) {
        await expireInteractionCard(interaction, "This decision has already been resolved. Please use the latest decision card.");
        return;
      }

      if (!selectedValue) {
        await safeEphemeralReply(interaction, "No option was selected. Please try again.");
        return;
      }

      const optIdx = parseInt(selectedValue, 10);
      const opt = entry.request.options[optIdx];
      if (!opt) {
        await expireInteractionCard(interaction, "This option is no longer valid. Please use the latest decision card.");
        return;
      }

      if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
      pendingByInteractionKey.delete(entry.interactionKey);
      pending.delete(elicitationId);
      resolved.add(elicitationId);
      stateDb.markElicitationResolved("discord", elicitationId, opt.value, opt.label, interaction.user.id, "discord", "selected");
      stateDb.setSessionStatus(entry.request.session_id, "decision_made");
      stateDb.appendProviderEvent("discord", "option_selected", { userId: interaction.user.id, selectedOption: opt.value }, entry.request.session_id, elicitationId, entry.request.metadata?.run_id);
      void persistState();

      const resolvedEmbed = buildEmbed(entry.request, `${truncField(opt.label, 120)} — selected by <@${interaction.user.id}>`);
      await finalizeInteractionUpdate(interaction, entry.channelId, entry.messageId, resolvedEmbed);

      const response = createElicitationResponse(elicitationId, "discord", interaction.user.id, {
        selectedOption: opt.value,
      });
      await resolveElicitation(elicitationId, response);
      await postMainChannelOutcome(entry.request, { value: opt.value, label: opt.label }, interaction.user.id);

      logger.info(`Elicitation ${elicitationId}: "${opt.value.slice(0, 80)}" selected by ${interaction.user.tag}`);
      return;
    }

    // Design review buttons
    if (interaction.isButton()) {
      const cid = interaction.customId;

      if (cid.startsWith(DESIGN_SELECT_PREFIX)) {
        const parts = cid.slice(DESIGN_SELECT_PREFIX.length);
        const colonIdx = parts.indexOf(":");
        if (colonIdx === -1) {
          await expireInteractionCard(interaction, "This design review card is expired.");
          return;
        }
        const ik = parts.slice(0, colonIdx);
        const optIdx = parseInt(parts.slice(colonIdx + 1), 10);
        const eid = pendingByInteractionKey.get(ik);
        if (!eid) {
          await expireInteractionCard(interaction, "This design review is no longer active.");
          return;
        }
        const entry = pending.get(eid);
        if (!entry) {
          await expireInteractionCard(interaction, "This design review has been resolved.");
          return;
        }
        const opt = entry.request.options[optIdx];
        if (!opt) {
          await expireInteractionCard(interaction, "This variant is no longer valid.");
          return;
        }

        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        pendingByInteractionKey.delete(entry.interactionKey);
        pending.delete(eid);
        resolved.add(eid);
        stateDb.markElicitationResolved("discord", eid, opt.value, opt.label, interaction.user.id, "discord", "selected");
        void persistState();

        const resolvedEmbed = new EmbedBuilder()
          .setTitle("Design Review Complete")
          .setDescription(`Selected: **${opt.label}** — by <@${interaction.user.id}>`)
          .setColor(0x57f287)
          .setTimestamp(new Date());
        await finalizeInteractionUpdate(interaction, entry.channelId, entry.messageId, resolvedEmbed);

        const response = createElicitationResponse(eid, "discord", interaction.user.id, {
          selectedOption: opt.value,
        });
        await resolveElicitation(eid, response);
        logger.info(`Design review ${eid}: variant "${opt.value}" selected by ${interaction.user.tag}`);
        return;
      }

      if (cid.startsWith(DESIGN_CHANGES_PREFIX)) {
        const ik = cid.slice(DESIGN_CHANGES_PREFIX.length);
        const eid = pendingByInteractionKey.get(ik);
        if (!eid) {
          await expireInteractionCard(interaction, "This design review is no longer active.");
          return;
        }
        const entry = pending.get(eid);
        if (!entry) {
          await expireInteractionCard(interaction, "This design review has been resolved.");
          return;
        }

        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        pendingByInteractionKey.delete(entry.interactionKey);
        pending.delete(eid);
        resolved.add(eid);
        stateDb.markElicitationResolved("discord", eid, undefined, undefined, interaction.user.id, "discord", "request_changes");
        void persistState();

        const resolvedEmbed = new EmbedBuilder()
          .setTitle("Design Review — Changes Requested")
          .setDescription(`Changes requested by <@${interaction.user.id}>. Reply in this thread with notes.`)
          .setColor(0xfee75c)
          .setTimestamp(new Date());
        await finalizeInteractionUpdate(interaction, entry.channelId, entry.messageId, resolvedEmbed);

        const response = createElicitationResponse(eid, "discord", interaction.user.id, {
          userContext: "Human requested design changes via Discord",
        });
        await resolveElicitation(eid, response);
        logger.info(`Design review ${eid}: changes requested by ${interaction.user.tag}`);
        return;
      }
    }

    if (interaction.isButton() || interaction.isStringSelectMenu()) {
      await expireInteractionCard(interaction, "This component is no longer active. Please use the latest decision card.");
    }
  }

  // ─── Design Review (Phase B) ────────────────────────────────────────────

  const DESIGN_SELECT_PREFIX = "design_select:";
  const DESIGN_CHANGES_PREFIX = "design_changes:";

  async function handleDesignReview(request: DesignReviewRequest): Promise<void> {
    const channelId = request.discord_channel_id;
    if (!channelId) {
      logger.warn(`Design review ${request.review_id}: no discord_channel_id — skipping`);
      return;
    }

    const interactionKey = interactionKeyFor(request.review_id);

    const embeds = request.variants.map((v, i) =>
      new EmbedBuilder()
        .setTitle(`${i + 1}. ${v.label}`)
        .setDescription(v.description)
        .setImage(v.image_url)
        .setColor([0x5865f2, 0x57f287, 0xfee75c, 0xed4245, 0xeb459e][i % 5])
        .setFooter({ text: `Variant: ${v.variant_id} | ${v.aspects_changed.join(', ') || 'mixed'}` }),
    );

    const headerEmbed = new EmbedBuilder()
      .setTitle(`Design Review: ${request.screen_context}`)
      .setDescription(
        `Select your preferred design direction for **${request.screen_context}**.\n` +
        `${request.variants.length} variants generated. ` +
        (request.timeout_seconds > 0
          ? `Auto-selects ${request.recommended_variant ? 'recommended' : 'first'} variant in ${request.timeout_seconds}s.`
          : 'Waiting for your selection.'),
      )
      .setColor(0x5865f2)
      .setTimestamp(new Date(request.timestamp));

    const buttonRow = new ActionRowBuilder<MessageActionRowComponentBuilder>();
    for (let i = 0; i < request.variants.length && i < MAX_BUTTONS_PER_ROW - 1; i++) {
      const v = request.variants[i];
      const isRecommended = v.variant_id === request.recommended_variant;
      buttonRow.addComponents(
        new ButtonBuilder()
          .setCustomId(`${DESIGN_SELECT_PREFIX}${interactionKey}:${i}`)
          .setLabel(`${i + 1}. ${v.label}`.slice(0, 80))
          .setStyle(isRecommended ? ButtonStyle.Success : ButtonStyle.Primary),
      );
    }
    buttonRow.addComponents(
      new ButtonBuilder()
        .setCustomId(`${DESIGN_CHANGES_PREFIX}${interactionKey}`)
        .setLabel("Request Changes")
        .setStyle(ButtonStyle.Secondary),
    );

    const message = await discord.postElicitation(channelId, headerEmbed, [buttonRow], [headerEmbed, ...embeds]);

    let timeoutTimer: ReturnType<typeof setTimeout> | undefined;
    if (request.timeout_seconds > 0) {
      const autoVariant = request.recommended_variant ?? request.variants[0]?.variant_id;
      timeoutTimer = setTimeout(async () => {
        const p = pending.get(request.review_id);
        if (!p) return;

        logger.info(`Design review ${request.review_id}: timeout — auto-selecting "${autoVariant}"`);

        const response = createElicitationResponse(request.review_id, "discord", "system:timeout", {
          selectedOption: autoVariant,
        });
        await resolveElicitation(request.review_id, response);
        pendingByInteractionKey.delete(p.interactionKey);
        pending.delete(request.review_id);
        resolved.add(request.review_id);
        stateDb.markElicitationResolved("discord", request.review_id, autoVariant, undefined, "system:timeout", "discord", "timeout");
        void persistState();

        const resolvedEmbed = new EmbedBuilder()
          .setTitle(`Design Review: ${request.screen_context}`)
          .setDescription(`Timeout — auto-selected: **${autoVariant}**`)
          .setColor(0x57f287)
          .setTimestamp(new Date());
        await discord.updateMessage(p.channelId, p.messageId, resolvedEmbed);
      }, request.timeout_seconds * 1000 + 5000);
    }

    const elicitRequest: ElicitationRequest = {
      elicitation_id: request.review_id,
      session_id: request.session_id,
      decision_id: `design-${request.screen_context}`,
      question: `Select design direction for ${request.screen_context}`,
      category: 'design-review',
      options: request.variants.map(v => ({
        value: v.variant_id,
        label: v.label,
        description: v.description,
      })),
      recommended_option: request.recommended_variant,
      vote_summary: { total_voters: 0, tally: {}, consensus_strength: 0, escalated: false },
      allow_redeliberation: false,
      timeout_seconds: request.timeout_seconds,
      informational: false,
      timestamp: request.timestamp,
      discord_channel_id: channelId,
      metadata: request.metadata,
    };

    pending.set(request.review_id, {
      request: elicitRequest,
      channelId,
      messageId: message.id,
      interactionKey,
      timeoutTimer,
    });
    pendingByInteractionKey.set(interactionKey, request.review_id);
    resolved.delete(request.review_id);
    stateDb.upsertProviderMessageRef("discord", request.review_id, {
      channelId,
      messageId: message.id,
      interactionKey,
    });
    void persistState();

    logger.info(`Design review ${request.review_id}: posted ${request.variants.length} variant embeds with buttons`);
  }

  // ─── Cancel from Linear ─────────────────────────────────────────────────

  async function handleCancel(cancel: ElicitationCancel): Promise<void> {
    const entry = pending.get(cancel.elicitation_id);
    if (!entry) return;

    if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
    pendingByInteractionKey.delete(entry.interactionKey);
    pending.delete(cancel.elicitation_id);
    resolved.add(cancel.elicitation_id);
    stateDb.markElicitationResolved("discord", cancel.elicitation_id, cancel.selected_option, undefined, "linear-user", "linear", "cancelled");
    stateDb.setSessionStatus(entry.request.session_id, "decision_made");
    stateDb.appendProviderEvent("discord", "cancelled_by_linear", cancel, entry.request.session_id, cancel.elicitation_id, entry.request.metadata?.run_id);
    void persistState();

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
    pendingByInteractionKey.clear();
    void persistState();
    stateDb.close();
  }

  function getStatus(elicitationId: string): { active: boolean; known: boolean } {
    if (pending.has(elicitationId)) {
      return { active: true, known: true };
    }
    const persisted = stateDb.getElicitationStatus("discord", elicitationId);
    if (persisted.known) {
      return persisted;
    }
    if (resolved.has(elicitationId)) {
      return { active: false, known: true };
    }
    return persisted;
  }

  function getDecisionHistory(sessionId?: string, limit = 100): unknown[] {
    return stateDb.listDecisions(sessionId, limit);
  }

  function getSessionHistory(limit = 200, status?: string): unknown[] {
    if (status === undefined) return stateDb.listSessions(limit);
    if (status === "created" || status === "waiting_user" || status === "decision_made" || status === "failed" || status === "completed") {
      return stateDb.listSessions(limit, status);
    }
    return stateDb.listSessions(limit);
  }

  function getWaitingSessions(limit = 200): unknown[] {
    return stateDb.listWaitingSessions(limit);
  }

  function getDecisionAudit(elicitationId: string, bridge?: string): unknown {
    return stateDb.getDecisionAudit(elicitationId, bridge);
  }

  function getDesignHistory(sessionId?: string, limit = 50): unknown[] {
    return stateDb.listDesignSnapshots(sessionId, limit);
  }

  function recordDesignSnapshot(snapshot: Record<string, unknown>): void {
    const sessionId = typeof snapshot["session_id"] === "string" && snapshot["session_id"]
      ? snapshot["session_id"]
      : "design-unscoped";
    const runId = typeof snapshot["run_id"] === "string" ? snapshot["run_id"] : undefined;
    const projectName = typeof snapshot["project_name"] === "string" ? snapshot["project_name"] : undefined;
    const designMode = typeof snapshot["design_mode"] === "string" ? snapshot["design_mode"] : undefined;
    const stitchRequired = snapshot["stitch_required"] === true || String(snapshot["stitch_required"] ?? "") === "true";
    const stitchStatus = typeof snapshot["stitch_status"] === "string" ? snapshot["stitch_status"] : undefined;
    const hasFrontend = snapshot["has_frontend"] === true || String(snapshot["has_frontend"] ?? "") === "true";
    const artifactBundlePath = typeof snapshot["artifact_bundle_path"] === "string" ? snapshot["artifact_bundle_path"] : undefined;
    const context = (snapshot["context"] && typeof snapshot["context"] === "object")
      ? (snapshot["context"] as Record<string, unknown>)
      : {};
    stateDb.saveDesignSnapshot({
      sessionId,
      runId,
      projectName,
      designMode,
      stitchRequired,
      stitchStatus,
      hasFrontend,
      artifactBundlePath,
      context,
    });
  }

  return {
    handleRequest,
    handleDesignReview,
    handleInteraction,
    handleCancel,
    getStatus,
    getDecisionHistory,
    getSessionHistory,
    getWaitingSessions,
    getDecisionAudit,
    getDesignHistory,
    recordDesignSnapshot,
    destroy,
  };
}
