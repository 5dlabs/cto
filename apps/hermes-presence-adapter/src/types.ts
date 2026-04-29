export interface PresenceInbound {
  schema: "cto.presence.v1";
  event_type: "message" | "interaction" | "thread" | "lifecycle";
  runtime: "hermes";
  agent_id: string;
  project_id?: string;
  task_id?: string;
  coderun_id?: string;
  discord: {
    account_id: string;
    guild_id?: string;
    channel_id: string;
    thread_id?: string;
    message_id?: string;
    user_id?: string;
    user_name?: string;
    chat_type?: "dm" | "group" | "thread";
    parent_channel_id?: string;
  };
  text?: string;
  attachments?: Array<{ url: string; content_type?: string; filename?: string }>;
  metadata?: Record<string, string>;
  session_key?: string;
}

export interface AdapterConfig {
  port: number;
  hermesApiUrl?: string;
  hermesInputUrl?: string;
  inboxPath: string;
  presenceRouterUrl?: string;
  presenceSharedToken?: string;
  route?: PresenceRouteRegistration;
}

export interface PresenceRouteRegistration {
  route_id: string;
  runtime: "hermes";
  agent_id: string;
  project_id?: string;
  task_id?: string;
  coderun_id?: string;
  worker_url: string;
  session_key?: string;
  discord?: {
    account_id: string;
    guild_id?: string;
    channel_id?: string;
    thread_id?: string;
  };
  metadata?: Record<string, string>;
}

export interface HermesRunRequest {
  input: string;
  metadata: Record<string, string>;
  session?: {
    platform: "discord";
    chat_id: string;
    chat_type?: string;
    user_id?: string;
    user_name?: string;
    thread_id?: string;
  };
}

export interface HermesRunResponse {
  id?: string;
  run_id?: string;
  status?: string;
  output?: string;
}
