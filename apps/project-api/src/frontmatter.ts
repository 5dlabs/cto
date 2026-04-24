/**
 * Minimal YAML frontmatter parser, scoped to what Morgan writes in
 * `.prd/PRD.md` and `.prd/architecture.md`:
 *
 *   ---
 *   project: <slug>
 *   status: drafting | ready
 *   updated: 2025-01-15T12:34:56Z
 *   ---
 *
 * We intentionally do NOT pull a full YAML dependency. Morgan-authored
 * frontmatter is flat key/value with optional quoting; anything more
 * elaborate gets ignored rather than throwing (we'd rather surface a
 * project with `state: "drafting"` than refuse to list it).
 */

export interface Frontmatter {
  /** Raw frontmatter block (without the `---` fences), or null if absent. */
  raw: string | null;
  /** Markdown body with frontmatter stripped. */
  body: string;
  /** Parsed flat key/value fields, lowercased keys, string values. */
  fields: Record<string, string>;
}

const FENCE_RE = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?/;

export function parseFrontmatter(source: string): Frontmatter {
  const match = FENCE_RE.exec(source);
  if (!match) {
    return { raw: null, body: source, fields: {} };
  }
  const raw = match[1] ?? "";
  const body = source.slice(match[0].length);
  return { raw, body, fields: parseFields(raw) };
}

function parseFields(raw: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const line of raw.split(/\r?\n/)) {
    // Skip blank lines and YAML comments.
    if (!line.trim() || line.trim().startsWith("#")) continue;
    const m = /^\s*([A-Za-z_][A-Za-z0-9_-]*)\s*:\s*(.*)$/.exec(line);
    if (!m) continue;
    const key = (m[1] ?? "").toLowerCase();
    out[key] = unquote((m[2] ?? "").trim());
  }
  return out;
}

function unquote(v: string): string {
  if (v.length >= 2) {
    const first = v[0];
    const last = v[v.length - 1];
    if ((first === '"' && last === '"') || (first === "'" && last === "'")) {
      return v.slice(1, -1);
    }
  }
  return v;
}

export type PrdStatus = "drafting" | "ready";

/**
 * Coerce a raw `status:` value into the canonical state field exposed to
 * the UI. Anything we don't recognize falls back to "drafting" — the tile
 * should light up as in-progress rather than silently disappear.
 */
export function coerceStatus(raw: string | undefined | null): PrdStatus {
  const v = (raw ?? "").toLowerCase().trim();
  if (v === "ready") return "ready";
  return "drafting";
}
