import { mkdirSync, existsSync, readFileSync, renameSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { DatabaseSync } from "node:sqlite";
import type { ElicitationRequest } from "../elicitation-types.js";
import { BRIDGE_STATE_SCHEMA_SQL } from "../../../../shared/bridge-state-schema.js";

export type SessionStatus = "created" | "waiting_user" | "decision_made" | "failed" | "completed";

export interface PersistedElicitation {
  bridge: string;
  elicitationId: string;
  request: ElicitationRequest;
  status: "active" | "resolved";
  linearSessionId?: string;
  channelId?: string;
  messageId?: string;
  interactionKey?: string;
}

export interface DecisionQueryRow {
  session_id: string;
  decision_id: string;
  elicitation_id: string;
  question: string;
  category: string;
  selected_option: string | null;
  selected_by: string | null;
  consensus_strength: number;
  total_voters: number;
  updated_at: number;
}

export interface MemoryDecisionRow {
  session_id: string;
  decision_id: string;
  elicitation_id: string;
  question: string;
  category: string;
  selected_option: string | null;
  selected_label: string | null;
  selected_by: string | null;
  source_bridge: string | null;
  response_type: string | null;
  decided_at: number | null;
  consensus_strength: number;
  total_voters: number;
  updated_at: number;
}

export interface SessionQueryRow {
  session_id: string;
  status: SessionStatus;
  project_name: string | null;
  created_at: number;
  updated_at: number;
  last_error: string | null;
}

export interface DecisionAudit {
  elicitation: Record<string, unknown> | null;
  options: Array<Record<string, unknown>>;
  votes: Array<Record<string, unknown>>;
  tallies: Array<Record<string, unknown>>;
  outcome: Record<string, unknown> | null;
  providerEvents: Array<Record<string, unknown>>;
  messageRefs: Array<Record<string, unknown>>;
}

export interface PersistedDesignSnapshot {
  sessionId: string;
  runId?: string;
  projectName?: string;
  designMode?: string;
  stitchRequired: boolean;
  stitchStatus?: string;
  hasFrontend: boolean;
  artifactBundlePath?: string;
  context: Record<string, unknown>;
}

export interface DesignSnapshotRow {
  id: number;
  session_id: string;
  run_id: string | null;
  project_name: string | null;
  design_mode: string | null;
  stitch_required: number;
  stitch_status: string | null;
  has_frontend: number;
  artifact_bundle_path: string | null;
  context: Record<string, unknown>;
  created_at: number;
}

function nowMs(): number {
  return Date.now();
}

function parseJson<T>(raw: string): T | undefined {
  try {
    return JSON.parse(raw) as T;
  } catch {
    return undefined;
  }
}

export interface BridgeStateDb {
  dbPath: string;
  close(): void;
  setSessionStatus(sessionId: string, status: SessionStatus, projectName?: string, lastError?: string): void;
  appendProviderEvent(
    provider: string,
    eventType: string,
    payload: unknown,
    sessionId?: string,
    elicitationId?: string,
    runId?: string,
  ): void;
  saveElicitationPending(data: PersistedElicitation): void;
  markElicitationResolved(
    bridge: string,
    elicitationId: string,
    selectedOption?: string,
    selectedLabel?: string,
    selectedBy?: string,
    sourceBridge?: string,
    responseType?: string,
  ): void;
  getElicitationStatus(bridge: string, elicitationId: string): { active: boolean; known: boolean };
  getActiveElicitations(bridge: string): PersistedElicitation[];
  upsertProviderMessageRef(
    provider: string,
    elicitationId: string,
    refs: { channelId?: string; messageId?: string; threadId?: string; interactionKey?: string },
  ): void;
  listDecisions(sessionId?: string, limit?: number): DecisionQueryRow[];
  listResolvedDecisionsForMemory(sessionId?: string, limit?: number): MemoryDecisionRow[];
  listSessions(limit?: number, status?: SessionStatus): SessionQueryRow[];
  listWaitingSessions(limit?: number): SessionQueryRow[];
  getDecisionAudit(elicitationId: string, bridge?: string): DecisionAudit;
  saveDesignSnapshot(snapshot: PersistedDesignSnapshot): void;
  listDesignSnapshots(sessionId?: string, limit?: number): DesignSnapshotRow[];
  importLegacyDiscordStateJson(path: string, logger?: { info: Function; warn: Function }): void;
}

export function createBridgeStateDb(path: string): BridgeStateDb {
  mkdirSync(dirname(path), { recursive: true });
  const db = new DatabaseSync(path);
  // Two bridge processes share this file. WAL + busy timeout
  // reduces SQLITE_BUSY/locked failures under concurrent reads/writes.
  db.exec("PRAGMA journal_mode=WAL;");
  db.exec("PRAGMA synchronous=NORMAL;");
  db.exec("PRAGMA busy_timeout=5000;");
  db.exec(BRIDGE_STATE_SCHEMA_SQL);

  const upsertSessionStmt = db.prepare(`
    INSERT INTO sessions(session_id, status, project_name, created_at, updated_at, last_error)
    VALUES (?, ?, ?, ?, ?, ?)
    ON CONFLICT(session_id) DO UPDATE SET
      status=excluded.status,
      project_name=COALESCE(excluded.project_name, sessions.project_name),
      updated_at=excluded.updated_at,
      last_error=excluded.last_error
  `);
  const insertEventStmt = db.prepare(`
    INSERT INTO provider_events(provider, event_type, session_id, elicitation_id, run_id, payload_json, created_at)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `);
  const upsertElicitationStmt = db.prepare(`
    INSERT INTO elicitation_requests(
      bridge, elicitation_id, session_id, decision_id, question, category, status,
      request_json, linear_session_id, created_at, updated_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT(bridge, elicitation_id) DO UPDATE SET
      session_id=excluded.session_id,
      decision_id=excluded.decision_id,
      question=excluded.question,
      category=excluded.category,
      status=excluded.status,
      request_json=excluded.request_json,
      linear_session_id=excluded.linear_session_id,
      updated_at=excluded.updated_at
  `);
  const clearOptionsStmt = db.prepare(`DELETE FROM decision_options WHERE bridge = ? AND elicitation_id = ?`);
  const insertOptionStmt = db.prepare(`
    INSERT INTO decision_options(bridge, elicitation_id, option_value, label, description, advocated_by, vote_count)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `);
  const clearVotesStmt = db.prepare(`DELETE FROM votes WHERE bridge = ? AND elicitation_id = ?`);
  const insertVoteStmt = db.prepare(`
    INSERT INTO votes(bridge, elicitation_id, voter_id, chose, reasoning, created_at)
    VALUES (?, ?, ?, ?, ?, ?)
  `);
  const insertTallyStmt = db.prepare(`
    INSERT INTO tally_snapshots(bridge, elicitation_id, total_voters, consensus_strength, escalated, tally_json, created_at)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `);
  const upsertOutcomeStmt = db.prepare(`
    INSERT INTO decision_outcomes(bridge, elicitation_id, selected_option, selected_label, selected_by, source_bridge, response_type, decided_at)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT(bridge, elicitation_id) DO UPDATE SET
      selected_option=excluded.selected_option,
      selected_label=excluded.selected_label,
      selected_by=excluded.selected_by,
      source_bridge=excluded.source_bridge,
      response_type=excluded.response_type,
      decided_at=excluded.decided_at
  `);
  const markResolvedStmt = db.prepare(`
    UPDATE elicitation_requests
    SET status='resolved', updated_at=?
    WHERE bridge = ? AND elicitation_id = ?
  `);
  const getStatusStmt = db.prepare(`
    SELECT status FROM elicitation_requests
    WHERE bridge = ? AND elicitation_id = ?
    LIMIT 1
  `);
  const getActiveStmt = db.prepare(`
    SELECT bridge, elicitation_id, request_json, status, linear_session_id
    FROM elicitation_requests
    WHERE bridge = ? AND status = 'active'
    ORDER BY updated_at DESC
  `);
  const getProviderRefsStmt = db.prepare(`
    SELECT channel_id, message_id, thread_id, interaction_key
    FROM provider_message_refs
    WHERE provider = ? AND elicitation_id = ?
    LIMIT 1
  `);
  const upsertProviderRefStmt = db.prepare(`
    INSERT INTO provider_message_refs(provider, elicitation_id, channel_id, message_id, thread_id, interaction_key, updated_at)
    VALUES (?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT(provider, elicitation_id) DO UPDATE SET
      channel_id=COALESCE(excluded.channel_id, provider_message_refs.channel_id),
      message_id=COALESCE(excluded.message_id, provider_message_refs.message_id),
      thread_id=COALESCE(excluded.thread_id, provider_message_refs.thread_id),
      interaction_key=COALESCE(excluded.interaction_key, provider_message_refs.interaction_key),
      updated_at=excluded.updated_at
  `);
  const listDecisionsStmtBase = `
    SELECT er.session_id, er.decision_id, er.elicitation_id, er.question, er.category,
      o.selected_option, o.selected_by,
      ts.consensus_strength, ts.total_voters, er.updated_at
    FROM elicitation_requests er
    LEFT JOIN decision_outcomes o
      ON o.bridge = er.bridge AND o.elicitation_id = er.elicitation_id
    LEFT JOIN tally_snapshots ts
      ON ts.id = (
        SELECT id FROM tally_snapshots t2
        WHERE t2.bridge = er.bridge AND t2.elicitation_id = er.elicitation_id
        ORDER BY t2.created_at DESC
        LIMIT 1
      )
  `;
  const listResolvedDecisionsForMemoryStmtBase = `
    SELECT er.session_id, er.decision_id, er.elicitation_id, er.question, er.category,
      o.selected_option, o.selected_label, o.selected_by, o.source_bridge, o.response_type, o.decided_at,
      COALESCE(ts.consensus_strength, 0) AS consensus_strength,
      COALESCE(ts.total_voters, 0) AS total_voters,
      er.updated_at
    FROM elicitation_requests er
    LEFT JOIN decision_outcomes o
      ON o.elicitation_id = er.elicitation_id AND o.bridge = er.bridge
    LEFT JOIN decision_vote_tallies ts
      ON ts.elicitation_id = er.elicitation_id AND ts.bridge = er.bridge
    WHERE er.status = 'resolved'
  `;
  const listSessionsStmtBase = `
    SELECT session_id, status, project_name, created_at, updated_at, last_error
    FROM sessions
  `;
  const getElicitationAuditStmt = db.prepare(`
    SELECT bridge, elicitation_id, session_id, decision_id, question, category, status, request_json, linear_session_id, created_at, updated_at
    FROM elicitation_requests
    WHERE elicitation_id = ? AND (? IS NULL OR bridge = ?)
    ORDER BY updated_at DESC
    LIMIT 1
  `);
  const getOptionsAuditStmt = db.prepare(`
    SELECT bridge, elicitation_id, option_value, label, description, advocated_by, vote_count
    FROM decision_options
    WHERE elicitation_id = ? AND (? IS NULL OR bridge = ?)
    ORDER BY option_value ASC
  `);
  const getVotesAuditStmt = db.prepare(`
    SELECT bridge, elicitation_id, voter_id, chose, reasoning, created_at
    FROM votes
    WHERE elicitation_id = ? AND (? IS NULL OR bridge = ?)
    ORDER BY created_at ASC, voter_id ASC
  `);
  const getTalliesAuditStmt = db.prepare(`
    SELECT bridge, elicitation_id, total_voters, consensus_strength, escalated, tally_json, created_at
    FROM tally_snapshots
    WHERE elicitation_id = ? AND (? IS NULL OR bridge = ?)
    ORDER BY created_at ASC
  `);
  const getOutcomeAuditStmt = db.prepare(`
    SELECT bridge, elicitation_id, selected_option, selected_label, selected_by, source_bridge, response_type, decided_at
    FROM decision_outcomes
    WHERE elicitation_id = ? AND (? IS NULL OR bridge = ?)
    ORDER BY decided_at DESC
    LIMIT 1
  `);
  const getProviderEventsAuditStmt = db.prepare(`
    SELECT provider, event_type, session_id, elicitation_id, run_id, payload_json, created_at
    FROM provider_events
    WHERE elicitation_id = ? OR session_id = (
      SELECT session_id FROM elicitation_requests
      WHERE elicitation_id = ? AND (? IS NULL OR bridge = ?)
      ORDER BY updated_at DESC
      LIMIT 1
    )
    ORDER BY created_at ASC
  `);
  const getMessageRefsAuditStmt = db.prepare(`
    SELECT provider, elicitation_id, channel_id, message_id, thread_id, interaction_key, updated_at
    FROM provider_message_refs
    WHERE elicitation_id = ? AND (? IS NULL OR provider = ?)
    ORDER BY updated_at DESC
  `);
  const insertDesignSnapshotStmt = db.prepare(`
    INSERT INTO design_snapshots(
      session_id, run_id, project_name, design_mode, stitch_required, stitch_status, has_frontend,
      artifact_bundle_path, context_json, created_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const listDesignSnapshotsBase = `
    SELECT id, session_id, run_id, project_name, design_mode, stitch_required, stitch_status, has_frontend,
      artifact_bundle_path, context_json, created_at
    FROM design_snapshots
  `;

  return {
    dbPath: path,
    close() {
      db.close();
    },
    setSessionStatus(sessionId, status, projectName, lastError) {
      const ts = nowMs();
      upsertSessionStmt.run(sessionId, status, projectName ?? null, ts, ts, lastError ?? null);
    },
    appendProviderEvent(provider, eventType, payload, sessionId, elicitationId, runId) {
      insertEventStmt.run(
        provider,
        eventType,
        sessionId ?? null,
        elicitationId ?? null,
        runId ?? null,
        JSON.stringify(payload ?? {}),
        nowMs(),
      );
    },
    saveElicitationPending(data) {
      const ts = nowMs();
      const { request } = data;
      db.exec("BEGIN IMMEDIATE");
      try {
        upsertElicitationStmt.run(
          data.bridge,
          data.elicitationId,
          request.session_id,
          request.decision_id,
          request.question,
          request.category,
          "active",
          JSON.stringify(request),
          data.linearSessionId ?? null,
          ts,
          ts,
        );
        clearOptionsStmt.run(data.bridge, data.elicitationId);
        for (const opt of request.options) {
          insertOptionStmt.run(
            data.bridge,
            data.elicitationId,
            opt.value,
            opt.label,
            opt.description ?? null,
            opt.advocated_by ?? null,
            opt.vote_count ?? null,
          );
        }
        clearVotesStmt.run(data.bridge, data.elicitationId);
        for (const note of request.vote_summary.voter_notes ?? []) {
          insertVoteStmt.run(
            data.bridge,
            data.elicitationId,
            note.voter_id,
            note.chose,
            note.reasoning,
            ts,
          );
        }
        insertTallyStmt.run(
          data.bridge,
          data.elicitationId,
          request.vote_summary.total_voters ?? 0,
          request.vote_summary.consensus_strength ?? 0,
          request.vote_summary.escalated ? 1 : 0,
          JSON.stringify(request.vote_summary.tally ?? {}),
          ts,
        );
        db.exec("COMMIT");
      } catch (error) {
        db.exec("ROLLBACK");
        throw error;
      }
    },
    markElicitationResolved(bridge, elicitationId, selectedOption, selectedLabel, selectedBy, sourceBridge, responseType) {
      const ts = nowMs();
      markResolvedStmt.run(ts, bridge, elicitationId);
      upsertOutcomeStmt.run(
        bridge,
        elicitationId,
        selectedOption ?? null,
        selectedLabel ?? null,
        selectedBy ?? null,
        sourceBridge ?? null,
        responseType ?? null,
        ts,
      );
    },
    getElicitationStatus(bridge, elicitationId) {
      const row = getStatusStmt.get(bridge, elicitationId) as { status: string } | undefined;
      if (!row) return { active: false, known: false };
      return { active: row.status === "active", known: true };
    },
    getActiveElicitations(bridge) {
      const rows = getActiveStmt.all(bridge) as Array<{
        bridge: string;
        elicitation_id: string;
        request_json: string;
        status: "active" | "resolved";
        linear_session_id: string | null;
      }>;
      return rows
        .map((row) => {
          const req = parseJson<ElicitationRequest>(row.request_json);
          if (!req) return undefined;
          const refs = getProviderRefsStmt.get(bridge, row.elicitation_id) as
            | { channel_id: string | null; message_id: string | null; thread_id: string | null; interaction_key: string | null }
            | undefined;
          return {
            bridge: row.bridge,
            elicitationId: row.elicitation_id,
            request: req,
            status: row.status,
            linearSessionId: row.linear_session_id ?? undefined,
            channelId: refs?.channel_id ?? undefined,
            messageId: refs?.message_id ?? undefined,
            interactionKey: refs?.interaction_key ?? undefined,
          } as PersistedElicitation;
        })
        .filter((v) => v !== undefined) as PersistedElicitation[];
    },
    upsertProviderMessageRef(provider, elicitationId, refs) {
      upsertProviderRefStmt.run(
        provider,
        elicitationId,
        refs.channelId ?? null,
        refs.messageId ?? null,
        refs.threadId ?? null,
        refs.interactionKey ?? null,
        nowMs(),
      );
    },
    listDecisions(sessionId, limit = 100) {
      const sql = sessionId
        ? `${listDecisionsStmtBase} WHERE er.session_id = ? ORDER BY er.updated_at DESC LIMIT ?`
        : `${listDecisionsStmtBase} ORDER BY er.updated_at DESC LIMIT ?`;
      const stmt = db.prepare(sql);
      const rows = (sessionId ? stmt.all(sessionId, limit) : stmt.all(limit)) as DecisionQueryRow[];
      return rows;
    },
    listResolvedDecisionsForMemory(sessionId, limit = 100) {
      const sql = sessionId
        ? `${listResolvedDecisionsForMemoryStmtBase} AND er.session_id = ? ORDER BY er.updated_at DESC LIMIT ?`
        : `${listResolvedDecisionsForMemoryStmtBase} ORDER BY er.updated_at DESC LIMIT ?`;
      const stmt = db.prepare(sql);
      const rows = (sessionId ? stmt.all(sessionId, limit) : stmt.all(limit)) as MemoryDecisionRow[];
      return rows;
    },
    listSessions(limit = 200, status) {
      const sql = status
        ? `${listSessionsStmtBase} WHERE status = ? ORDER BY updated_at DESC LIMIT ?`
        : `${listSessionsStmtBase} ORDER BY updated_at DESC LIMIT ?`;
      const stmt = db.prepare(sql);
      const rows = (status ? stmt.all(status, limit) : stmt.all(limit)) as SessionQueryRow[];
      return rows;
    },
    listWaitingSessions(limit = 200) {
      const stmt = db.prepare(`${listSessionsStmtBase} WHERE status = 'waiting_user' ORDER BY updated_at DESC LIMIT ?`);
      return stmt.all(limit) as SessionQueryRow[];
    },
    getDecisionAudit(elicitationId, bridge) {
      const elicitation = (getElicitationAuditStmt.get(elicitationId, bridge ?? null, bridge ?? null) as Record<string, unknown> | undefined) ?? null;
      const options = getOptionsAuditStmt.all(elicitationId, bridge ?? null, bridge ?? null) as Array<Record<string, unknown>>;
      const votes = getVotesAuditStmt.all(elicitationId, bridge ?? null, bridge ?? null) as Array<Record<string, unknown>>;
      const talliesRaw = getTalliesAuditStmt.all(elicitationId, bridge ?? null, bridge ?? null) as Array<Record<string, unknown>>;
      const tallies = talliesRaw.map((row) => ({
        ...row,
        tally: parseJson<Record<string, number>>(String(row["tally_json"] ?? "{}")) ?? {},
      }));
      const outcome = (getOutcomeAuditStmt.get(elicitationId, bridge ?? null, bridge ?? null) as Record<string, unknown> | undefined) ?? null;
      const providerEventsRaw = getProviderEventsAuditStmt.all(elicitationId, elicitationId, bridge ?? null, bridge ?? null) as Array<Record<string, unknown>>;
      const providerEvents = providerEventsRaw.map((row) => ({
        ...row,
        payload: parseJson<unknown>(String(row["payload_json"] ?? "{}")) ?? {},
      }));
      const messageRefs = getMessageRefsAuditStmt.all(elicitationId, bridge ?? null, bridge ?? null) as Array<Record<string, unknown>>;
      return {
        elicitation,
        options,
        votes,
        tallies,
        outcome,
        providerEvents,
        messageRefs,
      };
    },
    saveDesignSnapshot(snapshot) {
      insertDesignSnapshotStmt.run(
        snapshot.sessionId,
        snapshot.runId ?? null,
        snapshot.projectName ?? null,
        snapshot.designMode ?? null,
        snapshot.stitchRequired ? 1 : 0,
        snapshot.stitchStatus ?? null,
        snapshot.hasFrontend ? 1 : 0,
        snapshot.artifactBundlePath ?? null,
        JSON.stringify(snapshot.context ?? {}),
        nowMs(),
      );
    },
    listDesignSnapshots(sessionId, limit = 50) {
      const sql = sessionId
        ? `${listDesignSnapshotsBase} WHERE session_id = ? ORDER BY created_at DESC LIMIT ?`
        : `${listDesignSnapshotsBase} ORDER BY created_at DESC LIMIT ?`;
      const stmt = db.prepare(sql);
      const rows = (sessionId ? stmt.all(sessionId, limit) : stmt.all(limit)) as Array<{
        id: number;
        session_id: string;
        run_id: string | null;
        project_name: string | null;
        design_mode: string | null;
        stitch_required: number;
        stitch_status: string | null;
        has_frontend: number;
        artifact_bundle_path: string | null;
        context_json: string;
        created_at: number;
      }>;
      return rows.map((row) => ({
        id: row.id,
        session_id: row.session_id,
        run_id: row.run_id,
        project_name: row.project_name,
        design_mode: row.design_mode,
        stitch_required: row.stitch_required,
        stitch_status: row.stitch_status,
        has_frontend: row.has_frontend,
        artifact_bundle_path: row.artifact_bundle_path,
        context: parseJson<Record<string, unknown>>(row.context_json) ?? {},
        created_at: row.created_at,
      }));
    },
    importLegacyDiscordStateJson(path, logger) {
      if (!existsSync(path)) return;
      try {
        const raw = readFileSync(path, "utf-8");
        const parsed = parseJson<{
          pending?: Array<{
            elicitationId: string;
            request: ElicitationRequest;
            channelId?: string;
            messageId?: string;
            interactionKey?: string;
          }>;
          resolved?: string[];
        }>(raw);
        if (!parsed) return;
        for (const row of parsed.pending ?? []) {
          if (!row?.elicitationId || !row?.request) continue;
          this.saveElicitationPending({
            bridge: "discord",
            elicitationId: row.elicitationId,
            request: row.request,
            status: "active",
          });
          this.upsertProviderMessageRef("discord", row.elicitationId, {
            channelId: row.channelId,
            messageId: row.messageId,
            interactionKey: row.interactionKey,
          });
        }
        for (const id of parsed.resolved ?? []) {
          if (typeof id !== "string") continue;
          this.markElicitationResolved("discord", id);
        }
        const archived = `${path}.migrated`;
        renameSync(path, archived);
        logger?.info?.(`Imported legacy Discord state file into SQLite and archived it to ${archived}`);
      } catch (err) {
        logger?.warn?.(`Failed importing legacy Discord state JSON: ${err}`);
      }
    },
  };
}

export function defaultBridgeStateDbPath(): string {
  const root = defaultWorkspaceRoot();
  return `${root}/.intake/bridge-state.db`;
}

export function defaultWorkspaceRoot(): string {
  if (process.env["WORKSPACE"]) return process.env["WORKSPACE"];
  let dir = resolve(process.cwd());
  for (;;) {
    if (existsSync(`${dir}/cto-config.json`)) return dir;
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return process.cwd();
}
