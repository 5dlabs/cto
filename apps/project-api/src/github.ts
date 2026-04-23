import { CONFIG } from "./config";

export interface RepoInfo {
  exists: boolean;
  cloneUrl: string | null;
  defaultBranch: string | null;
  private: boolean | null;
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
  const url = `https://api.github.com/repos/${encodeURIComponent(
    org,
  )}/${encodeURIComponent(name)}`;
  const headers: Record<string, string> = {
    accept: "application/vnd.github+json",
    "user-agent": "morgan-project-api/0.1",
    "x-github-api-version": "2022-11-28",
  };
  if (CONFIG.githubToken) headers.authorization = `Bearer ${CONFIG.githubToken}`;

  const res = await fetch(url, { headers });
  if (res.status === 404) {
    return { exists: false, cloneUrl: null, defaultBranch: null, private: null };
  }
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(
      `github repo lookup failed: ${res.status} ${res.statusText}${body ? ` — ${body.slice(0, 240)}` : ""}`,
    );
  }
  const data = (await res.json()) as {
    clone_url?: string;
    default_branch?: string;
    private?: boolean;
  };
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
