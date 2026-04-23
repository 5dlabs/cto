import { existsSync } from "node:fs";
import { join } from "node:path";
import { validateSlug } from "./projects";
import { CONFIG } from "./config";
import { commitAll } from "./git";

export interface WritePrdResult {
  project: string;
  path: string;
  bytesWritten: number;
}

/**
 * Write (or overwrite) `prd.md` at the root of a project and commit it as
 * Morgan. The file lives directly at the repo root to match the existing
 * convention used by the intake pipeline (tests/intake/<slug>/prd.md).
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

  const prdPath = join(path, "prd.md");
  const normalized = content.endsWith("\n") ? content : content + "\n";
  const bytes = await Bun.write(prdPath, normalized);

  try {
    await commitAll(path, "docs: write prd.md");
  } catch {
    // Non-fatal — the file is on disk; the human can commit manually.
  }

  return { project: name, path: prdPath, bytesWritten: bytes };
}
