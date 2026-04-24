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
 * Write (or overwrite) `.prd/prd.md` for a project, commit as Morgan, and
 * push to origin. GitHub is the authoritative source for project
 * discovery, so a write that doesn't make it to GitHub isn't a write we
 * want to acknowledge.
 *
 * If a legacy uppercase marker exists in the repo (`.prd/PRD.md` or
 * `.PRD/PRD.md`), we rewrite into that same path to avoid ending up with
 * two parallel markers. Fresh projects get the canonical lowercase path.
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

  // Prefer existing legacy locations to avoid duplicate markers; otherwise
  // write to the canonical lowercase path.
  const legacyCandidates = [
    join(path, ".prd", "PRD.md"),
    join(path, ".PRD", "PRD.md"),
  ];
  const existingLegacy = legacyCandidates.find((p) => existsSync(p));
  const prdPath = existingLegacy ?? join(path, ".prd", "prd.md");
  await mkdir(join(path, ".prd"), { recursive: true });
  const normalized = content.endsWith("\n") ? content : content + "\n";
  const bytes = await Bun.write(prdPath, normalized);

  await commitAll(path, "docs: update .prd/prd.md");
  await pushCurrentBranch(path);

  // A PRD write can turn a "no-PRD" repo into a project — invalidate the
  // discovery cache so the next list call picks it up.
  invalidateListCache();

  return { project: name, path: prdPath, bytesWritten: bytes };
}
