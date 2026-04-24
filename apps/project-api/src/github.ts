import { CONFIG } from "./config";
import { parseFrontmatter } from "./frontmatter";

export interface RepoInfo {
  exists: boolean;
  cloneUrl: string | null;
  defaultBranch: string | null;
  private: boolean | null;
}

async function githubJson<T>(
  method: "GET" | "POST",
  path: string,
  body?: unknown,
): Promise<{ status: number; data: T | null; text: string }> {
  const headers: Record<string, string> = {
    accept: "application/vnd.github+json",
    "user-agent": "morgan-project-api/0.1",
    "x-github-api-version": "2022-11-28",
  };
  if (CONFIG.githubToken) headers.authorization = `Bearer ${CONFIG.githubToken}`;
  if (body !== undefined) headers["content-type"] = "application/json";

  const res = await fetch(`https://api.github.com${path}`, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text().catch(() => "");
  let data: T | null = null;
  try {
    data = text ? (JSON.parse(text) as T) : null;
  } catch {
    data = null;
  }
  return { status: res.status, data, text };
}

/**
 * Ask GitHub whether `<org>/<name>` exists. Uses the configured token when
 * available (required for private repos and higher rate limits). Returns a
 * shallow descriptor that the caller can use to decide clone-vs-init.
 */
export async function lookupRepo(
  org: string,
  name: string,
): Promise<RepoInfo> {
  const result = await githubJson<{
    clone_url?: string;
    default_branch?: string;
    private?: boolean;
  }>("GET", `/repos/${encodeURIComponent(org)}/${encodeURIComponent(name)}`);
  if (result.status === 404) {
    return { exists: false, cloneUrl: null, defaultBranch: null, private: null };
  }
  if (result.status < 200 || result.status >= 300) {
    throw new Error(
      `github repo lookup failed: ${result.status}${result.text ? ` — ${result.text.slice(0, 240)}` : ""}`,
    );
  }
  const data = result.data ?? {};
  return {
    exists: true,
    cloneUrl: data.clone_url ?? null,
    defaultBranch: data.default_branch ?? null,
    private: data.private ?? null,
  };
}

/**
 * Create a repository in `<org>` and return its clone metadata.
 */
export async function createRepo(org: string, name: string): Promise<RepoInfo> {
  if (!CONFIG.githubToken) {
    throw new Error("github token missing; cannot create remote repository");
  }
  const result = await githubJson<{
    clone_url?: string;
    default_branch?: string;
    private?: boolean;
    message?: string;
  }>("POST", `/orgs/${encodeURIComponent(org)}/repos`, {
    name,
    private: true,
    auto_init: true,
  });
  if (result.status === 422) {
    // Already exists or name conflict; re-check and let caller continue.
    return lookupRepo(org, name);
  }
  if (result.status < 200 || result.status >= 300) {
    const msg = result.data?.message || result.text || "unknown github error";
    throw new Error(`github repo create failed: ${result.status} — ${msg}`);
  }
  const data = result.data ?? {};
  return {
    exists: true,
    cloneUrl: data.clone_url ?? null,
    defaultBranch: data.default_branch ?? null,
    private: data.private ?? null,
  };
}

/**
 * Build a clone URL that embeds the configured token when present, so
 * `git clone` can auth non-interactively inside the pod. Falls back to the
 * plain clone URL for public repos when no token is configured.
 *
 * TODO(security): move to a git credential helper so the token is never
 * persisted in `.git/config` on the PVC. Tracked in the branch scope.
 */
export function authenticatedCloneUrl(cloneUrl: string): string {
  if (!CONFIG.githubToken) return cloneUrl;
  try {
    const u = new URL(cloneUrl);
    u.username = "x-access-token";
    u.password = CONFIG.githubToken;
    return u.toString();
  } catch {
    return cloneUrl;
  }
}

/**
 * Paths we try (in order) when probing for a PRD marker. The lowercase
 * `.prd/` form is canonical after the PR #4820 rename; `.PRD/` is a
 * transitional fallback so repos created before the rename don't silently
 * drop out of discovery. TODO: remove the uppercase fallback once every
 * active repo in the org has been migrated.
 */
const PRD_MARKER_PATHS = [".prd/PRD.md", ".PRD/PRD.md"] as const;
const ARCHITECTURE_MARKER_PATHS = [
  ".prd/architecture.md",
  ".PRD/architecture.md",
] as const;

/**
 * Check whether `<org>/<name>` contains a PRD marker at the default branch.
 * Accepts the canonical `.prd/PRD.md` or the legacy uppercase `.PRD/PRD.md`.
 * Returns `false` for 404 / any non-2xx — discovery treats "not sure" as
 * "exclude from list".
 */
export async function hasPrdMarker(
  org: string,
  name: string,
): Promise<boolean> {
  for (const path of PRD_MARKER_PATHS) {
    const result = await githubJson<{ type?: string; size?: number }>(
      "GET",
      `/repos/${encodeURIComponent(org)}/${encodeURIComponent(name)}/contents/${path}`,
    );
    if (result.status >= 200 && result.status < 300) return true;
  }
  return false;
}

/**
 * Check whether `<org>/<name>` contains an architecture.md marker at the
 * default branch. Accepts the legacy uppercase path too. Used by discovery
 * to set `hasArchitecture` on the `ProjectDescriptor` so the UI can
 * distinguish "PRD-only" from "intake-ready".
 */
export async function hasArchitectureMarker(
  org: string,
  name: string,
): Promise<boolean> {
  for (const path of ARCHITECTURE_MARKER_PATHS) {
    const result = await githubJson<{ type?: string; size?: number }>(
      "GET",
      `/repos/${encodeURIComponent(org)}/${encodeURIComponent(name)}/contents/${path}`,
    );
    if (result.status >= 200 && result.status < 300) return true;
  }
  return false;
}

export interface PrdInfo {
  exists: boolean;
  /** Decoded file content, when available. */
  content: string | null;
  /** Parsed frontmatter keys (lowercased). */
  fields: Record<string, string>;
}

/**
 * Fetch the PRD marker from `<org>/<name>` and parse its YAML frontmatter.
 * Tries the canonical `.prd/PRD.md` first, then falls back to the legacy
 * `.PRD/PRD.md` path for repos that predate PR #4820. Without the fallback
 * every pre-rename repo silently disappears from the Projects list.
 *
 * Returns `{exists:false}` when neither path resolves — discovery drops the
 * tile from the list in that case.
 */
export async function fetchPrdInfo(
  org: string,
  name: string,
): Promise<PrdInfo> {
  for (const path of PRD_MARKER_PATHS) {
    const result = await githubJson<{ content?: string; encoding?: string }>(
      "GET",
      `/repos/${encodeURIComponent(org)}/${encodeURIComponent(name)}/contents/${path}`,
    );
    if (result.status < 200 || result.status >= 300 || !result.data) continue;

    let content: string | null = null;
    if (
      result.data.encoding === "base64" &&
      typeof result.data.content === "string"
    ) {
      try {
        // GitHub's base64 blobs arrive wrapped every 60 chars; atob tolerates
        // whitespace in modern runtimes (Bun does).
        content = atob(result.data.content.replace(/\n/g, ""));
      } catch {
        content = null;
      }
    }

    const fm = content ? parseFrontmatter(content) : { fields: {} };
    return { exists: true, content, fields: fm.fields };
  }
  return { exists: false, content: null, fields: {} };
}

export interface OrgRepoSummary {
  name: string;
  cloneUrl: string | null;
  defaultBranch: string | null;
  updatedAt: string | null;
  description: string | null;
  archived: boolean;
  private: boolean;
}

/**
 * Enumerate every repo in `<org>` across all pages (up to a sane safety cap).
 * Uses `type=all` to include forks and private repos when the token has
 * access; filters out archived repos, which shouldn't surface in project
 * discovery.
 */
export async function listOrgRepos(org: string): Promise<OrgRepoSummary[]> {
  const perPage = 100;
  const maxPages = 20; // hard cap: 2000 repos is already a lot for this use-case
  const out: OrgRepoSummary[] = [];

  for (let page = 1; page <= maxPages; page++) {
    const result = await githubJson<
      Array<{
        name?: string;
        clone_url?: string;
        default_branch?: string;
        updated_at?: string;
        pushed_at?: string;
        description?: string | null;
        archived?: boolean;
        private?: boolean;
      }>
    >(
      "GET",
      `/orgs/${encodeURIComponent(org)}/repos?per_page=${perPage}&page=${page}&type=all&sort=pushed`,
    );
    if (result.status < 200 || result.status >= 300) {
      throw new Error(
        `github list org repos failed: ${result.status}${result.text ? ` — ${result.text.slice(0, 240)}` : ""}`,
      );
    }
    const rows = result.data ?? [];
    for (const row of rows) {
      if (!row.name) continue;
      if (row.archived) continue;
      out.push({
        name: row.name,
        cloneUrl: row.clone_url ?? null,
        defaultBranch: row.default_branch ?? null,
        updatedAt: row.pushed_at ?? row.updated_at ?? null,
        description: row.description ?? null,
        archived: Boolean(row.archived),
        private: Boolean(row.private),
      });
    }
    if (rows.length < perPage) break;
  }
  return out;
}
