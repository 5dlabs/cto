import { readFile } from "fs/promises";
import { resolve } from "path";
import { NextRequest, NextResponse } from "next/server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const MORGAN_IMAGE = resolve(process.cwd(), "../morgan.jpg");
const DOG_PROMPT =
  "A golden retriever dog wearing a suit is talking. Keep the face and muzzle clearly canine, preserve the dog identity, natural dog mouth motion, no human face.";

function appendOptional(form: FormData, key: string, value: FormDataEntryValue | null) {
  if (typeof value === "string" && value.trim()) {
    form.set(key, value.trim());
  }
}

export async function POST(request: NextRequest) {
  const appUrl = process.env.ECHOMIMIC_APP_URL?.trim();
  if (!appUrl) {
    return NextResponse.json(
      { error: "Server misconfigured. Set ECHOMIMIC_APP_URL to the EchoMimic app URL." },
      { status: 500 },
    );
  }

  const requestForm = await request.formData();
  const audio = requestForm.get("audio");
  if (!(audio instanceof File)) {
    return NextResponse.json({ error: "audio file is required" }, { status: 400 });
  }

  const sourceBytes = await readFile(MORGAN_IMAGE);
  const form = new FormData();
  form.set(
    "source",
    new Blob([new Uint8Array(sourceBytes)], { type: "image/jpeg" }),
    "morgan.jpg",
  );
  form.set("audio", audio, audio.name || "turn.mp3");
  form.set("prompt", requestForm.get("prompt")?.toString().trim() || DOG_PROMPT);
  appendOptional(form, "video_length", requestForm.get("video_length"));
  appendOptional(form, "sample_height", requestForm.get("sample_height"));
  appendOptional(form, "sample_width", requestForm.get("sample_width"));
  appendOptional(form, "weight_dtype", requestForm.get("weight_dtype"));

  const response = await fetch(`${appUrl.replace(/\/$/, "")}/animate`, {
    method: "POST",
    body: form,
    signal: AbortSignal.timeout(900_000),
  });

  if (!response.ok || !response.body) {
    const detail = await response.text().catch(() => "");
    return NextResponse.json(
      { error: "EchoMimic render failed", status: response.status, detail: detail.slice(0, 2000) },
      { status: 502 },
    );
  }

  return new Response(response.body, {
    headers: {
      "Cache-Control": "no-store",
      "Content-Disposition": 'inline; filename="morgan-turn.mp4"',
      "Content-Type": response.headers.get("content-type") || "video/mp4",
      "X-EchoMimic-Elapsed-S": response.headers.get("x-echomimic-elapsed-s") || "",
      "X-EchoMimic-Job-Id": response.headers.get("x-echomimic-job-id") || "",
    },
  });
}
