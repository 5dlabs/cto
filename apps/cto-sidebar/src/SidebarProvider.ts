import * as vscode from "vscode";

export class SidebarProvider implements vscode.WebviewViewProvider {
  private _view?: vscode.WebviewView;

  constructor(
    private readonly _extensionUri: vscode.Uri,
    private readonly _context: vscode.ExtensionContext
  ) {}

  public postMessage(message: unknown) {
    this._view?.webview.postMessage(message);
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
          const config = vscode.workspace.getConfiguration("cto");
          this.postMessage({
            type: "config",
            apiBase: config.get<string>("apiBase", "http://localhost:18789"),
            defaultAgent: config.get<string>("defaultAgent", ""),
          });
          break;
        }
      }
    });
  }

  private async _fetchAgents() {
    const config = vscode.workspace.getConfiguration("cto");
    const apiBase = config.get<string>("apiBase", "http://localhost:18789");

    try {
      const resp = await fetch(`${apiBase}/agents`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const agents = await resp.json();
      this.postMessage({ type: "agents", agents });
    } catch {
      // Fallback agent list when gateway is unreachable
      this.postMessage({
        type: "agents",
        agents: [
          { id: "morgan", name: "Morgan", role: "Intake & PRD", status: "offline" },
          { id: "rex", name: "Rex", role: "Rust", status: "offline" },
          { id: "blaze", name: "Blaze", role: "Frontend", status: "offline" },
          { id: "bolt", name: "Bolt", role: "DevOps", status: "offline" },
        ],
        offline: true,
      });
    }
  }

  private async _handleChatMessage(text: string, agent: string) {
    const config = vscode.workspace.getConfiguration("cto");
    const apiBase = config.get<string>("apiBase", "http://localhost:18789");

    // Stream response from ACP/OpenClaw
    try {
      const resp = await fetch(`${apiBase}/chat`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message: text,
          agent,
          context: {
            file: vscode.window.activeTextEditor?.document.fileName,
            selection: vscode.window.activeTextEditor?.selection
              ? vscode.window.activeTextEditor.document.getText(
                  vscode.window.activeTextEditor.selection
                )
              : undefined,
            workspace: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
          },
        }),
      });

      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);

      // Handle streaming response
      if (resp.body) {
        const reader = resp.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        this.postMessage({ type: "streamStart" });

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += (decoder as any).decode(value, { stream: true });
          this.postMessage({ type: "streamChunk", text: buffer });
        }

        this.postMessage({ type: "streamEnd", text: buffer });
      } else {
        const data = (await resp.json()) as Record<string, unknown>;
        this.postMessage({ type: "response", text: (data.response ?? data.message ?? JSON.stringify(data)) as string });
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.postMessage({
        type: "error",
        text: `Connection failed: ${msg}\n\nMake sure your CTO gateway is running at ${apiBase}`,
      });
    }
  }

  private _getHtml(webview: vscode.Webview): string {
    const nonce = getNonce();

    return /*html*/ `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width,initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}'; font-src ${webview.cspSource};">
  <style nonce="${nonce}">
    :root {
      --cto-accent: #6c5ce7;
      --cto-accent-hover: #5a4bd1;
      --cto-green: #00b894;
      --cto-red: #e17055;
      --cto-orange: #fdcb6e;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: var(--vscode-font-family);
      font-size: var(--vscode-font-size);
      color: var(--vscode-foreground);
      background: var(--vscode-sideBar-background);
      display: flex;
      flex-direction: column;
      height: 100vh;
      overflow: hidden;
    }

    /* Header */
    .header {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 10px 12px 8px;
      border-bottom: 1px solid var(--vscode-panel-border);
    }
    .header-logo {
      width: 22px;
      height: 22px;
      background: var(--cto-accent);
      border-radius: 5px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 700;
      font-size: 11px;
      color: #fff;
      flex-shrink: 0;
    }
    .header-title {
      font-weight: 600;
      font-size: 12px;
      flex: 1;
    }
    .header-status {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      flex-shrink: 0;
    }
    .header-status.online { background: var(--cto-green); }
    .header-status.offline { background: var(--cto-red); }
    .header-status.connecting { background: var(--cto-orange); animation: pulse 1.5s infinite; }
    @keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.4; } }

    /* Agent selector */
    .agent-bar {
      display: flex;
      gap: 4px;
      padding: 6px 12px;
      overflow-x: auto;
      border-bottom: 1px solid var(--vscode-panel-border);
      scrollbar-width: none;
    }
    .agent-bar::-webkit-scrollbar { display: none; }
    .agent-chip {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 3px 8px;
      border-radius: 12px;
      font-size: 11px;
      cursor: pointer;
      white-space: nowrap;
      background: var(--vscode-badge-background);
      color: var(--vscode-badge-foreground);
      border: 1px solid transparent;
      transition: all 0.15s;
    }
    .agent-chip:hover { border-color: var(--cto-accent); }
    .agent-chip.active {
      background: var(--cto-accent);
      color: #fff;
    }
    .agent-chip .dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
    }
    .agent-chip .dot.online { background: var(--cto-green); }
    .agent-chip .dot.offline { background: var(--cto-red); }

    /* Messages area */
    .messages {
      flex: 1;
      overflow-y: auto;
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 10px;
    }
    .message {
      display: flex;
      flex-direction: column;
      gap: 4px;
      max-width: 95%;
    }
    .message.user { align-self: flex-end; }
    .message.assistant { align-self: flex-start; }
    .message-meta {
      font-size: 10px;
      opacity: 0.6;
      padding: 0 4px;
    }
    .message-bubble {
      padding: 8px 12px;
      border-radius: 10px;
      font-size: 12.5px;
      line-height: 1.5;
      white-space: pre-wrap;
      word-break: break-word;
    }
    .message.user .message-bubble {
      background: var(--cto-accent);
      color: #fff;
      border-bottom-right-radius: 3px;
    }
    .message.assistant .message-bubble {
      background: var(--vscode-editor-background);
      border: 1px solid var(--vscode-panel-border);
      border-bottom-left-radius: 3px;
    }
    .message-bubble code {
      font-family: var(--vscode-editor-font-family);
      font-size: 11.5px;
      background: rgba(0,0,0,0.15);
      padding: 1px 4px;
      border-radius: 3px;
    }
    .message-bubble pre {
      background: var(--vscode-textCodeBlock-background);
      padding: 8px;
      border-radius: 4px;
      overflow-x: auto;
      margin: 6px 0;
    }

    /* Welcome */
    .welcome {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 12px;
      padding: 20px;
      text-align: center;
      opacity: 0.7;
    }
    .welcome-icon {
      width: 48px;
      height: 48px;
      background: var(--cto-accent);
      border-radius: 12px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 22px;
      font-weight: 700;
      color: #fff;
    }
    .welcome h3 { font-size: 14px; }
    .welcome p { font-size: 12px; line-height: 1.5; }

    /* Typing indicator */
    .typing {
      display: flex;
      gap: 4px;
      padding: 8px 12px;
      align-self: flex-start;
    }
    .typing span {
      width: 6px;
      height: 6px;
      background: var(--cto-accent);
      border-radius: 50%;
      animation: bounce 1.4s infinite ease-in-out;
    }
    .typing span:nth-child(1) { animation-delay: 0s; }
    .typing span:nth-child(2) { animation-delay: 0.2s; }
    .typing span:nth-child(3) { animation-delay: 0.4s; }
    @keyframes bounce {
      0%,80%,100% { transform: translateY(0); }
      40% { transform: translateY(-8px); }
    }

    /* Input area */
    .input-area {
      border-top: 1px solid var(--vscode-panel-border);
      padding: 8px 12px;
      display: flex;
      gap: 6px;
      align-items: flex-end;
    }
    .input-area textarea {
      flex: 1;
      background: var(--vscode-input-background);
      color: var(--vscode-input-foreground);
      border: 1px solid var(--vscode-input-border);
      border-radius: 8px;
      padding: 8px 10px;
      font-family: var(--vscode-font-family);
      font-size: 12.5px;
      resize: none;
      min-height: 36px;
      max-height: 120px;
      line-height: 1.4;
      outline: none;
    }
    .input-area textarea:focus {
      border-color: var(--cto-accent);
    }
    .input-area textarea::placeholder {
      color: var(--vscode-input-placeholderForeground);
    }
    .send-btn {
      width: 32px;
      height: 32px;
      border: none;
      border-radius: 8px;
      background: var(--cto-accent);
      color: #fff;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      transition: background 0.15s;
    }
    .send-btn:hover { background: var(--cto-accent-hover); }
    .send-btn:disabled { opacity: 0.4; cursor: default; }
    .send-btn svg { width: 16px; height: 16px; }
  </style>
</head>
<body>
  <div class="header">
    <div class="header-logo">5D</div>
    <div class="header-title">5dlabs CTO</div>
    <div class="header-status connecting" id="statusDot" title="Connecting..."></div>
  </div>

  <div class="agent-bar" id="agentBar">
    <div class="agent-chip active" data-agent="auto">Auto</div>
  </div>

  <div class="messages" id="messages">
    <div class="welcome" id="welcome">
      <div class="welcome-icon">5D</div>
      <h3>5dlabs CTO Agents</h3>
      <p>Ask anything. Your message is routed<br>to the best available agent.</p>
    </div>
  </div>

  <div class="input-area">
    <textarea id="input" rows="1" placeholder="Ask CTO agents..."></textarea>
    <button class="send-btn" id="sendBtn" title="Send">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="22" y1="2" x2="11" y2="13"></line>
        <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
      </svg>
    </button>
  </div>

  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const messagesEl = document.getElementById('messages');
    const welcomeEl = document.getElementById('welcome');
    const inputEl = document.getElementById('input');
    const sendBtn = document.getElementById('sendBtn');
    const agentBar = document.getElementById('agentBar');
    const statusDot = document.getElementById('statusDot');

    let selectedAgent = 'auto';
    let isStreaming = false;
    let messages = [];

    // Request agents on load
    vscode.postMessage({ type: 'getAgents' });
    vscode.postMessage({ type: 'getConfig' });

    // Auto-resize textarea
    inputEl.addEventListener('input', () => {
      inputEl.style.height = 'auto';
      inputEl.style.height = Math.min(inputEl.scrollHeight, 120) + 'px';
    });

    // Send on Enter (Shift+Enter for newline)
    inputEl.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
      }
    });

    sendBtn.addEventListener('click', sendMessage);

    function sendMessage() {
      const text = inputEl.value.trim();
      if (!text || isStreaming) return;

      // Hide welcome
      if (welcomeEl) welcomeEl.style.display = 'none';

      // Add user message
      addMessage('user', text);
      inputEl.value = '';
      inputEl.style.height = 'auto';

      // Send to extension
      vscode.postMessage({ type: 'sendMessage', text, agent: selectedAgent });
      isStreaming = true;
      sendBtn.disabled = true;

      // Show typing indicator
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

      const meta = document.createElement('div');
      meta.className = 'message-meta';
      meta.textContent = role === 'user' ? 'You' : (agentName || 'CTO');

      const bubble = document.createElement('div');
      bubble.className = 'message-bubble';
      bubble.textContent = text;

      div.appendChild(meta);
      div.appendChild(bubble);
      messagesEl.appendChild(div);
      scrollToBottom();
      return bubble;
    }

    function scrollToBottom() {
      messagesEl.scrollTop = messagesEl.scrollHeight;
    }

    let streamBubble = null;

    // Handle messages from extension
    window.addEventListener('message', (event) => {
      const msg = event.data;

      switch (msg.type) {
        case 'agents': {
          agentBar.innerHTML = '<div class="agent-chip active" data-agent="auto">Auto</div>';
          const agentList = Array.isArray(msg.agents) ? msg.agents : [];
          agentList.forEach(a => {
            const chip = document.createElement('div');
            chip.className = 'agent-chip';
            chip.dataset.agent = a.id || a.name;
            chip.innerHTML = '<span class="dot ' + (a.status === 'online' ? 'online' : 'offline') + '"></span>' + (a.name || a.id);
            agentBar.appendChild(chip);
          });

          // Update status
          statusDot.className = 'header-status ' + (msg.offline ? 'offline' : 'online');
          statusDot.title = msg.offline ? 'Gateway offline' : 'Connected';

          // Click handlers
          agentBar.querySelectorAll('.agent-chip').forEach(chip => {
            chip.addEventListener('click', () => {
              agentBar.querySelectorAll('.agent-chip').forEach(c => c.classList.remove('active'));
              chip.classList.add('active');
              selectedAgent = chip.dataset.agent;
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
            streamBubble.textContent = msg.text;
            scrollToBottom();
          }
          break;
        }

        case 'streamEnd': {
          if (streamBubble) {
            streamBubble.textContent = msg.text;
          }
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
          const bubble = addMessage('assistant', msg.text);
          bubble.style.borderColor = 'var(--cto-red)';
          isStreaming = false;
          sendBtn.disabled = false;
          inputEl.focus();
          break;
        }

        case 'newChat': {
          messagesEl.innerHTML = '';
          if (welcomeEl) {
            messagesEl.appendChild(welcomeEl);
            welcomeEl.style.display = 'flex';
          }
          messages = [];
          break;
        }

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
