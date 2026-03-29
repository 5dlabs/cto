import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Load markdown from `docs/legal/` at build time (static export).
 */
export function loadLegalMarkdown(name: "privacy-policy.md" | "terms-of-service.md"): string {
  return readFileSync(join(process.cwd(), "docs/legal", name), "utf8");
}
