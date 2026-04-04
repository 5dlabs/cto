/**
 * Upload a pitch-deck PPTX to the signed-in user's Google Drive (drive.file scope).
 * Used from the static-export pitch page with GIS OAuth in the browser.
 */

export type DriveUploadResult = { id: string; name: string; webViewLink?: string };

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
          }) => { requestAccessToken: (overrideConfig?: { prompt?: string }) => void };
        };
      };
    };
  }
}

export function loadGoogleIdentityScript(): Promise<void> {
  const src = "https://accounts.google.com/gsi/client";
  if (document.querySelector(`script[src="${src}"]`)) {
    return Promise.resolve();
  }
  return new Promise((resolve, reject) => {
    const s = document.createElement("script");
    s.src = src;
    s.async = true;
    s.defer = true;
    s.onload = () => resolve();
    s.onerror = () => reject(new Error("Could not load Google sign-in"));
    document.head.appendChild(s);
  });
}

/** Request an OAuth access token with drive.file scope (user picks account in popup). */
export function requestDriveFileAccessToken(
  clientId: string,
  onDone: (token: string | null, error?: string) => void,
): void {
  const g = window.google;
  if (!g?.accounts?.oauth2) {
    onDone(null, "Google sign-in not ready");
    return;
  }
  const client = g.accounts.oauth2.initTokenClient({
    client_id: clientId,
    scope: "https://www.googleapis.com/auth/drive.file",
    callback: (resp) => {
      if (resp.error) {
        onDone(
          null,
          resp.error_description || resp.error || "Google sign-in failed",
        );
        return;
      }
      if (resp.access_token) {
        onDone(resp.access_token);
        return;
      }
      onDone(null, "No access token returned");
    },
  });
  client.requestAccessToken({ prompt: "consent" });
}

export async function uploadPptxBlobToDrive(
  accessToken: string,
  fileBlob: Blob,
  options?: { folderId?: string; fileName?: string },
): Promise<DriveUploadResult> {
  const fileName = options?.fileName ?? "5D-Labs-Pitch-Deck.pptx";
  const boundary = `5d_pitch_${Date.now()}_${Math.random().toString(36).slice(2)}`;

  const metadata: Record<string, unknown> = {
    name: fileName,
    mimeType:
      "application/vnd.openxmlformats-officedocument.presentationml.presentation",
  };
  if (options?.folderId) {
    metadata.parents = [options.folderId];
  }

  const encoder = new TextEncoder();
  const metaChunk = encoder.encode(
    `--${boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n${JSON.stringify(metadata)}\r\n`,
  );
  const fileHeader = encoder.encode(
    `--${boundary}\r\nContent-Type: application/vnd.openxmlformats-officedocument.presentationml.presentation\r\n\r\n`,
  );
  const pptxBytes = new Uint8Array(await fileBlob.arrayBuffer());
  const endChunk = encoder.encode(`\r\n--${boundary}--`);

  const body = new Uint8Array(
    metaChunk.length + fileHeader.length + pptxBytes.length + endChunk.length,
  );
  body.set(metaChunk, 0);
  body.set(fileHeader, metaChunk.length);
  body.set(pptxBytes, metaChunk.length + fileHeader.length);
  body.set(endChunk, metaChunk.length + fileHeader.length + pptxBytes.length);

  const q = new URLSearchParams({
    uploadType: "multipart",
    fields: "id,name,webViewLink",
  });

  const res = await fetch(
    `https://www.googleapis.com/upload/drive/v3/files?${q.toString()}`,
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${accessToken}`,
        "Content-Type": `multipart/related; boundary=${boundary}`,
      },
      body,
    },
  );

  if (!res.ok) {
    const errText = await res.text();
    throw new Error(`Drive upload failed (${res.status}): ${errText.slice(0, 500)}`);
  }

  return (await res.json()) as DriveUploadResult;
}
