import * as vscode from "vscode";
import * as fs from "fs";

const OFFLINE_AGENTS = [
  { id: "morgan", name: "Morgan", role: "Intake & PRD" },
  { id: "rex", name: "Rex", role: "Rust" },
  { id: "blaze", name: "Blaze", role: "Frontend" },
  { id: "grizz", name: "Grizz", role: "Go" },
  { id: "bolt", name: "Bolt", role: "DevOps" },
  { id: "tess", name: "Tess", role: "Testing" },
  { id: "cleo", name: "Cleo", role: "Code quality" },
  { id: "cipher", name: "Cipher", role: "Security" },
  { id: "nova", name: "Nova", role: "Research" },
  { id: "vex", name: "Vex", role: "Debugging" },
];

let inPodLogged = false;
function isInPod(): boolean {
  if (process.env.KUBERNETES_SERVICE_HOST) return true;
  try {
    return fs.existsSync("/var/run/secrets/kubernetes.io/serviceaccount/token");
  } catch {
    return false;
  }
}

export class SidebarProvider implements vscode.WebviewViewProvider {
  private _view?: vscode.WebviewView;

  constructor(
    private readonly _extensionUri: vscode.Uri,
    private readonly _context: vscode.ExtensionContext
  ) {}

  public postMessage(message: unknown) {
    this._view?.webview.postMessage(message);
  }

  private _gatewayConfig() {
    const config = vscode.workspace.getConfiguration("cto");
    const apiBase = config
      .get<string>("apiBase", "http://localhost:18789")
      .replace(/\/+$/, "");
    const apiToken = config.get<string>("apiToken", "openclaw-internal");
    const defaultAgent = config.get<string>("defaultAgent", "");
    const requestTimeoutMs = config.get<number>("requestTimeoutMs", 30000);
    const inPod = isInPod();
    if (inPod && !inPodLogged) {
      console.log("[cto-sidebar] detected in-pod environment; using " + apiBase);
      inPodLogged = true;
    }
    const narrator = {
      enabled: config.get<boolean>("narrator.enabled", false),
      musetalkUrl: config
        .get<string>("narrator.musetalkUrl", "/proxy/8081")
        .replace(/\/+$/, ""),
      hunyuanUrl: config
        .get<string>("narrator.hunyuanUrl", "/proxy/8082")
        .replace(/\/+$/, ""),
      defaultBackend: config.get<string>("narrator.defaultBackend", "musetalk"),
    };
    return { apiBase, apiToken, defaultAgent, requestTimeoutMs, inPod, narrator };
  }

  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    _ctx: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ) {
    this._view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this._extensionUri],
    };

    webviewView.webview.html = this._getHtml(webviewView.webview);

    // Handle messages from the webview
    webviewView.webview.onDidReceiveMessage(async (msg) => {
      switch (msg.type) {
        case "sendMessage": {
          await this._handleChatMessage(msg.text, msg.agent);
          break;
        }
        case "getAgents": {
          await this._fetchAgents();
          break;
        }
        case "getConfig": {
          const cfg = this._gatewayConfig();
          this.postMessage({
            type: "config",
            apiBase: cfg.apiBase,
            defaultAgent: cfg.defaultAgent,
            inPod: cfg.inPod,
            narrator: cfg.narrator,
          });
          break;
        }
      }
    });
  }

  private async _fetchAgents() {
    const { apiBase, apiToken } = this._gatewayConfig();

    const fallback = () => {
      this.postMessage({
        type: "agents",
        agents: OFFLINE_AGENTS.map((a) => ({ ...a, status: "offline" })),
        offline: true,
      });
    };

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 5000);

    try {
      const resp = await fetch(`${apiBase}/v1/models`, {
        headers: { Authorization: `Bearer ${apiToken}` },
        signal: controller.signal,
      });
      clearTimeout(timer);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const payload = (await resp.json()) as { data?: Array<{ id: string }> };
      const ids = (payload.data ?? [])
        .map((m) => m.id)
        .filter((id) => typeof id === "string");

      const agents = ids
        .map((id) => {
          const stripped = id.startsWith("openclaw/")
            ? id.slice("openclaw/".length)
            : id === "openclaw"
            ? ""
            : id;
          return stripped;
        })
        .filter((id) => id && id !== "default")
        .map((id) => {
          const match = OFFLINE_AGENTS.find((a) => a.id === id);
          return match
            ? { ...match, status: "online" as const }
            : { id, name: id.charAt(0).toUpperCase() + id.slice(1), role: "Agent", status: "online" as const };
        });

      if (agents.length === 0) {
        fallback();
        return;
      }
      this.postMessage({ type: "agents", agents, offline: false });
    } catch {
      clearTimeout(timer);
      fallback();
    }
  }

  private async _handleChatMessage(text: string, agent: string) {
    const { apiBase, apiToken, requestTimeoutMs } = this._gatewayConfig();

    const editor = vscode.window.activeTextEditor;
    const parts: string[] = [text];
    const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspace) parts.push(`\n\n_workspace:_ \`${workspace}\``);
    if (editor) {
      parts.push(`\n_file:_ \`${editor.document.fileName}\``);
      const sel = editor.selection;
      if (sel && !sel.isEmpty) {
        const selText = editor.document.getText(sel);
        parts.push(`\n\n_selection:_\n\`\`\`\n${selText}\n\`\`\``);
      }
    }
    const content = parts.join("");

    const model = agent ? `openclaw/${agent}` : "openclaw";

    const controller = new AbortController();
    let firstByte = false;
    const firstByteTimer = setTimeout(() => {
      if (!firstByte) controller.abort();
    }, requestTimeoutMs);

    try {
      const resp = await fetch(`${apiBase}/v1/chat/completions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${apiToken}`,
        },
        body: JSON.stringify({
          model,
          messages: [{ role: "user", content }],
          stream: true,
        }),
        signal: controller.signal,
      });

      if (!resp.ok) {
        clearTimeout(firstByteTimer);
        const body = await resp.text().catch(() => "");
        this.postMessage({
          type: "error",
          text: this._connectionErrorBanner(
            new Error(`HTTP ${resp.status}${body ? `: ${body.slice(0, 200)}` : ""}`),
            apiBase,
            resp.status,
            requestTimeoutMs
          ),
        });
        return;
      }

      if (!resp.body) {
        clearTimeout(firstByteTimer);
        this.postMessage({
          type: "error",
          text: `Gateway at ${apiBase} returned no stream body.`,
        });
        return;
      }

      const reader = resp.body.getReader();
      const decoder = new TextDecoder();
      let sseBuffer = "";
      this.postMessage({ type: "streamStart" });

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        if (!firstByte) {
          firstByte = true;
          clearTimeout(firstByteTimer);
        }

        sseBuffer += (decoder as { decode: (v: Uint8Array, opts?: { stream?: boolean }) => string }).decode(value, { stream: true });

        let sepIdx: number;
        // SSE events are delimited by a blank line
        // eslint-disable-next-line no-cond-assign
        while ((sepIdx = sseBuffer.indexOf("\n\n")) !== -1) {
          const rawEvent = sseBuffer.slice(0, sepIdx);
          sseBuffer = sseBuffer.slice(sepIdx + 2);
          const lines = rawEvent.split("\n");
          for (const line of lines) {
            if (!line.startsWith("data:")) continue;
            const data = line.slice(5).trim();
            if (!data) continue;
            if (data === "[DONE]") {
              this.postMessage({ type: "streamEnd" });
              return;
            }
            try {
              const evt = JSON.parse(data) as {
                choices?: Array<{ delta?: { content?: string }; finish_reason?: string | null }>;
              };
              const delta = evt.choices?.[0]?.delta?.content;
              if (typeof delta === "string" && delta.length > 0) {
                this.postMessage({ type: "streamChunk", delta });
              }
            } catch {
              // ignore malformed SSE frames
            }
          }
        }
      }

      this.postMessage({ type: "streamEnd" });
    } catch (err: unknown) {
      clearTimeout(firstByteTimer);
      if (firstByte) {
        const msg = err instanceof Error ? err.message : String(err);
        this.postMessage({ type: "error", text: `Stream interrupted: ${msg}` });
      } else {
        this.postMessage({
          type: "error",
          text: this._connectionErrorBanner(err, apiBase, undefined, requestTimeoutMs),
        });
      }
    }
  }

  private _connectionErrorBanner(
    err: unknown,
    apiBase: string,
    httpStatus: number | undefined,
    requestTimeoutMs: number
  ): string {
    const msg = err instanceof Error ? err.message : String(err);
    const name = err instanceof Error ? err.name : "";
    const seconds = Math.round(requestTimeoutMs / 1000);

    if (name === "AbortError" || /abort/i.test(msg)) {
      return `Gateway did not respond within ${seconds}s at ${apiBase}. Is the OpenClaw sidecar running? Try: kubectl -n cto exec <pod> -c agent -- curl -sS http://localhost:18789/health`;
    }
    if (/ECONNREFUSED|fetch failed|ENOTFOUND|network/i.test(msg)) {
      return `Cannot reach OpenClaw gateway at ${apiBase}. The sidecar may not be running, or the apiBase/port-forward is misconfigured. Check: curl -sS ${apiBase}/health`;
    }
    if (httpStatus === 401 || httpStatus === 403) {
      return `Gateway rejected the bearer token (HTTP ${httpStatus}). Check the cto.apiToken setting — it must match gateway.auth.token in /workspace/.openclaw/openclaw.json.`;
    }
    if (httpStatus && httpStatus >= 500) {
      return `Gateway returned ${httpStatus} at ${apiBase}. Check the agent pod logs (kubectl -n cto logs <pod> -c agent).`;
    }
    if (httpStatus && httpStatus >= 400) {
      return `Gateway error ${httpStatus} at ${apiBase}: ${msg}`;
    }
    return `Connection failed: ${msg}. Make sure the OpenClaw gateway is reachable at ${apiBase}.`;
  }

  private _getHtml(webview: vscode.Webview): string {
    const nonce = getNonce();

    return /*html*/ `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width,initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}'; font-src ${webview.cspSource}; media-src blob: mediastream: https: http://localhost:* http://127.0.0.1:*; img-src data: blob: ${webview.cspSource}; connect-src https: http: ws: wss: http://localhost:* http://127.0.0.1:*;">
  <style nonce="${nonce}">
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: var(--vscode-font-family);
      font-size: var(--vscode-font-size);
      color: var(--vscode-foreground);
      background: var(--vscode-panel-background, var(--vscode-sideBar-background));
      display: flex;
      flex-direction: column;
      height: 100vh;
      overflow: hidden;
    }

    /* Messages / content area */
    .messages {
      flex: 1;
      overflow-y: auto;
      padding: 0;
      display: flex;
      flex-direction: column;
    }

    /* Welcome — matches Copilot chat empty state */
    .welcome {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 8px;
      padding: 40px 20px;
      text-align: center;
    }
    .welcome-icon {
      width: 48px;
      height: 48px;
      opacity: 0.5;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    .welcome-icon svg { width: 48px; height: 48px; }
    .welcome-title {
      font-size: 14px;
      font-weight: 600;
      color: var(--vscode-foreground);
    }
    .welcome-sub {
      font-size: 12px;
      color: var(--vscode-descriptionForeground);
    }

    /* Message blocks — VS Code chat style (no bubbles) */
    .message {
      padding: 12px 16px;
      display: flex;
      flex-direction: column;
      gap: 6px;
    }
    .message + .message {
      border-top: 1px solid var(--vscode-panel-border, rgba(255,255,255,0.06));
    }
    .message-header {
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .message-avatar {
      width: 20px;
      height: 20px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 10px;
      font-weight: 700;
      flex-shrink: 0;
    }
    .message.user .message-avatar {
      background: var(--vscode-charts-purple, #9d8abf);
      color: #fff;
    }
    .message.assistant .message-avatar {
      background: var(--vscode-charts-blue, #4fc1ff);
      color: #fff;
    }
    .message-sender {
      font-size: 12px;
      font-weight: 600;
      color: var(--vscode-foreground);
    }
    .message-body {
      font-size: 13px;
      line-height: 1.55;
      white-space: pre-wrap;
      word-break: break-word;
      color: var(--vscode-foreground);
      padding-left: 26px;
    }
    .message-body code {
      font-family: var(--vscode-editor-font-family);
      font-size: 12px;
      background: var(--vscode-textCodeBlock-background, rgba(255,255,255,0.06));
      padding: 1px 5px;
      border-radius: 3px;
    }
    .message-body pre {
      background: var(--vscode-textCodeBlock-background, rgba(255,255,255,0.06));
      padding: 10px 12px;
      border-radius: 6px;
      overflow-x: auto;
      margin: 8px 0;
      font-size: 12px;
      line-height: 1.45;
    }

    /* Typing indicator */
    .typing {
      display: flex;
      gap: 4px;
      padding: 12px 16px 12px 42px;
    }
    .typing span {
      width: 5px;
      height: 5px;
      background: var(--vscode-descriptionForeground);
      border-radius: 50%;
      animation: bounce 1.4s infinite ease-in-out;
    }
    .typing span:nth-child(1) { animation-delay: 0s; }
    .typing span:nth-child(2) { animation-delay: 0.2s; }
    .typing span:nth-child(3) { animation-delay: 0.4s; }
    @keyframes bounce {
      0%,80%,100% { transform: translateY(0); }
      40% { transform: translateY(-6px); }
    }

    /* Input area — matches Copilot chat input box */
    .input-wrapper {
      padding: 0 12px 4px;
    }
    .input-box {
      background: var(--vscode-input-background);
      border: 1px solid var(--vscode-input-border, var(--vscode-panel-border));
      border-radius: 8px;
      display: flex;
      flex-direction: column;
      overflow: hidden;
      transition: border-color 0.15s;
    }
    .input-box:focus-within {
      border-color: var(--vscode-focusBorder);
    }
    .input-box textarea {
      flex: 1;
      background: transparent;
      color: var(--vscode-input-foreground);
      border: none;
      padding: 10px 12px 4px;
      font-family: var(--vscode-font-family);
      font-size: 13px;
      resize: none;
      min-height: 24px;
      max-height: 140px;
      line-height: 1.4;
      outline: none;
    }
    .input-box textarea::placeholder {
      color: var(--vscode-input-placeholderForeground);
    }
    .input-toolbar {
      display: flex;
      align-items: center;
      padding: 2px 6px 6px;
      gap: 2px;
    }
    .input-toolbar .spacer { flex: 1; }
    .toolbar-btn {
      width: 26px;
      height: 26px;
      border: none;
      border-radius: 5px;
      background: transparent;
      color: var(--vscode-descriptionForeground);
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: background 0.1s, color 0.1s;
    }
    .toolbar-btn:hover {
      background: var(--vscode-toolbar-hoverBackground, rgba(255,255,255,0.1));
      color: var(--vscode-foreground);
    }
    .toolbar-btn:disabled { opacity: 0.3; cursor: default; }
    .toolbar-btn svg { width: 16px; height: 16px; }
    .toolbar-btn.send-active {
      color: var(--vscode-button-foreground, #fff);
      background: var(--vscode-button-background, #0078d4);
    }
    .toolbar-btn.send-active:hover {
      background: var(--vscode-button-hoverBackground, #026ec1);
    }

    /* Footer bar — matches Copilot "Local / Default Approvals" bar */
    .footer-bar {
      display: flex;
      align-items: center;
      padding: 4px 12px 8px;
      gap: 8px;
      font-size: 12px;
    }
    .footer-item {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 2px 6px;
      border-radius: 4px;
      cursor: pointer;
      color: var(--vscode-descriptionForeground);
      transition: background 0.1s;
      user-select: none;
    }
    .footer-item:hover {
      background: var(--vscode-toolbar-hoverBackground, rgba(255,255,255,0.1));
      color: var(--vscode-foreground);
    }
    .footer-item .dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      flex-shrink: 0;
    }
    .footer-item .dot.online { background: #3fb950; }
    .footer-item .dot.offline { background: #f85149; }
    .footer-item .dot.connecting { background: #d29922; animation: pulse 1.5s infinite; }
    @keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.35; } }
    .footer-item .chevron {
      font-size: 10px;
      opacity: 0.6;
    }
    .footer-item.spacer { flex: 1; cursor: default; background: none; }

    /* Agent dropdown */
    .agent-dropdown {
      position: absolute;
      bottom: 100%;
      left: 0;
      margin-bottom: 4px;
      background: var(--vscode-dropdown-background, var(--vscode-input-background));
      border: 1px solid var(--vscode-dropdown-border, var(--vscode-panel-border));
      border-radius: 6px;
      padding: 4px;
      min-width: 180px;
      box-shadow: 0 4px 16px rgba(0,0,0,0.3);
      z-index: 100;
      display: none;
    }
    .agent-dropdown.show { display: block; }
    .agent-option {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 6px 10px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 12px;
      color: var(--vscode-foreground);
    }
    .agent-option:hover {
      background: var(--vscode-list-hoverBackground, rgba(255,255,255,0.06));
    }
    .agent-option.active {
      background: var(--vscode-list-activeSelectionBackground, rgba(255,255,255,0.1));
    }
    .agent-option .role {
      font-size: 11px;
      color: var(--vscode-descriptionForeground);
      margin-left: auto;
    }

    /* Avatar tile — live narration */
    .avatar-tile {
      display: none;
      flex-direction: column;
      border-bottom: 1px solid var(--vscode-panel-border, rgba(128,128,128,0.2));
      background: var(--vscode-textCodeBlock-background, rgba(0,0,0,0.15));
    }
    .avatar-tile.show { display: flex; }
    .avatar-tile-header {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 6px 10px;
      font-size: 11px;
      color: var(--vscode-descriptionForeground);
    }
    .avatar-tile-header .title { font-weight: 600; color: var(--vscode-foreground); }
    .avatar-tile-header .spacer { flex: 1; }
    .avatar-tile-toggle {
      background: transparent;
      border: 1px solid var(--vscode-panel-border, rgba(128,128,128,0.3));
      color: var(--vscode-foreground);
      padding: 2px 8px;
      border-radius: 3px;
      font-size: 11px;
      cursor: pointer;
      font-family: inherit;
    }
    .avatar-tile-toggle.active {
      background: var(--vscode-list-activeSelectionBackground, rgba(255,255,255,0.08));
      border-color: var(--vscode-charts-blue, #4da6ff);
    }
    .avatar-tile-toggle:hover {
      background: var(--vscode-list-hoverBackground, rgba(255,255,255,0.05));
    }
    .backend-segmented {
      display: inline-flex;
      border: 1px solid var(--vscode-panel-border, rgba(128,128,128,0.3));
      border-radius: 3px;
      overflow: hidden;
    }
    .backend-segmented button {
      background: transparent;
      border: none;
      color: var(--vscode-descriptionForeground);
      padding: 2px 10px;
      font-size: 11px;
      cursor: pointer;
      font-family: inherit;
    }
    .backend-segmented button.active {
      background: var(--vscode-charts-blue, #4da6ff);
      color: var(--vscode-editor-background, #1e1e1e);
      font-weight: 600;
    }
    .avatar-video-wrap {
      position: relative;
      width: 100%;
      aspect-ratio: 1 / 1;
      background: #000;
      overflow: hidden;
    }
    .avatar-video-wrap video {
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
    }
    .avatar-status {
      position: absolute;
      bottom: 6px;
      left: 6px;
      padding: 2px 6px;
      font-size: 10px;
      background: rgba(0,0,0,0.6);
      color: #fff;
      border-radius: 3px;
    }
    .avatar-controls {
      display: flex;
      gap: 6px;
      padding: 6px 10px;
      border-top: 1px solid var(--vscode-panel-border, rgba(128,128,128,0.15));
    }
    .avatar-controls .spacer { flex: 1; }
  </style>
</head>
<body>
  <div class="avatar-tile" id="avatarTile">
    <div class="avatar-tile-header">
      <span class="title">Blaze</span>
      <span class="backend-segmented" role="tablist" aria-label="Lipsync backend">
        <button id="backendMuseTalk" class="active" data-backend="musetalk">MuseTalk</button>
        <button id="backendHunyuan" data-backend="hunyuan">Hunyuan</button>
      </span>
      <span class="spacer"></span>
      <button class="avatar-tile-toggle active" id="avatarToggleVid" title="Video on/off">Vid</button>
      <button class="avatar-tile-toggle" id="avatarToggleMic" title="Voice interrupt">Mic</button>
      <button class="avatar-tile-toggle" id="avatarToggleTxt" title="Text interrupt">Txt</button>
    </div>
    <div class="avatar-video-wrap">
      <video id="avatarVideo" autoplay playsinline muted></video>
      <span class="avatar-status" id="avatarStatus">idle</span>
    </div>
    <div class="avatar-controls" id="avatarTextRow" style="display:none;">
      <input id="avatarTextInput" type="text" placeholder="Interrupt Blaze…" style="flex:1; background:transparent; border:1px solid var(--vscode-panel-border, rgba(128,128,128,0.3)); color:var(--vscode-foreground); padding:4px 8px; border-radius:3px; font-family:inherit; font-size:12px;">
      <button class="avatar-tile-toggle" id="avatarTextSend">Send</button>
    </div>
  </div>
  <div class="messages" id="messages">
    <div class="welcome" id="welcome">
      <div class="welcome-icon">
        <svg viewBox="0 0 48 48" fill="none">
          <rect x="4" y="4" width="40" height="40" rx="10" fill="var(--vscode-descriptionForeground)" opacity="0.15"/>
          <text x="24" y="30" text-anchor="middle" fill="var(--vscode-descriptionForeground)" font-size="18" font-weight="700" font-family="var(--vscode-font-family)">5D</text>
        </svg>
      </div>
      <div class="welcome-title">Build with CTO Agent</div>
      <div class="welcome-sub">AI responses may be inaccurate</div>
    </div>
  </div>

  <div class="input-wrapper">
    <div class="input-box">
      <textarea id="input" rows="1" placeholder="Describe what to build"></textarea>
      <div class="input-toolbar">
        <button class="toolbar-btn" title="Attach context" id="attachBtn">
          <svg viewBox="0 0 16 16" fill="currentColor"><path d="M11.5 1a3.5 3.5 0 0 1 .59 6.95L12 8v4.5a3.5 3.5 0 0 1-6.95.59L5 13V4.5a2.5 2.5 0 0 1 4.95-.49L10 4.5V12a1.5 1.5 0 0 1-2.95.35L7 12V4.5a.5.5 0 0 0-1 0V13a2.5 2.5 0 0 0 4.95.49L11 13V4.5a3.5 3.5 0 0 0-6.95-.49L4 4.5V13a4.5 4.5 0 0 0 8.95.49L13 13V4.5a3.5 3.5 0 0 0-1.5-3.5z"/></svg>
        </button>
        <div class="spacer"></div>
        <button class="toolbar-btn" id="sendBtn" title="Send (Enter)">
          <svg viewBox="0 0 16 16" fill="currentColor"><path d="M1.724 1.053a.5.5 0 0 1 .54-.068l12 6a.5.5 0 0 1 0 .894l-12 6A.5.5 0 0 1 1.5 13.5v-4.379l6.854-1.027a.125.125 0 0 0 0-.247L1.5 6.82V2.5a.5.5 0 0 1 .224-.447z"/></svg>
        </button>
      </div>
    </div>
  </div>

  <div class="footer-bar">
    <div class="footer-item" id="agentSelector" style="position:relative;">
      <span class="dot connecting" id="statusDot"></span>
      <span id="agentLabel">Auto</span>
      <span class="chevron">▾</span>
      <div class="agent-dropdown" id="agentDropdown">
        <div class="agent-option active" data-agent="auto">
          Auto <span class="role">best match</span>
        </div>
      </div>
    </div>
    <div class="footer-item spacer"></div>
    <div class="footer-item" id="connectionStatus">
      <span id="connectionLabel">Connecting</span>
    </div>
  </div>

  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const messagesEl = document.getElementById('messages');
    const welcomeEl = document.getElementById('welcome');
    const inputEl = document.getElementById('input');
    const sendBtn = document.getElementById('sendBtn');
    const statusDot = document.getElementById('statusDot');
    const agentLabel = document.getElementById('agentLabel');
    const agentSelector = document.getElementById('agentSelector');
    const agentDropdown = document.getElementById('agentDropdown');
    const connectionLabel = document.getElementById('connectionLabel');

    let selectedAgent = 'auto';
    let isStreaming = false;
    let messages = [];

    // Request agents on load
    vscode.postMessage({ type: 'getAgents' });
    vscode.postMessage({ type: 'getConfig' });

    // ── Narrator / AvatarTile (Live Narration PoC) ────────────────────
    const avatarTileEl = document.getElementById('avatarTile');
    const avatarVideoEl = document.getElementById('avatarVideo');
    const avatarStatusEl = document.getElementById('avatarStatus');
    const avatarToggleVid = document.getElementById('avatarToggleVid');
    const avatarToggleMic = document.getElementById('avatarToggleMic');
    const avatarToggleTxt = document.getElementById('avatarToggleTxt');
    const avatarTextRow = document.getElementById('avatarTextRow');
    const avatarTextInput = document.getElementById('avatarTextInput');
    const avatarTextSend = document.getElementById('avatarTextSend');
    const backendMuseTalkBtn = document.getElementById('backendMuseTalk');
    const backendHunyuanBtn = document.getElementById('backendHunyuan');

    let narratorCfg = null;        // { enabled, musetalkUrl, hunyuanUrl, defaultBackend }
    let currentBackend = 'musetalk';
    let pc = null;                 // RTCPeerConnection
    let narratorSessionId = null;
    let micStream = null;

    function backendBaseUrl() {
      if (!narratorCfg) return null;
      return currentBackend === 'hunyuan' ? narratorCfg.hunyuanUrl : narratorCfg.musetalkUrl;
    }

    function setAvatarStatus(s) { if (avatarStatusEl) avatarStatusEl.textContent = s; }

    async function startNarrator() {
      const base = backendBaseUrl();
      if (!base) { setAvatarStatus('no config'); return; }
      try {
        setAvatarStatus('connecting…');
        await stopNarrator(false);
        pc = new RTCPeerConnection({ iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] });
        pc.addTransceiver('video', { direction: 'recvonly' });
        pc.addTransceiver('audio', { direction: 'recvonly' });
        pc.ontrack = (e) => { if (avatarVideoEl.srcObject !== e.streams[0]) { avatarVideoEl.srcObject = e.streams[0]; avatarVideoEl.muted = false; } };
        pc.onconnectionstatechange = () => setAvatarStatus(pc ? pc.connectionState : 'closed');

        const offer = await pc.createOffer();
        await pc.setLocalDescription(offer);

        const resp = await fetch(base + '/sessions', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ persona_id: 'blaze', webrtc_offer: { sdp: offer.sdp, type: offer.type } }),
        });
        if (!resp.ok) throw new Error('sessions POST ' + resp.status);
        const data = await resp.json();
        narratorSessionId = data.session_id || data.id || null;
        await pc.setRemoteDescription(data.webrtc_answer || data.answer);
        setAvatarStatus('live · ' + currentBackend);
      } catch (err) {
        console.error('[narrator] start failed', err);
        setAvatarStatus('error');
      }
    }

    async function stopNarrator(updateStatus = true) {
      try {
        if (narratorSessionId && backendBaseUrl()) {
          fetch(backendBaseUrl() + '/sessions/' + narratorSessionId, { method: 'DELETE' }).catch(() => {});
        }
      } catch {}
      narratorSessionId = null;
      if (pc) { try { pc.close(); } catch {} pc = null; }
      if (avatarVideoEl) avatarVideoEl.srcObject = null;
      if (micStream) { micStream.getTracks().forEach(t => t.stop()); micStream = null; }
      if (updateStatus) setAvatarStatus('idle');
    }

    async function sendInterrupt(text, source) {
      const base = backendBaseUrl();
      if (!base || !narratorSessionId || !text) return;
      try {
        await fetch(base + '/sessions/' + narratorSessionId + '/interrupt', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ text, source: source || 'text' }),
        });
      } catch (err) { console.error('[narrator] interrupt failed', err); }
    }

    function setBackend(b) {
      if (b !== 'musetalk' && b !== 'hunyuan') return;
      if (b === currentBackend) return;
      currentBackend = b;
      backendMuseTalkBtn.classList.toggle('active', b === 'musetalk');
      backendHunyuanBtn.classList.toggle('active', b === 'hunyuan');
      if (pc) startNarrator();
    }

    backendMuseTalkBtn.addEventListener('click', () => setBackend('musetalk'));
    backendHunyuanBtn.addEventListener('click', () => setBackend('hunyuan'));

    avatarToggleVid.addEventListener('click', () => {
      const on = !avatarToggleVid.classList.contains('active');
      avatarToggleVid.classList.toggle('active', on);
      if (on) startNarrator(); else stopNarrator();
    });

    avatarToggleMic.addEventListener('click', async () => {
      const on = !avatarToggleMic.classList.contains('active');
      avatarToggleMic.classList.toggle('active', on);
      if (on) {
        try {
          micStream = await navigator.mediaDevices.getUserMedia({ audio: true });
          if (pc) micStream.getAudioTracks().forEach(t => pc.addTrack(t, micStream));
        } catch (err) { console.error('[narrator] mic denied', err); avatarToggleMic.classList.remove('active'); }
      } else if (micStream) {
        micStream.getTracks().forEach(t => t.stop()); micStream = null;
      }
    });

    avatarToggleTxt.addEventListener('click', () => {
      const on = !avatarToggleTxt.classList.contains('active');
      avatarToggleTxt.classList.toggle('active', on);
      avatarTextRow.style.display = on ? 'flex' : 'none';
    });

    avatarTextSend.addEventListener('click', () => {
      const t = avatarTextInput.value.trim();
      if (t) { sendInterrupt(t, 'text'); avatarTextInput.value = ''; }
    });
    avatarTextInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') { e.preventDefault(); avatarTextSend.click(); }
    });

    // Auto-resize textarea
    inputEl.addEventListener('input', () => {
      inputEl.style.height = 'auto';
      inputEl.style.height = Math.min(inputEl.scrollHeight, 140) + 'px';
      // Toggle send button active state
      sendBtn.classList.toggle('send-active', inputEl.value.trim().length > 0);
    });

    // Send on Enter (Shift+Enter for newline)
    inputEl.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
      }
    });

    sendBtn.addEventListener('click', sendMessage);

    // Agent dropdown toggle
    agentSelector.addEventListener('click', (e) => {
      e.stopPropagation();
      agentDropdown.classList.toggle('show');
    });
    document.addEventListener('click', () => {
      agentDropdown.classList.remove('show');
    });

    function sendMessage() {
      const text = inputEl.value.trim();
      if (!text || isStreaming) return;

      if (welcomeEl) welcomeEl.style.display = 'none';

      addMessage('user', text);
      inputEl.value = '';
      inputEl.style.height = 'auto';
      sendBtn.classList.remove('send-active');

      vscode.postMessage({ type: 'sendMessage', text, agent: selectedAgent });
      isStreaming = true;
      sendBtn.disabled = true;

      const typing = document.createElement('div');
      typing.className = 'typing';
      typing.id = 'typing';
      typing.innerHTML = '<span></span><span></span><span></span>';
      messagesEl.appendChild(typing);
      scrollToBottom();
    }

    function addMessage(role, text, agentName) {
      const div = document.createElement('div');
      div.className = 'message ' + role;

      const header = document.createElement('div');
      header.className = 'message-header';

      const avatar = document.createElement('div');
      avatar.className = 'message-avatar';
      avatar.textContent = role === 'user' ? 'U' : '5D';

      const sender = document.createElement('div');
      sender.className = 'message-sender';
      sender.textContent = role === 'user' ? 'You' : (agentName || 'CTO Agent');

      header.appendChild(avatar);
      header.appendChild(sender);

      const body = document.createElement('div');
      body.className = 'message-body';
      body.textContent = text;

      div.appendChild(header);
      div.appendChild(body);
      messagesEl.appendChild(div);
      scrollToBottom();
      return body;
    }

    function scrollToBottom() {
      messagesEl.scrollTop = messagesEl.scrollHeight;
    }

    let streamBubble = null;

    window.addEventListener('message', (event) => {
      const msg = event.data;

      switch (msg.type) {
        case 'agents': {
          const agentList = Array.isArray(msg.agents) ? msg.agents : [];
          agentDropdown.innerHTML = '<div class="agent-option active" data-agent="auto">Auto <span class="role">best match</span></div>';
          agentList.forEach(a => {
            const opt = document.createElement('div');
            opt.className = 'agent-option';
            opt.dataset.agent = a.id || a.name;
            opt.innerHTML = '<span class="dot ' + (a.status === 'online' ? 'online' : 'offline') + '"></span>'
              + (a.name || a.id)
              + '<span class="role">' + (a.role || '') + '</span>';
            agentDropdown.appendChild(opt);
          });

          // Status
          statusDot.className = 'dot ' + (msg.offline ? 'offline' : 'online');
          connectionLabel.textContent = msg.offline ? 'Offline' : 'Connected';

          // Click handlers for agent options
          agentDropdown.querySelectorAll('.agent-option').forEach(opt => {
            opt.addEventListener('click', (e) => {
              e.stopPropagation();
              agentDropdown.querySelectorAll('.agent-option').forEach(o => o.classList.remove('active'));
              opt.classList.add('active');
              selectedAgent = opt.dataset.agent;
              agentLabel.textContent = opt.textContent.replace(opt.querySelector('.role')?.textContent || '', '').trim();
              agentDropdown.classList.remove('show');
            });
          });
          break;
        }

        case 'streamStart': {
          const typingEl = document.getElementById('typing');
          if (typingEl) typingEl.remove();
          streamBubble = addMessage('assistant', '');
          break;
        }

        case 'streamChunk': {
          if (streamBubble) {
            const piece = msg.delta ?? msg.text ?? '';
            streamBubble.textContent = (streamBubble.textContent || '') + piece;
            scrollToBottom();
          }
          break;
        }

        case 'streamEnd': {
          if (streamBubble && typeof msg.text === 'string') streamBubble.textContent = msg.text;
          streamBubble = null;
          isStreaming = false;
          sendBtn.disabled = false;
          inputEl.focus();
          break;
        }

        case 'response': {
          const typingEl = document.getElementById('typing');
          if (typingEl) typingEl.remove();
          addMessage('assistant', msg.text);
          isStreaming = false;
          sendBtn.disabled = false;
          inputEl.focus();
          break;
        }

        case 'error': {
          const typingEl = document.getElementById('typing');
          if (typingEl) typingEl.remove();
          const body = addMessage('assistant', msg.text);
          body.style.color = 'var(--vscode-errorForeground, #f85149)';
          isStreaming = false;
          sendBtn.disabled = false;
          inputEl.focus();
          break;
        }

        case 'config': {
          narratorCfg = msg.narrator || null;
          if (narratorCfg && narratorCfg.enabled) {
            currentBackend = narratorCfg.defaultBackend === 'hunyuan' ? 'hunyuan' : 'musetalk';
            backendMuseTalkBtn.classList.toggle('active', currentBackend === 'musetalk');
            backendHunyuanBtn.classList.toggle('active', currentBackend === 'hunyuan');
            avatarTileEl.classList.add('show');
          } else {
            avatarTileEl.classList.remove('show');
          }
          break;
        }

        case 'narratorStart': { if (narratorCfg && narratorCfg.enabled) startNarrator(); break; }
        case 'narratorStop': { stopNarrator(); break; }
        case 'narratorInterrupt': { sendInterrupt(msg.text, msg.source || 'text'); break; }

        case 'newChat':
        case 'clearHistory': {
          messagesEl.innerHTML = '';
          if (welcomeEl) {
            messagesEl.appendChild(welcomeEl);
            welcomeEl.style.display = 'flex';
          }
          messages = [];
          break;
        }
      }
    });

    inputEl.focus();
  </script>
</body>
</html>`;
  }
}

function getNonce() {
  let text = "";
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return text;
}
