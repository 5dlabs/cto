import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";
import { invalidateListCache, validateSlug } from "./projects";
import { CONFIG } from "./config";
import { commitAll, pushCurrentBranch } from "./git";

export interface WriteArchitectureResult {
  project: string;
  path: string;
  bytesWritten: number;
}

/**
 * Write (or overwrite) `.prd/architecture.md` for a project, commit as
 * Morgan, and push to origin. Parallel to `writePrd` but for the
 * Sigma-Long architecture sibling doc. We invalidate the list cache so
 * the `hasArchitecture` flag flips promptly in the UI.
 */
export async function writeArchitecture(
  name: string,
  content: string,
): Promise<WriteArchitectureResult> {
  validateSlug(name);
  const path = `${CONFIG.reposRoot}/${name}`;
  if (!existsSync(path)) {
    throw Object.assign(new Error(`project "${name}" not found`), {
      status: 404,
    });
  }

  const prdDir = join(path, ".prd");
  await mkdir(prdDir, { recursive: true });
  const archPath = join(prdDir, "architecture.md");
  const normalized = content.endsWith("\n") ? content : content + "\n";
  const bytes = await Bun.write(archPath, normalized);

  await commitAll(path, "docs: update .prd/architecture.md");
  await pushCurrentBranch(path);

  invalidateListCache();

  return { project: name, path: archPath, bytesWritten: bytes };
}
