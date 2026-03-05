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
import type {
  ElicitationRequest,
  ElicitationCancel,
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
}

// =============================================================================
// Handler
// =============================================================================

export interface ElicitationHandler {
  /** Handle an inbound elicitation request (via HTTP POST) */
  handleRequest(request: ElicitationRequest): Promise<void>;
  /** Handle a Linear webhook event (prompted = human responded to select) */
  handleWebhookEvent(event: AgentSessionWebhookEvent): Promise<void>;
  /** Handle a cancel from the other bridge (Discord answered first) */
  handleCancel(cancel: ElicitationCancel): Promise<void>;
  /** Handle a run callback (Discord or external → Lobster resume) */
  handleRunCallback(runId: string, data: Record<string, unknown>): Promise<void>;
  /** Clean up resources */
  destroy(): void;
}

export function createElicitationHandler(
  linear: LinearClient,
  sessionManager: AgentSessionManager,
  runRegistry: RunRegistry,
  discordBridgeUrl: string,
  logger: { info: Function; warn: Function; error: Function },
): ElicitationHandler {
  const pending = new Map<string, PendingElicitation>();

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

  // ─── HTTP → Linear ──────────────────────────────────────────────────────

  async function handleRequest(request: ElicitationRequest): Promise<void> {
    const { elicitation_id, session_id, linear_issue_id } = request;

    if (!linear_issue_id) {
      logger.warn(`Elicitation ${elicitation_id}: no linear_issue_id — skipping Linear rendering`);
      return;
    }

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
        pending.delete(elicitation_id);

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

    pending.set(elicitation_id, {
      request,
      linearSessionId: session?.linearSessionId ?? '',
      timeoutTimer,
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
      } else {
        logger.info(`Agent session created: ${linearSessionId} (no deliberation mapping)`);
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
      // TODO: Forward stop to coordinator agent via /hooks/wake
      await linear.createAgentActivity({
        agentSessionId: linearSessionId,
        content: { type: 'response', body: 'Pipeline stop requested. Disengaging...' },
      }).catch((err) => logger.error(`Failed to post stop activity: ${err}`));
      return;
    }

    if (event.action !== 'prompted') return;

    const linearSessionId = event.data.id;
    const selectedValue = event.data.promptedValue;

    if (!selectedValue) {
      logger.warn(`Webhook prompted event missing promptedValue for session ${linearSessionId}`);
      return;
    }

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
      return;
    }

    if (matchedEntry.timeoutTimer) clearTimeout(matchedEntry.timeoutTimer);
    pending.delete(matchedId);

    const isRedeliberate = selectedValue === '__redeliberate__';

    // Build response
    const response = createElicitationResponse(
      matchedId,
      'linear',
      'linear-user',
      isRedeliberate
        ? { userContext: 'Human requested re-deliberation via Linear' }
        : { selectedOption: selectedValue },
    );

    // Cross-cancel to Discord
    const cancel = createElicitationCancel(
      matchedId,
      'linear',
      isRedeliberate ? undefined : selectedValue,
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
      const confirmBody = isRedeliberate
        ? 'Human requested re-deliberation via Linear'
        : `Human selected: **${selectedValue}**`;

      await linear.createAgentActivity({
        agentSessionId: matchedEntry.linearSessionId,
        content: { type: 'response', body: confirmBody },
      }).catch((err) => logger.error(`Failed to post confirmation activity: ${err}`));
    }

    logger.info(
      `Elicitation ${matchedId}: resolved via Linear — ${isRedeliberate ? 'redeliberate' : selectedValue}`,
    );
  }

  // ─── Run callback (from Discord or external) ─────────────────────────

  async function handleRunCallback(runId: string, data: Record<string, unknown>): Promise<void> {
    const run = runRegistry.lookup(runId);
    if (!run) {
      logger.warn(`Run callback for unknown run ${runId}`);
      return;
    }

    // The callback data is an ElicitationResponse from Discord
    const elicitationId = data.elicitation_id as string | undefined;
    if (elicitationId) {
      const entry = pending.get(elicitationId);
      if (entry) {
        if (entry.timeoutTimer) clearTimeout(entry.timeoutTimer);
        pending.delete(elicitationId);

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
    pending.delete(cancel.elicitation_id);

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
    }
    pending.clear();
  }

  return { handleRequest, handleWebhookEvent, handleCancel, handleRunCallback, destroy };
}
