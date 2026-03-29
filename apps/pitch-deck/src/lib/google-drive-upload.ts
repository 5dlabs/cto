/**
 * Browser-only: Google Identity Services + Drive API (files created by this app only).
 * Requires NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID and Drive API enabled in Google Cloud.
 */

const DRIVE_FILE_SCOPE = "https://www.googleapis.com/auth/drive.file";

declare global {
  interface Window {
    google?: {
      accounts: {
        oauth2: {
          initTokenClient: (config: {
            client_id: string;
            scope: string;
            callback: (resp: {
              access_token?: string;
              error?: string;
              error_description?: string;
            }) => void;
          }) => { requestAccessToken: (override?: object) => void };
        };
      };
    };
  }
}

let gsiLoadPromise: Promise<void> | null = null;

export function loadGoogleIdentityServices(): Promise<void> {
  if (typeof window === "undefined") {
    return Promise.reject(new Error("Google sign-in runs in the browser only."));
  }
  if (window.google?.accounts?.oauth2) {
    return Promise.resolve();
  }
  if (gsiLoadPromise) return gsiLoadPromise;
  gsiLoadPromise = new Promise((resolve, reject) => {
    const existing = document.querySelector<HTMLScriptElement>(
      'script[src="https://accounts.google.com/gsi/client"]',
    );
    if (existing) {
      existing.addEventListener("load", () => resolve());
      existing.addEventListener("error", () =>
        reject(new Error("Failed to load Google Identity Services")),
      );
      return;
    }
    const s = document.createElement("script");
    s.src = "https://accounts.google.com/gsi/client";
    s.async = true;
    s.defer = true;
    s.onload = () => resolve();
    s.onerror = () => reject(new Error("Failed to load Google Identity Services"));
    document.head.appendChild(s);
  });
  return gsiLoadPromise;
}

/**
 * OAuth token with drive.file scope (user’s Drive — files this app creates).
 */
export function requestDriveAccessToken(clientId: string): Promise<string> {
  return loadGoogleIdentityServices().then(
    () =>
      new Promise((resolve, reject) => {
        const oauth2 = window.google?.accounts?.oauth2;
        if (!oauth2) {
          reject(new Error("Google Identity Services did not initialize."));
          return;
        }
        const client = oauth2.initTokenClient({
          client_id: clientId,
          scope: DRIVE_FILE_SCOPE,
          callback: (resp) => {
            if (resp.error) {
              reject(
                new Error(
                  resp.error_description?.trim() ||
                    resp.error ||
                    "Google sign-in was cancelled or failed.",
                ),
              );
              return;
            }
            if (resp.access_token) {
              resolve(resp.access_token);
              return;
            }
            reject(new Error("No access token returned."));
          },
        });
        client.requestAccessToken();
      }),
  );
}

export type DriveUploadResult = {
  id: string;
  name: string;
  webViewLink?: string;
};

/**
 * Multipart upload (metadata + media) — Drive API v3.
 */
export async function uploadBlobToDrive(
  blob: Blob,
  filename: string,
  mimeType: string,
  accessToken: string,
): Promise<DriveUploadResult> {
  const boundary = `5dlabsDrive${Date.now().toString(36)}`;
  const metadata = JSON.stringify({ name: filename, mimeType });
  const head = `--${boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n${metadata}\r\n`;
  const mid = `--${boundary}\r\nContent-Type: ${mimeType}\r\n\r\n`;
  const tail = `\r\n--${boundary}--`;
  const body = new Blob([head, mid, blob, tail]);

  const url = new URL("https://www.googleapis.com/upload/drive/v3/files");
  url.searchParams.set("uploadType", "multipart");
  url.searchParams.set("fields", "id,name,webViewLink");

  const res = await fetch(url.toString(), {
    method: "POST",
    headers: {
      Authorization: `Bearer ${accessToken}`,
      "Content-Type": `multipart/related; boundary=${boundary}`,
    },
    body,
  });

  const text = await res.text();
  if (!res.ok) {
    let msg = `Drive upload failed (${res.status})`;
    try {
      const j = JSON.parse(text) as { error?: { message?: string } };
      if (j.error?.message) msg = j.error.message;
    } catch {
      if (text) msg = text.slice(0, 200);
    }
    throw new Error(msg);
  }

  const data = JSON.parse(text) as DriveUploadResult;
  if (!data.id) {
    throw new Error("Drive returned no file id.");
  }
  return data;
}

export function getGoogleOAuthClientId(): string | undefined {
  const id = process.env.NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID;
  return typeof id === "string" && id.trim() !== "" ? id.trim() : undefined;
}
