export const BRIDGE_STATE_SCHEMA_SQL = `
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sessions (
  session_id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  project_name TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  last_error TEXT
);

CREATE TABLE IF NOT EXISTS runs (
  run_id TEXT PRIMARY KEY,
  agent_pod TEXT NOT NULL,
  session_key TEXT NOT NULL,
  issue_id TEXT NOT NULL,
  linear_session_id TEXT,
  resume_token TEXT,
  registered_at INTEGER NOT NULL,
  last_accessed_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS elicitation_requests (
  bridge TEXT NOT NULL,
  elicitation_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  decision_id TEXT NOT NULL,
  question TEXT NOT NULL,
  category TEXT NOT NULL,
  status TEXT NOT NULL,
  request_json TEXT NOT NULL,
  linear_session_id TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (bridge, elicitation_id)
);

CREATE INDEX IF NOT EXISTS idx_elicitation_status
  ON elicitation_requests(bridge, status, updated_at);

CREATE TABLE IF NOT EXISTS decision_options (
  bridge TEXT NOT NULL,
  elicitation_id TEXT NOT NULL,
  option_value TEXT NOT NULL,
  label TEXT NOT NULL,
  description TEXT,
  advocated_by TEXT,
  vote_count INTEGER,
  PRIMARY KEY (bridge, elicitation_id, option_value)
);

CREATE TABLE IF NOT EXISTS votes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bridge TEXT NOT NULL,
  elicitation_id TEXT NOT NULL,
  voter_id TEXT NOT NULL,
  chose TEXT NOT NULL,
  reasoning TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_votes_elicitation ON votes(bridge, elicitation_id);

CREATE TABLE IF NOT EXISTS tally_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  bridge TEXT NOT NULL,
  elicitation_id TEXT NOT NULL,
  total_voters INTEGER NOT NULL,
  consensus_strength REAL NOT NULL,
  escalated INTEGER NOT NULL,
  tally_json TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tally_elicitation ON tally_snapshots(bridge, elicitation_id, created_at);

CREATE TABLE IF NOT EXISTS decision_outcomes (
  bridge TEXT NOT NULL,
  elicitation_id TEXT NOT NULL,
  selected_option TEXT,
  selected_label TEXT,
  selected_by TEXT,
  source_bridge TEXT,
  response_type TEXT,
  decided_at INTEGER NOT NULL,
  PRIMARY KEY (bridge, elicitation_id)
);

CREATE TABLE IF NOT EXISTS provider_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  provider TEXT NOT NULL,
  event_type TEXT NOT NULL,
  session_id TEXT,
  elicitation_id TEXT,
  run_id TEXT,
  payload_json TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_provider_events_session ON provider_events(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_provider_events_elicitation ON provider_events(elicitation_id, created_at);

CREATE TABLE IF NOT EXISTS provider_message_refs (
  provider TEXT NOT NULL,
  elicitation_id TEXT NOT NULL,
  channel_id TEXT,
  message_id TEXT,
  thread_id TEXT,
  interaction_key TEXT,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (provider, elicitation_id)
);

CREATE TABLE IF NOT EXISTS design_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id TEXT NOT NULL,
  run_id TEXT,
  project_name TEXT,
  design_mode TEXT,
  stitch_required INTEGER NOT NULL,
  stitch_status TEXT,
  has_frontend INTEGER NOT NULL,
  artifact_bundle_path TEXT,
  context_json TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_design_snapshots_session_created
  ON design_snapshots(session_id, created_at DESC);
`;
