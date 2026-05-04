import fs from "node:fs/promises";
import path from "node:path";
import type { HermesRunRequest, HermesRunResponse, PresenceInbound } from "./types.js";

const HERMES_FETCH_TIMEOUT_MS = 30_000;
const PRESENCE_FETCH_TIMEOUT_MS = 10_000;

function jsonHeaders(presenceSharedToken?: string): Record<string, string> {
  return {
    "Content-Type": "application/json",
    ...(presenceSharedToken ? { Authorization: `Bearer ${presenceSharedToken}` } : {}),
  };
}

function buildHermesRun(event: PresenceInbound): HermesRunRequest {
  const content = event.text?.trim() || "(The user sent a message with no text content)";
  const attachmentText = event.attachments?.length
    ? `\n\nAttachments:\n${event.attachments.map((a) => `- ${a.filename ?? "attachment"}: ${a.url}`).join("\n")}`
    : "";
  return {
    input: `${content}${attachmentText}`,
    metadata: {
      schema: event.schema,
      runtime: event.runtime,
      agent_id: event.agent_id,
      project_id: event.project_id ?? "",
      task_id: event.task_id ?? "",
      coderun_id: event.coderun_id ?? "",
      discord_account_id: event.discord.account_id,
      discord_channel_id: event.discord.channel_id,
      discord_thread_id: event.discord.thread_id ?? "",
      discord_message_id: event.discord.message_id ?? "",
      discord_reference_message_id: event.discord.reference_message_id ?? "",
      discord_reference_channel_id: event.discord.reference_channel_id ?? "",
      discord_reference_guild_id: event.discord.reference_guild_id ?? "",
      ...(event.metadata ?? {}),
      session_key: event.session_key ?? "",
    },
    session: {
      platform: "discord",
      chat_id: event.discord.thread_id ?? event.discord.channel_id,
      chat_type: event.discord.chat_type,
      user_id: event.discord.user_id,
      user_name: event.discord.user_name,
      thread_id: event.discord.thread_id,
      home_id: event.metadata?.home_id,
      home_route_id: event.metadata?.home_route_id,
      route_id: event.metadata?.route_id,
    },
  };
}

export async function startHermesRun(hermesApiUrl: string, event: PresenceInbound): Promise<HermesRunResponse> {
  const response = await fetch(`${hermesApiUrl}/v1/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(buildHermesRun(event)),
    signal: AbortSignal.timeout(HERMES_FETCH_TIMEOUT_MS),
  });
  if (!response.ok) {
    const body = await response.text().catch(() => "");
    throw new Error(`Hermes API returned HTTP ${response.status}${body ? `: ${body}` : ""}`);
  }
  return (await response.json()) as HermesRunResponse;
}

function inputText(event: PresenceInbound): string {
  const content = event.text?.trim() || "(The user sent a message with no text content)";
  const attachmentText = event.attachments?.length
    ? `\n\nAttachments:\n${event.attachments.map((a) => `- ${a.filename ?? "attachment"}: ${a.url}`).join("\n")}`
    : "";
  return `${content}${attachmentText}`;
}

export async function postHermesInput(hermesInputUrl: string, event: PresenceInbound): Promise<HermesRunResponse> {
  const response = await fetch(hermesInputUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(buildHermesRun(event)),
    signal: AbortSignal.timeout(HERMES_FETCH_TIMEOUT_MS),
  });
  if (!response.ok) {
    const body = await response.text().catch(() => "");
    throw new Error(`Hermes input endpoint returned HTTP ${response.status}${body ? `: ${body}` : ""}`);
  }
  return { id: event.coderun_id, status: "queued" };
}

export async function appendInbox(inboxPath: string, event: PresenceInbound): Promise<HermesRunResponse> {
  await fs.mkdir(path.dirname(inboxPath), { recursive: true });
  await fs.appendFile(inboxPath, `${JSON.stringify({ received_at: new Date().toISOString(), event })}\n`, "utf8");
  return { id: event.coderun_id, status: "queued" };
}

export async function postPresenceStatus(
  presenceRouterUrl: string | undefined,
  presenceSharedToken: string | undefined,
  event: PresenceInbound,
  state: "started" | "running" | "blocked" | "done" | "failed",
  detail?: string,
): Promise<void> {
  if (!presenceRouterUrl) {
    return;
  }
  const response = await fetch(`${presenceRouterUrl}/presence/outbound`, {
    method: "POST",
    headers: jsonHeaders(presenceSharedToken),
    body: JSON.stringify({
      op: "status",
      state,
      detail,
      target: {
        account_id: event.discord.account_id,
        channel_id: event.discord.channel_id,
        thread_id: event.discord.thread_id,
      },
      message_id: event.discord.message_id,
    }),
    signal: AbortSignal.timeout(PRESENCE_FETCH_TIMEOUT_MS),
  });
  if (!response.ok) {
    const body = await response.text().catch(() => "");
    throw new Error(`Presence status post returned HTTP ${response.status}${body ? `: ${body}` : ""}`);
  }
}

export async function registerPresenceRoute(
  presenceRouterUrl: string | undefined,
  presenceSharedToken: string | undefined,
  route: unknown,
): Promise<void> {
  if (!presenceRouterUrl || !route) {
    return;
  }
  const response = await fetch(`${presenceRouterUrl}/presence/routes`, {
    method: "POST",
    headers: jsonHeaders(presenceSharedToken),
    body: JSON.stringify(route),
    signal: AbortSignal.timeout(PRESENCE_FETCH_TIMEOUT_MS),
  });
  if (!response.ok) {
    const body = await response.text().catch(() => "");
    throw new Error(`Presence route registration returned HTTP ${response.status}${body ? `: ${body}` : ""}`);
  }
}

export async function deletePresenceRoute(
  presenceRouterUrl: string | undefined,
  presenceSharedToken: string | undefined,
  routeId: string | undefined,
): Promise<void> {
  if (!presenceRouterUrl || !routeId) {
    return;
  }
  const response = await fetch(`${presenceRouterUrl}/presence/routes/${encodeURIComponent(routeId)}`, {
    method: "DELETE",
    headers: jsonHeaders(presenceSharedToken),
    signal: AbortSignal.timeout(PRESENCE_FETCH_TIMEOUT_MS),
  });
  if (!response.ok && response.status !== 404) {
    const body = await response.text().catch(() => "");
    throw new Error(`Presence route delete returned HTTP ${response.status}${body ? `: ${body}` : ""}`);
  }
}
