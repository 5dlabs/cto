import { CONFIG } from "./config";

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
 */
export function authenticatedCloneUrl(cloneUrl: string): string {
  if (!CONFIG.githubToken) return cloneUrl;
  try {
    const u = new URL(cloneUrl);
    // GitHub supports x-access-token PATs and classic bearer tokens equally
    // over HTTPS basic auth; username is ignored but must be non-empty.
    u.username = "x-access-token";
    u.password = CONFIG.githubToken;
    return u.toString();
  } catch {
    return cloneUrl;
  }
}
