/**
 * Elicitation Handler — Linear Bridge
 *
 * Bidirectional human-in-the-loop decision flow:
 *   HTTP POST /elicitation     → Linear Agent API (select signal)
 *   Linear webhook (prompted)  → Lobster resume + Discord cross-cancel via HTTP
 *   HTTP POST /elicitation/cancel → Update Linear with resolution message
 */

import type { LinearClient, AgentActivityCreateInput } from './linear-client.js';
import type { AgentSessionManager } from './agent-session-manager.js';
import type { RunRegistry } from './run-registry.js';
import type { AgentSessionWebhookEvent } from './http-server.js';
import type { BridgeStateDb } from './state/bridge-state-db.js';
import type {
  ElicitationRequest,
  ElicitationCancel,
  DesignReviewRequest,
} from './elicitation-types.js';
import {
  createElicitationResponse,
  createElicitationCancel,
} from './elicitation-types.js';

// =============================================================================
// Types
// =============================================================================

interface PendingElicitation {
  request: ElicitationRequest;
  linearSessionId: string;
  timeoutTimer?: ReturnType<typeof setTimeout>;
  pollTimer?: ReturnType<typeof setInterval>;
  /** ISO timestamp: only activities after this point are considered user responses */
  elicitationPostedAt?: string;
}

// =============================================================================
// Handler
// =============================================================================

export interface ElicitationHandler {
  /** Handle an inbound elicitation request (via HTTP POST) */
  handleRequest(request: ElicitationRequest): Promise<void>;
  /** Handle a design review request (Phase B: variant image selection) */
  handleDesignReview(request: DesignReviewRequest): Promise<void>;
  /** Handle a Linear webhook event (prompted = human responded to select) */
  handleWebhookEvent(event: AgentSessionWebhookEvent): Promise<void>;
  /** Handle a cancel from the other bridge (Discord answered first) */
  handleCancel(cancel: ElicitationCancel): Promise<void>;
  /** Handle a run callback (Discord or external → Lobster resume) */
  handleRunCallback(runId: string, data: Record<string, unknown>): Promise<void>;
  /** Query whether an elicitation is still active */
  getStatus(elicitationId: string): { active: boolean; known: boolean };
  /** Query decision history for debugging/audit */
  getDecisionHistory(sessionId?: string, limit?: number): unknown[];
  /** Query session timeline */
  getSessionHistory(limit?: number, status?: string): unknown[];
  /** Query unresolved waiting sessions */
  getWaitingSessions(limit?: number): unknown[];
  /** Query full decision audit bundle */
  getDecisionAudit(elicitationId: string, bridge?: string): unknown;
  /** Query persisted design snapshots */
  getDesignHistory(sessionId?: string, limit?: number): unknown[];
  /** Persist design snapshot metadata */
  recordDesignSnapshot(snapshot: Record<string, unknown>): void;
  /** Clean up resources */
  destroy(): void;
}

export function createElicitationHandler(
  linear: LinearClient,
  sessionManager: AgentSessionManager,
  runRegistry: RunRegistry,
  stateDb: BridgeStateDb,
  discordBridgeUrl: string,
  logger: { info: Function; warn: Function; error: Function },
): ElicitationHandler {
  const pending = new Map<string, PendingElicitation>();
  const resolved = new Set<string>();

  for (const row of stateDb.getActiveElicitations("linear")) {
    pending.set(row.elicitationId, {
      request: row.request,
      linearSessionId: row.linearSessionId ?? "",
    });
  }

  // ─── HTTP helpers ─────────────────────────────────────────────────────

  async function postToDiscordBridge(path: string, data: unknown): Promise<void> {
    try {
      await fetch(`${discordBridgeUrl}${path}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });
    } catch (err) {
      logger.warn(`Failed to POST to discord-bridge ${path}: ${err}`);
    }
  }

  // ─── Build vote breakdown markdown ──────────────────────────────────────

  function buildVoteBody(req: ElicitationRequest): string {
    const lines: string[] = [];
    lines.push(`## Decision Required: ${req.question}`);
    lines.push('');
    lines.push(`**Category**: ${req.category}`);
    lines.push(`**Session**: \`${req.session_id}\` | **Decision**: \`${req.decision_id}\``);
    lines.push('');

    // Vote breakdown table
    lines.push('| Option | Votes | Advocated By |');
    lines.push('|--------|-------|-------------|');
    for (const opt of req.options) {
      const isWinner = opt.value === req.recommended_option;
      const marker = isWinner ? ' **recommended**' : '';
      lines.push(
        `| ${opt.label}${marker} | ${opt.vote_count ?? req.vote_summary.tally[opt.value] ?? 0} | ${opt.advocated_by ?? '—'} |`,
      );
    }
    lines.push('');

    // Consensus info
    const { consensus_strength, total_voters, escalated } = req.vote_summary;
    lines.push(
      `**Consensus**: ${Math.round(consensus_strength * 100)}% (${total_voters} voters)${escalated ? ' — **ESCALATED (tie)**' : ''}`,
    );
    lines.push('');
    lines.push('### Your input');
    lines.push('- Select one option from the UI below, or reply in free text.');
    lines.push('- Free-text replies are interpreted as clarification/re-deliberation context.');

    // Voter reasoning
    if (req.vote_summary.voter_notes?.length) {
      lines.push('');
      lines.push('### Voter Reasoning');
      for (const note of req.vote_summary.voter_notes) {
        lines.push(`- **${note.voter_id}** chose *${note.chose}*: ${note.reasoning}`);
      }
    }

    return lines.join('\n');
  }

  function normalizeText(value: string | undefined): string {
    return (value ?? '')
      .trim()
      .replace(/\*\*/g, '')
      .replace(/\s+/g, ' ');
  }

  function optionValueFromText(input: string, req: ElicitationRequest): string | undefined {
    const text = normalizeText(input).toLowerCase();
    if (!text) return undefined;
    for (const opt of req.options) {
      if (normalizeText(opt.value).toLowerCase() === text) return opt.value;
      if (normalizeText(opt.label).toLowerCase() === text) return opt.value;
    }
    return undefined;
  }

  // ─── HTTP → Linear ──────────────────────────────────────────────────────

  async function handleRequest(request: ElicitationRequest): Promise<void> {
    const { elicitation_id, session_id, linear_issue_id } = request;

    if (!linear_issue_id) {
      logger.warn(`Elicitation ${elicitation_id}: no linear_issue_id — skipping Linear rendering`);
      resolved.add(elicitation_id);
      stateDb.markElicitationResolved("linear", elicitation_id, undefined, undefined, undefined, "linear", "skipped-no-issue");
      return;
    }

    stateDb.setSessionStatus(session_id, "waiting_user");
    stateDb.saveElicitationPending({
      bridge: "linear",
      elicitationId: elicitation_id,
      request,
      status: "active",
    });
    stateDb.appendProviderEvent("linear", "elicitation_received", request, session_id, elicitation_id, request.metadata?.run_id);

    // Store resume token in run registry if present
    if (request.resume_token && request.metadata?.run_id) {
      runRegistry.update(request.metadata.run_id, { resumeToken: request.resume_token });
    }

    // Look up or reuse agent session
    let session = sessionManager.findByDeliberation(session_id);
    if (!session) {
      logger.info(`Elicitation ${elicitation_id}: no active agent session for ${session_id} — creating activity without session`);
    }

    const body = buildVoteBody(request);

    if (request.informational) {
      await linear.createComment(linear_issue_id, body);
      logger.info(`Elicitation ${elicitation_id}: posted informational comment to ${linear_issue_id}`);
      return;
    }

    // Interactive: post as agent activity with select signal
    if (session) {
      await linear.createAgentActivity({
        agentSessionId: session.linearSessionId,
        content: { type: 'thought', body: `Presenting committee vote on ${request.decision_id}...` },
        ephemeral: true,
      });

      const selectOptions = request.options.map((opt) => ({
        value: opt.value,
        label: opt.label,
      }));

      if (request.allow_redeliberation) {
        selectOptions.push({
          value: '__redeliberate__',
          label: 'Request re-deliberation',
        });
      }

      const input: AgentActivityCreateInput = {
        agentSessionId: session.linearSessionId,
        content: { type: 'elicitation', body },
        signal: 'select',
        signalMetadata: { options: selectOptions },
      };

      await linear.createAgentActivity(input);
      logger.info(`Elicitation ${elicitation_id}: posted select signal to Linear session ${session.linearSessionId}`);
    } else {
      const commentBody = body + '\n\n_Note: No active agent session — respond via Discord or restart session._';
      await linear.createComment(linear_issue_id, commentBody);
      logger.info(`Elicitation ${elicitation_id}: posted fallback comment (no agent session)`);
    }

    // Set up timeout
    let timeoutTimer: ReturnType<typeof setTimeout> | undefined;
    if (request.timeout_seconds > 0 && request.recommended_option) {
      timeoutTimer = setTimeout(async () => {
        const p = pending.get(elicitation_id);
        if (!p) return;
        if (p.pollTimer) clearInterval(p.pollTimer);
        pending.delete(elicitation_id);
        resolved.add(elicitation_id);
        stateDb.markElicitationResolved("linear", elicitation_id, request.recommended_option, undefined, "system:timeout", "linear", "timeout");
        stateDb.setSessionStatus(request.session_id, "decision_made");
        stateDb.appendProviderEvent("linear", "timeout_auto_select", { selectedOption: request.recommended_option }, request.session_id, elicitation_id, request.metadata?.run_id);

        logger.info(`Elicitation ${elicitation_id}: timeout — auto-selecting "${request.recommended_option}"`);

        // Cross-cancel to Discord
        const cancel = createElicitationCancel(elicitation_id, 'linear', request.recommended_option);
        await postToDiscordBridge('/elicitation/cancel', cancel);

        // Post timeout notification
        if (session) {
          await linear.createAgentActivity({
            agentSessionId: session.linearSessionId,
            content: {
              type: 'response',
              body: `Timeout — auto-selected: **${request.recommended_option}**`,
            },
          }).catch((err) => logger.error(`Failed to post timeout activity: ${err}`));
        }
      }, request.timeout_seconds * 1000);
    }

    const elicitationPostedAt = new Date().toISOString();

    // Start polling for user response (webhook fallback — Linear agent session
    // webhooks don't fire reliably for API-created sessions)
    let pollTimer: ReturnType<typeof setInterval> | undefined;
    if (session) {
      const POLL_INTERVAL_MS = 3000;
      pollTimer = setInterval(async () => {
        if (!pending.has(elicitation_id)) {
          clearInterval(pollTimer!);
          return;
        }
        try {
          const activities = await linear.getSessionActivities(
            session!.linearSessionId,
            elicitationPostedAt,
          );
          const promptActivity = activities.find(a => a.contentType === 'prompt');
          if (promptActivity) {
            clearInterval(pollTimer!);
            logger.info(`Elicitation ${elicitation_id}: detected user response via polling: "${promptActivity.body}"`);
            // Synthesize a webhook-like event and handle it
            const syntheticEvent: AgentSessionWebhookEvent = {
              action: 'prompted',
              type: 'AgentSession',
              createdAt: promptActivity.createdAt,
              data: {
                id: session!.linearSessionId,
                promptedValue: promptActivity.body ?? '',
                agentActivity: { body: promptActivity.body },
              },
            };
            await handleWebhookEvent(syntheticEvent);
          }
        } catch (err) {
          logger.warn(`Poll error for elicitation ${elicitation_id}: ${err}`);
        }
      }, POLL_INTERVAL_MS);
    }

    pending.set(elicitation_id, {
      request,
      linearSessionId: session?.linearSessionId ?? '',
      timeoutTimer,
      pollTimer,
      elicitationPostedAt,
    });
    resolved.delete(elicitation_id);
    stateDb.upsertProviderMessageRef("linear", elicitation_id, {
      threadId: session?.linearSessionId ?? undefined,
    });
  }

  // ─── Linear → Lobster resume + Discord cross-cancel ────────────────────

  async function handleWebhookEvent(event: AgentSessionWebhookEvent): Promise<void> {
    // Handle 'created' — register the agent session
    if (event.action === 'created') {
      const linearSessionId = event.data.id;
      const deliberationSessionId = event.data.deliberationSessionId as string | undefined;
      const issueId = event.data.issueId as string | undefined;

      if (deliberationSessionId && issueId) {
        sessionManager.register(deliberationSessionId, linearSessionId, issueId);
        logger.info(`Registered agent session ${linearSessionId} for deliberation ${deliberationSessionId}`);
        stateDb.setSessionStatus(deliberationSessionId, "created");
        stateDb.appendProviderEvent("linear", "session_created", event, deliberationSessionId);

        // Best practice: set delegate and move to started status
        linear.setDelegateIfNone(issueId).catch(err =>
          logger.warn(`Failed to set delegate on ${issueId}: ${err}`));
        linear.moveToStartedStatus(issueId).catch(err =>
          logger.warn(`Failed to move ${issueId} to started: ${err}`));
      } else {
        logger.info(`Agent session created: ${linearSessionId} (no deliberation mapping)`);
        stateDb.appendProviderEvent("linear", "session_created_unmapped", event);
      }

      // 10-second acknowledgment: emit ephemeral thought
      await linear.createAgentActivity({
        agentSessionId: linearSessionId,
        content: { type: 'thought', body: 'Initializing pipeline...' },
        ephemeral: true,
      }).catch((err) => logger.error(`Failed to post ack activity: ${err}`));
      return;
    }

    // Handle 'prompted' with stop signal
    if (event.action === 'prompted' && event.data.promptedValue === '__stop__') {
      const linearSessionId = event.data.id;
      logger.info(`Stop signal received for session ${linearSessionId}`);
      stateDb.appendProviderEvent("linear", "stop_signal", event);
      // TODO: Forward stop to coordinator agent via /hooks/wake
      await linear.createAgentActivity({
        agentSessionId: linearSessionId,
        content: { type: 'response', body: 'Pipeline stop requested. Disengaging...' },
      }).catch((err) => logger.error(`Failed to post stop activity: ${err}`));
      return;
    }

    if (event.action !== 'prompted') return;

    const linearSessionId = event.data.id;

    // Find matching pending elicitation
    let matchedId: string | undefined;
    let matchedEntry: PendingElicitation | undefined;

    for (const [eid, entry] of pending) {
      if (entry.linearSessionId === linearSessionId) {
        matchedId = eid;
        matchedEntry = entry;
        break;
      }
    }

    if (!matchedId || !matchedEntry) {
      logger.warn(`No pending elicitation for Linear session ${linearSessionId}`);
      stateDb.appendProviderEvent("linear", "prompted_without_pending", event);
      return;
    }

    const promptBody = normalizeText(
      (event.data.agentActivity?.body as string | undefined)
      ?? (event.data.body as string | undefined),
    );
    const promptedValue = normalizeText(event.data.promptedValue);
    const rawInput = promptedValue || promptBody;
    if (!rawInput) {
      logger.warn(`Prompted event missing selection/body for session ${linearSessionId}`);
      return;
    }

    const selectedByOption = optionValueFromText(rawInput, matchedEntry.request);
    const looksRedeliberate = /re[-\s]?deliberat|debate again|reconsider|more context/i.test(rawInput);
    const isExplicitRedeliberate = rawInput === '__redeliberate__';
    const isRedeliberate = isExplicitRedeliberate || (!selectedByOption && looksRedeliberate);
    const selectedValue = !isRedeliberate ? selectedByOption : undefined;
    const userContext = !selectedValue ? rawInput : undefined;

    if (matchedEntry.timeoutTimer) clearTimeout(matchedEntry.timeoutTimer);
    if (matchedEntry.pollTimer) clearInterval(matchedEntry.pollTimer);
    pending.delete(matchedId);
    resolved.add(matchedId);
    stateDb.markElicitationResolved(
      "linear",
      matchedId,
      selectedValue,
      matchedEntry.request.options.find((o) => o.value === selectedValue)?.label,
      (event.data.agentActivity?.userId as string | undefined)
        ?? (event.data.userId as string | undefined)
        ?? "linear-user",
      "linear",
      isRedeliberate || !selectedValue ? "redeliberate" : "selected",
    );
    stateDb.setSessionStatus(matchedEntry.request.session_id, "decision_made");
    stateDb.appendProviderEvent("linear", "prompted_resolved", { rawInput, selectedValue, isRedeliberate }, matchedEntry.request.session_id, matchedId, matchedEntry.request.metadata?.run_id);

    // Write design selections to file for Lobster workflow consumption
    if (matchedEntry.request.category === 'design-review' || matchedEntry.request.decision_id?.startsWith('design-') || matchedId.startsWith('design-')) {
      try {
        const fs = await import('fs');
        const path = await import('path');
        const ws = process.env.WORKSPACE || process.cwd();
        const selectionsFile = path.join(ws, '.intake', 'design', 'stitch', 'design-selections.json');
        let existing: Array<Record<string, unknown>> = [];
        try { existing = JSON.parse(fs.readFileSync(selectionsFile, 'utf-8')); } catch { /* first selection */ }
        existing.push({
          decision_id: matchedEntry.request.decision_id ?? matchedId,
          selected_value: selectedValue ?? rawInput,
          is_redeliberate: isRedeliberate,
          resolved_at: new Date().toISOString(),
          source: 'linear',
        });
        fs.mkdirSync(path.dirname(selectionsFile), { recursive: true });
        fs.writeFileSync(selectionsFile, JSON.stringify(existing, null, 2));
        logger.info(`Design selection written to ${selectionsFile} (${existing.length} total)`);
      } catch (err) {
        logger.warn(`Failed to write design selections file: ${err}`);
      }
    }

    // Build response
    const response = createElicitationResponse(
      matchedId,
      'linear',
      (event.data.agentActivity?.userId as string | undefined)
        ?? (event.data.userId as string | undefined)
        ?? 'linear-user',
      isRedeliberate || !selectedValue
        ? { userContext: userContext ?? 'Human requested re-deliberation via Linear' }
        : { selectedOption: selectedValue },
    );

    // Cross-cancel to Discord
    const cancel = createElicitationCancel(
      matchedId,
      'linear',
      isRedeliberate || !selectedValue ? undefined : selectedValue,
    );
    await postToDiscordBridge('/elicitation/cancel', cancel);

    // If run has a resume token, trigger Lobster resume
    const runId = matchedEntry.request.metadata?.run_id;
    if (runId) {
      const run = runRegistry.lookup(runId);
      if (run?.resumeToken) {
        logger.info(`Elicitation ${matchedId}: triggering Lobster resume for run ${runId}`);
        // Lobster resume is handled by the coordinator — the response data is the resume payload
        // In practice, the Lobster runtime picks this up via its approval gate mechanism
      }
    }

    // Post confirmation activity
    if (matchedEntry.linearSessionId) {
      const confirmBody = isRedeliberate || !selectedValue
        ? `Human replied in free text: "${(userContext ?? '').slice(0, 240)}"`
        : `Human selected: **${selectedValue}**`;

      await linear.createAgentActivity({
        agentSessionId: matchedEntry.linearSessionId,
        content: { type: 'response', body: confirmBody },
      }).catch((err) => logger.error(`Failed to post confirmation activity: ${err}`));
    }

    logger.info(
      `Elicitation ${matchedId}: resolved via Linear — ${isRedeliberate || !selectedValue ? 'free-text/redeliberate' : selectedValue}`,
    );
  }

  // ─── Run callback (from Discord or external) ─────────────────────────

  async function handleRunCallback(runId: string, data: Record<string, unknown>): Promise<void> {
    const run = runRegistry.lookup(runId);
    if (!run) {
      logger.warn(`Run callback for unknown run ${runId}`);
      stateDb.appendProviderEvent("linear", "run_callback_unknown", data, undefined, undefined, runId);
      return;
    }

    // The callback data is an ElicitationResponse from Discord
    const elicitationId = data.elicitation_id as string | undefined;
    if (elicitationId) {
      const entry = pending.get(elicitationId);
      if (entry) {
        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        if (entry.pollTimer) clearInterval(entry.pollTimer);
        pending.delete(elicitationId);
        resolved.add(elicitationId);
        stateDb.markElicitationResolved(
          "linear",
          elicitationId,
          (data.selected_option as string | undefined) ?? undefined,
          entry.request.options.find((o) => o.value === (data.selected_option as string | undefined))?.label,
          (data.responded_by as string | undefined) ?? "discord-user",
          "discord",
          (data.response_type as string | undefined) ?? "selected",
        );
        stateDb.setSessionStatus(entry.request.session_id, "decision_made");
        stateDb.appendProviderEvent("linear", "run_callback_resolved", data, entry.request.session_id, elicitationId, runId);

        // Post resolution to Linear
        if (entry.linearSessionId) {
          const selectedOption = data.selected_option as string | undefined;
          const responseType = data.response_type as string | undefined;
          const body = responseType === 'redeliberate'
            ? 'Human requested re-deliberation via Discord'
            : `Human selected via Discord: **${selectedOption ?? 'unknown'}**`;

          await linear.createAgentActivity({
            agentSessionId: entry.linearSessionId,
            content: { type: 'response', body },
          }).catch((err) => logger.error(`Failed to post Discord resolution activity: ${err}`));
        }
      }
    }

    logger.info(`Run ${runId}: callback processed`);
  }

  // ─── Cancel from Discord ────────────────────────────────────────────────

  async function handleCancel(cancel: ElicitationCancel): Promise<void> {
    const entry = pending.get(cancel.elicitation_id);
    if (!entry) return;

    if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
    if (entry.pollTimer) clearInterval(entry.pollTimer);
    pending.delete(cancel.elicitation_id);
    resolved.add(cancel.elicitation_id);
    stateDb.markElicitationResolved("linear", cancel.elicitation_id, cancel.selected_option, undefined, "discord-user", "discord", "cancelled");
    stateDb.appendProviderEvent("linear", "cancelled_by_discord", cancel, entry.request.session_id, cancel.elicitation_id, entry.request.metadata?.run_id);

    const resolvedMsg = cancel.selected_option
      ? `Resolved via Discord: **${cancel.selected_option}**`
      : 'Resolved via Discord (re-deliberation requested)';

    if (entry.linearSessionId) {
      await linear.createAgentActivity({
        agentSessionId: entry.linearSessionId,
        content: { type: 'response', body: resolvedMsg },
      }).catch((err) => logger.error(`Failed to post cancel activity: ${err}`));
    } else if (entry.request.linear_issue_id) {
      await linear.createComment(entry.request.linear_issue_id, resolvedMsg)
        .catch((err) => logger.error(`Failed to post cancel comment: ${err}`));
    }

    logger.info(`Elicitation ${cancel.elicitation_id}: cancelled — ${resolvedMsg}`);
  }

  // ─── Cleanup ────────────────────────────────────────────────────────────

  function destroy(): void {
    for (const entry of pending.values()) {
      if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
      if (entry.pollTimer) clearInterval(entry.pollTimer);
    }
    pending.clear();
  }

  function getStatus(elicitationId: string): { active: boolean; known: boolean } {
    if (pending.has(elicitationId)) return { active: true, known: true };
    const persisted = stateDb.getElicitationStatus("linear", elicitationId);
    if (persisted.known) return persisted;
    if (resolved.has(elicitationId)) return { active: false, known: true };
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

  async function handleDesignReview(request: DesignReviewRequest): Promise<void> {
    const issueId = request.linear_issue_id ?? request.metadata?.linear_issue_id;
    if (!issueId) {
      logger.warn(`Design review ${request.review_id}: no linear_issue_id — skipping Linear rendering`);
      return;
    }

    const variantLines = request.variants.map((v, i) =>
      `### ${i + 1}. ${v.label}\n![${v.label}](${v.image_url})\n${v.description}\n_Aspects: ${v.aspects_changed.join(', ') || 'mixed'}_`,
    );

    const body = [
      `## Design Review: ${request.screen_context}`,
      '',
      `Select your preferred design direction for **${request.screen_context}**.`,
      '',
      ...variantLines,
      '',
      '---',
      `**Reply with the number** (1–${request.variants.length}) or reply "changes" with notes.`,
    ].join('\n');

    const session = sessionManager.findByDeliberation(request.session_id);

    if (session) {
      const selectOptions = request.variants.map((v, i) => ({
        value: v.variant_id,
        label: `${i + 1}. ${v.label}`,
      }));
      selectOptions.push({ value: 'request_changes', label: 'Request Changes' });

      const input: AgentActivityCreateInput = {
        agentSessionId: session.linearSessionId,
        content: { type: 'elicitation', body },
        signal: 'select',
        signalMetadata: { options: selectOptions },
      };

      await linear.createAgentActivity(input);
      logger.info(`Design review ${request.review_id}: posted select signal to Linear session ${session.linearSessionId}`);
    } else {
      await linear.createComment(issueId, body);
      logger.info(`Design review ${request.review_id}: posted comment to issue ${issueId}`);
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
      linear_issue_id: issueId,
      metadata: request.metadata,
    };

    pending.set(request.review_id, {
      request: elicitRequest,
      linearSessionId: session?.linearSessionId ?? '',
    });
    stateDb.saveElicitationPending({
      bridge: 'linear',
      elicitationId: request.review_id,
      request: elicitRequest,
      status: 'active',
    });
  }

  return {
    handleRequest,
    handleDesignReview,
    handleWebhookEvent,
    handleCancel,
    handleRunCallback,
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
