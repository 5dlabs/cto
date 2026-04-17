import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
  const handler: vscode.ChatRequestHandler = async (
    request,
    _chatContext,
    stream,
    token
  ) => {
    const config = vscode.workspace.getConfiguration("ctoChat");
    const apiBase = config.get<string>("apiBase", "http://localhost:18789");
    const apiToken = config.get<string>("apiToken", "openclaw-internal");
    const model = config.get<string>("model", "openclaw/coder");

    stream.progress("Connecting to CTO agent...");

    const controller = new AbortController();
    token.onCancellationRequested(() => controller.abort());

    try {
      const res = await fetch(`${apiBase}/v1/chat/completions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${apiToken}`,
        },
        body: JSON.stringify({
          model,
          messages: [{ role: "user", content: request.prompt }],
          stream: true,
        }),
        signal: controller.signal,
      });

      if (!res.ok) {
        const errBody = await res.text();
        stream.markdown(`**Error ${res.status}**: ${errBody}`);
        return {};
      }

      if (!res.body) {
        stream.markdown("No response body received.");
        return {};
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
            // skip malformed chunks
          }
        }
      }
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

  const participant = vscode.chat.createChatParticipant(
    "cto-chat.cto",
    handler
  );
  participant.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "icon.png"
  );
  context.subscriptions.push(participant);
}

export function deactivate() {}
