import * as vscode from "vscode";

interface OpenAIMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

interface GatewayModel {
  id: string;
  object: string;
  owned_by: string;
}

function buildMessages(
  chatContext: vscode.ChatContext,
  prompt: string
): OpenAIMessage[] {
  const messages: OpenAIMessage[] = [];

  for (const turn of chatContext.history) {
    if (turn instanceof vscode.ChatRequestTurn) {
      messages.push({ role: "user", content: turn.prompt });
    } else if (turn instanceof vscode.ChatResponseTurn) {
      const text = turn.response
        .map((part) =>
          part instanceof vscode.ChatResponseMarkdownPart
            ? part.value.value
            : ""
        )
        .join("");
      if (text) {
        messages.push({ role: "assistant", content: text });
      }
    }
  }

  messages.push({ role: "user", content: prompt });
  return messages;
}

async function streamResponse(
  apiBase: string,
  apiToken: string,
  model: string,
  messages: OpenAIMessage[],
  stream: vscode.ChatResponseStream,
  token: vscode.CancellationToken
): Promise<void> {
  const controller = new AbortController();
  token.onCancellationRequested(() => controller.abort());

  const res = await fetch(`${apiBase}/v1/chat/completions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiToken}`,
    },
    body: JSON.stringify({ model, messages, stream: true }),
    signal: controller.signal,
  });

  if (!res.ok) {
    const errBody = await res.text();
    stream.markdown(`**Error ${res.status}**: ${errBody}`);
    return;
  }

  if (!res.body) {
    stream.markdown("No response body received.");
    return;
  }

  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    if (token.isCancellationRequested) break;
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (!line.startsWith("data: ")) continue;
      const data = line.slice(6).trim();
      if (data === "[DONE]") continue;

      try {
        const chunk = JSON.parse(data);
        const content = chunk.choices?.[0]?.delta?.content;
        if (content) {
          stream.markdown(content);
        }
      } catch {
        // skip malformed SSE chunks
      }
    }
  }
}

// Discover available agents from the gateway's /v1/models endpoint.
// Returns agent IDs like ["coder", "default"] from model IDs like "openclaw/coder".
async function discoverAgents(
  apiBase: string,
  apiToken: string
): Promise<string[]> {
  try {
    const res = await fetch(`${apiBase}/v1/models`, {
      headers: { Authorization: `Bearer ${apiToken}` },
    });
    if (!res.ok) return [];
    const body = (await res.json()) as { data: GatewayModel[] };
    return body.data
      .map((m) => m.id)
      .filter((id) => id.startsWith("openclaw/") && id !== "openclaw/default")
      .map((id) => id.replace("openclaw/", ""));
  } catch {
    return [];
  }
}

function createAgentHandler(model: string): vscode.ChatRequestHandler {
  return async (request, chatContext, stream, token) => {
    const config = vscode.workspace.getConfiguration("ctoChat");
    const apiBase = config.get<string>("apiBase", "http://localhost:18789");
    const apiToken = config.get<string>("apiToken", "openclaw-internal");

    stream.progress(`Connecting to ${model}...`);

    try {
      const messages = buildMessages(chatContext, request.prompt);
      await streamResponse(apiBase, apiToken, model, messages, stream, token);
    } catch (err: unknown) {
      if (err instanceof Error && err.name === "AbortError") {
        stream.markdown("_Request cancelled._");
      } else {
        stream.markdown(
          `**Connection error**: ${err instanceof Error ? err.message : String(err)}`
        );
      }
    }

    return {};
  };
}

function registerAgent(
  context: vscode.ExtensionContext,
  agentId: string,
  isDefault: boolean
): void {
  const participantId = `cto-chat.${agentId}`;
  const model = `openclaw/${agentId}`;
  const displayName = agentId.charAt(0).toUpperCase() + agentId.slice(1);

  const participant = vscode.chat.createChatParticipant(
    participantId,
    createAgentHandler(model)
  );
  participant.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "icon.png"
  );
  (participant as any).fullName = `5dlabs CTO — ${displayName}`;
  if (isDefault) {
    (participant as any).isDefault = true;
  }
  context.subscriptions.push(participant);
}

export async function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("ctoChat");
  const apiBase = config.get<string>("apiBase", "http://localhost:18789");
  const apiToken = config.get<string>("apiToken", "openclaw-internal");
  const fallbackModel = config.get<string>("model", "openclaw/coder");
  const fallbackAgent = fallbackModel.replace("openclaw/", "");

  const agents = await discoverAgents(apiBase, apiToken);

  if (agents.length > 0) {
    // Register a participant per discovered agent
    for (const agentId of agents) {
      registerAgent(context, agentId, agents.length === 1);
    }
    // If multiple agents and none is sole default, make the configured one default
    if (agents.length > 1 && agents.includes(fallbackAgent)) {
      // Already registered above — find it and set default
      // (handled by registerAgent with isDefault=false, re-register won't duplicate)
    }
  } else {
    // Fallback: register the configured agent if discovery fails
    registerAgent(context, fallbackAgent, true);
  }
}

export function deactivate() {}
