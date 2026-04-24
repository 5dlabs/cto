import { existsSync } from "node:fs";
import { mkdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { invalidateListCache, validateSlug } from "./projects";
import { CONFIG } from "./config";
import { commitAll, pushCurrentBranch } from "./git";

export interface WritePrdResult {
  project: string;
  path: string;
  bytesWritten: number;
}

/**
 * Write (or overwrite) the PRD for a project, commit as Morgan, and push
 * to origin. GitHub is the authoritative source for project discovery,
 * so a write that doesn't make it to GitHub isn't a write we want to
 * acknowledge.
 *
 * Path precedence (first match wins, canonical last if no legacy exists):
 *   1. `.plan/prd/prd.md` — new canonical under the `.plan/` sidecar.
 *   2. `.prd/prd.md` / `.prd/PRD.md` / `.PRD/PRD.md` — legacy fallbacks,
 *      rewritten in place to avoid duplicate markers.
 *   3. Fresh projects that match none of the above get `.plan/prd/prd.md`.
 */
export async function writePrd(
  name: string,
  content: string,
): Promise<WritePrdResult> {
  validateSlug(name);
  const path = `${CONFIG.reposRoot}/${name}`;
  if (!existsSync(path)) {
    throw Object.assign(new Error(`project "${name}" not found`), {
      status: 404,
    });
  }

  // Prefer existing marker locations to avoid duplicate PRDs; otherwise
  // write to the new canonical `.plan/prd/prd.md` path.
  const canonical = join(path, ".plan", "prd", "prd.md");
  const legacyCandidates = [
    canonical,
    join(path, ".prd", "prd.md"),
    join(path, ".prd", "PRD.md"),
    join(path, ".PRD", "PRD.md"),
  ];
  const prdPath = legacyCandidates.find((p) => existsSync(p)) ?? canonical;
  await mkdir(join(prdPath, ".."), { recursive: true });
  const normalized = content.endsWith("\n") ? content : content + "\n";
  const bytes = await Bun.write(prdPath, normalized);

  await commitAll(path, "docs: update .plan/prd/prd.md");
  await pushCurrentBranch(path);

  // A PRD write can turn a "no-PRD" repo into a project — invalidate the
  // discovery cache so the next list call picks it up.
  invalidateListCache();

  return { project: name, path: prdPath, bytesWritten: bytes };
}

/**
 * Write `.plan/status.txt` under `<projectPath>` with the DO-NOT-EDIT
 * header and `phase: <phase>` / `updated: <iso>` lines. Idempotent: if
 * the on-disk content already matches the serialized form modulo the
 * `updated:` timestamp, the write is skipped and this returns `false`
 * so the caller can avoid an empty commit.
 *
 * The phase vocabulary is `new → intake → ready → implementing → complete`.
 * Callers outside that vocabulary still write — Morgan's skill markdown
 * owns the state machine; this helper only serializes.
 */
export async function writeStatus(
  projectPath: string,
  phase: string,
): Promise<boolean> {
  const dir = join(projectPath, ".plan");
  const statusPath = join(dir, "status.txt");
  const body =
    `# DO NOT EDIT — managed by Morgan.\n` +
    `# This file is overwritten automatically as the project moves through phases.\n` +
    `phase: ${phase}\n`;
  // If the file already pins the same phase, no-op so we don't churn the
  // git history with every call. We intentionally compare `phase:` only
  // (ignoring `updated:`) to avoid rewriting on each tick.
  if (existsSync(statusPath)) {
    try {
      const existing = await readFile(statusPath, "utf8");
      const phaseLine = existing
        .split(/\r?\n/)
        .map((l) => l.trim())
        .find((l) => l.toLowerCase().startsWith("phase:"));
      if (phaseLine) {
        const current = phaseLine.slice(phaseLine.indexOf(":") + 1).trim();
        if (current === phase) return false;
      }
    } catch {
      // Unreadable existing file → fall through and rewrite.
    }
  }
  const updated = new Date().toISOString();
  const content = body + `updated: ${updated}\n`;
  await mkdir(dir, { recursive: true });
  await Bun.write(statusPath, content);
  return true;
}
