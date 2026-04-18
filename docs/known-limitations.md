# Known Limitations

Tracks platform limitations we've investigated and cannot fully work around with
the current toolchain. Each entry links to a concrete PR/issue so we don't
re-litigate the same ground.

---

## Activity bar icon hiding in code-server 4.116 (Parity Stream D)

### Goal

Render the code-server activity bar with **only** three icons, in this order:

1. 5dlabs CTO sidebar (`cto-sidebar`)
2. Explorer (`workbench.view.explorer`)
3. Search (`workbench.view.search`)

Hide the rest: SCM, Run and Debug, Extensions, Remote Explorer, and the GitHub
Copilot Chat container injected by the bundled extension.

### Outcome

**No reliable programmatic approach exists in code-server 4.116 / VS Code
1.116.** The `cto-sidebar` extension attempts four command variants as a
best-effort hook for future releases, but all currently silently no-op. A
manual workaround is available to end users (see below).

### Approaches evaluated

| # | Approach | Result | Notes |
|---|----------|--------|-------|
| 1 | `workbench.activityBar.visible` setting | ❌ Removed upstream | Deleted from VS Code 1.74+. Rejected because it also hid the entire bar. |
| 2 | `workbench.activityBar.pinnedViewContainers` setting | ❌ No-op | Fork-specific. Writes succeed but are ignored on stock VS Code / code-server. |
| 3 | `workbench.action.toggleActivityBarVisibility` | ⚠️ Too aggressive | Hides the entire activity bar, including the CTO sidebar icon. |
| 4 | `<viewId>.toggleVisibility` command | ❌ Command not found | Does not exist in 1.116. |
| 5 | `workbench.action.toggleViewContainerVisibility.<viewId>` | ❌ Command not found | Does not exist in 1.116. |
| 6 | `workbench.action.hideViewContainer.<viewId>` | ❌ Command not found | Does not exist in 1.116. |
| 7 | `<viewId>.removeView` command | ❌ Command not found | Does not exist in 1.116. |
| 8 | `contributes.viewsContainers` with `when` clause | ❌ Ignored | `when` only applies to `views` and menu items, not to container declarations themselves. |
| 9 | Pre-seed `storage.json` activity-bar keys | ❌ Overwritten at startup | VS Code rewrites bundled view container state from built-in defaults on launch. |
| 10 | Webview `<style>` CSS injection | ❌ Sandboxed | Webviews run in iframes that cannot reach the workbench DOM. |
| 11 | `vscode-custom-css` (be5invis) — patches `workbench.html` | ❌ Rejected | Triggers "your VS Code installation appears to be corrupt" banner, needs write access to the install root, and breaks on every code-server upgrade. Unsuitable for a managed deployment. |
| 12 | Forking / patching `code-server` workbench | ❌ Out of scope | Maintenance cost far exceeds value; locks us out of upstream security updates. |

### Current workaround (shipping)

The `cto-sidebar` extension's `setupLayout()` routine iterates approaches 4–7
above on first activation, gated by the `cto-sidebar.layoutConfigured.v2`
global state key. All four commands currently throw `command not found` in
1.116, but the loop is left in place as a forward-compatible stub so the
extension transparently benefits if any future code-server release ships
these commands.

`setupLayout()` also attempts approach 2 (the fork-specific pinned containers
setting), which likewise silently no-ops on stock VS Code.

### Manual workaround for end users

Users who want a cleaner activity bar today can right-click any unwanted icon
in the activity bar and choose **Hide from Activity Bar**. This preference is
persisted per user profile and survives code-server restarts.

### Unblockers we're watching

- Upstream VS Code issue to expose a public API for hiding built-in view
  containers (search `github.com/microsoft/vscode` for
  `hideViewContainer` / `activitybar pinned`).
- code-server release notes for new `workbench.action.*ViewContainer*`
  commands.
- Any future first-class "customize activity bar" setting similar to the
  existing macOS/Windows customization model.

When any of the above land, revisit `apps/cto-sidebar/src/extension.ts` and
remove this workaround entry.

### Related

- Extension code: [`apps/cto-sidebar/src/extension.ts`](../apps/cto-sidebar/src/extension.ts)
- Parity swarm: Stream D — activity bar cleanup
