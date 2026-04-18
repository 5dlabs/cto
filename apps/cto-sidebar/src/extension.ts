import * as vscode from "vscode";
import { SidebarProvider } from "./SidebarProvider";

const SETUP_KEY = "cto-sidebar.layoutConfigured.v2";

// View containers to hide from activity bar (keep Explorer + Search + CTO)
const HIDE_VIEW_CONTAINERS = [
  "workbench.view.scm",
  "workbench.view.debug",
  "workbench.view.extensions",
  "workbench.view.extension.github-copilot-chat",
  "workbench.view.remote",
];

export function activate(context: vscode.ExtensionContext) {
  const provider = new SidebarProvider(context.extensionUri, context);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider("cto-sidebar.chat", provider, {
      webviewOptions: { retainContextWhenHidden: true },
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("cto-sidebar.newChat", () => {
      provider.postMessage({ type: "newChat" });
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("cto-sidebar.clearHistory", () => {
      provider.postMessage({ type: "clearHistory" });
    })
  );

  // Reset layout command (for manual re-run)
  context.subscriptions.push(
    vscode.commands.registerCommand("cto-sidebar.resetLayout", async () => {
      await context.globalState.update(SETUP_KEY, false);
      await setupLayout(context);
    })
  );

  setupLayout(context);
}

async function setupLayout(context: vscode.ExtensionContext): Promise<void> {
  // Wait for VS Code to fully initialize
  await new Promise((r) => setTimeout(r, 2000));

  // Always focus CTO sidebar on startup
  try {
    await vscode.commands.executeCommand(
      "workbench.view.extension.cto-sidebar.focus"
    );
  } catch {
    // View may not be ready
  }

  // Close the Copilot auxiliary bar on the right
  try {
    await vscode.commands.executeCommand("workbench.action.closeAuxiliaryBar");
  } catch {
    // Not available in all versions
  }

  // Only hide views once
  const configured = context.globalState.get<boolean>(SETUP_KEY);
  if (configured) {
    return;
  }

  // Log available commands for debugging
  const allCommands = await vscode.commands.getCommands(true);
  const viewCommands = allCommands.filter(
    (c) =>
      c.includes("activitybar") ||
      c.includes("ActivityBar") ||
      c.includes("viewContainer")
  );
  console.log("[CTO] Available view commands:", viewCommands);

  // Try multiple approaches to hide unwanted view containers
  for (const viewId of HIDE_VIEW_CONTAINERS) {
    // Approach 1: Modern VS Code toggle command
    for (const cmd of [
      `${viewId}.toggleVisibility`,
      `workbench.action.toggleViewContainerVisibility.${viewId}`,
    ]) {
      try {
        await vscode.commands.executeCommand(cmd);
        console.log(`[CTO] Hidden ${viewId} via ${cmd}`);
        break;
      } catch {
        // Try next approach
      }
    }
  }

  // Move CTO sidebar to top of activity bar (order 0)
  // This is done by writing the pinnedViewlets to workspace storage
  try {
    const config = vscode.workspace.getConfiguration();
    // Some VS Code forks support this
    await config.update(
      "workbench.activityBar.pinnedViewContainers",
      ["workbench.view.extension.cto-sidebar", "workbench.view.search", "workbench.view.explorer"],
      vscode.ConfigurationTarget.Global
    );
  } catch {
    // Setting may not exist
  }

  await context.globalState.update(SETUP_KEY, true);
  console.log("[CTO] Layout setup complete");
}

export function deactivate() {}
