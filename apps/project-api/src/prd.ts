import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
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
 * Write (or overwrite) `.PRD/PRD.md` for a project, commit as Morgan, and
 * push to origin. GitHub is the authoritative source for project
 * discovery, so a write that doesn't make it to GitHub isn't a write we
 * want to acknowledge.
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

  const prdDir = join(path, ".PRD");
  await mkdir(prdDir, { recursive: true });
  const prdPath = join(prdDir, "PRD.md");
  const normalized = content.endsWith("\n") ? content : content + "\n";
  const bytes = await Bun.write(prdPath, normalized);

  await commitAll(path, "docs: update .PRD/PRD.md");
  await pushCurrentBranch(path);

  // A PRD write can turn a "no-PRD" repo into a project — invalidate the
  // discovery cache so the next list call picks it up.
  invalidateListCache();

  return { project: name, path: prdPath, bytesWritten: bytes };
}
