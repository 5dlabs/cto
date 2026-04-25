import { randomUUID } from "crypto";
import { readFile } from "fs/promises";
import { resolve } from "path";
import { NextRequest, NextResponse } from "next/server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const MORGAN_IMAGE = resolve(process.cwd(), "../morgan.jpg");
const DOG_PROMPT =
  "A golden retriever dog wearing a suit is talking. Keep the face and muzzle clearly canine, preserve the dog identity, natural dog mouth motion, no human face.";

const JOB_TTL_MS = 30 * 60 * 1000;
const RENDER_TIMEOUT_MS = 15 * 60 * 1000;
const MAX_JOBS = 64;

const NO_STORE_HEADERS = {
  "Cache-Control": "no-store, no-transform",
  "Content-Type": "application/json; charset=utf-8",
} as const;

function jsonResponse(body: unknown, init: { status: number }) {
  return new NextResponse(JSON.stringify(body), {
    status: init.status,
    headers: NO_STORE_HEADERS,
  });
}

type JobState =
  | { status: "queued"; createdAt: number }
  | { status: "running"; createdAt: number; startedAt: number }
  | {
      status: "done";
      createdAt: number;
      finishedAt: number;
      buffer: ArrayBuffer;
      contentType: string;
      elapsedSeconds: string;
      upstreamJobId: string;
    }
  | {
      status: "error";
      createdAt: number;
      finishedAt: number;
      detail: string;
      upstreamStatus: number | null;
    };

type JobStore = Map<string, JobState>;

const globalForJobs = globalThis as unknown as { __echoTurnAvatarJobs?: JobStore };
const jobs: JobStore = globalForJobs.__echoTurnAvatarJobs ?? new Map<string, JobState>();
globalForJobs.__echoTurnAvatarJobs = jobs;

function pruneJobs() {
  const now = Date.now();
  for (const [id, job] of jobs) {
    const finishedAt =
      job.status === "done" || job.status === "error" ? job.finishedAt : null;
    if (finishedAt !== null && now - finishedAt > JOB_TTL_MS) {
      jobs.delete(id);
    } else if (now - job.createdAt > JOB_TTL_MS * 2) {
      jobs.delete(id);
    }
  }

  // Bounded retention: if still over cap, evict oldest finished first, then
  // oldest queued/running. Map iteration order is insertion order, so the
  // first entries are the oldest.
  if (jobs.size > MAX_JOBS) {
    const finished: string[] = [];
    const inflight: string[] = [];
    for (const [id, job] of jobs) {
      if (job.status === "done" || job.status === "error") {
        finished.push(id);
      } else {
        inflight.push(id);
      }
    }
    const order = [...finished, ...inflight];
    while (jobs.size > MAX_JOBS && order.length > 0) {
      const id = order.shift();
      if (id) jobs.delete(id);
    }
  }
}

function appendOptional(form: FormData, key: string, value: FormDataEntryValue | null) {
  if (typeof value === "string" && value.trim()) {
    form.set(key, value.trim());
  }
}

async function runRenderJob(jobId: string, appUrl: string, form: FormData) {
  const existing = jobs.get(jobId);
  const createdAt = existing?.createdAt ?? Date.now();
  jobs.set(jobId, { status: "running", createdAt, startedAt: Date.now() });

  try {
    const response = await fetch(`${appUrl.replace(/\/$/, "")}/animate`, {
      method: "POST",
      body: form,
      signal: AbortSignal.timeout(RENDER_TIMEOUT_MS),
    });

    if (!response.ok) {
      const detail = await response.text().catch(() => "");
      jobs.set(jobId, {
        status: "error",
        createdAt,
        finishedAt: Date.now(),
        detail: detail.slice(0, 2000) || `upstream returned ${response.status}`,
        upstreamStatus: response.status,
      });
      return;
    }

    const buffer = await response.arrayBuffer();
    jobs.set(jobId, {
      status: "done",
      createdAt,
      finishedAt: Date.now(),
      buffer,
      contentType: response.headers.get("content-type") || "video/mp4",
      elapsedSeconds: response.headers.get("x-echomimic-elapsed-s") || "",
      upstreamJobId: response.headers.get("x-echomimic-job-id") || "",
    });
  } catch (err) {
    jobs.set(jobId, {
      status: "error",
      createdAt,
      finishedAt: Date.now(),
      detail: err instanceof Error ? err.message : "EchoMimic render failed",
      upstreamStatus: null,
    });
  }
}

export async function POST(request: NextRequest) {
  pruneJobs();

  const appUrl = process.env.ECHOMIMIC_APP_URL?.trim();
  if (!appUrl) {
    return jsonResponse(
      { error: "Server misconfigured. Set ECHOMIMIC_APP_URL to the EchoMimic app URL." },
      { status: 500 },
    );
  }

  const requestForm = await request.formData();
  const audio = requestForm.get("audio");
  if (!(audio instanceof File)) {
    return jsonResponse({ error: "audio file is required" }, { status: 400 });
  }

  const audioBytes = new Uint8Array(await audio.arrayBuffer());
  const sourceBytes = await readFile(MORGAN_IMAGE);

  const form = new FormData();
  form.set(
    "source",
    new Blob([new Uint8Array(sourceBytes)], { type: "image/jpeg" }),
    "morgan.jpg",
  );
  form.set(
    "audio",
    new Blob([audioBytes], { type: audio.type || "audio/mpeg" }),
    audio.name || "turn.mp3",
  );
  form.set("prompt", requestForm.get("prompt")?.toString().trim() || DOG_PROMPT);
  appendOptional(form, "video_length", requestForm.get("video_length"));
  appendOptional(form, "sample_height", requestForm.get("sample_height"));
  appendOptional(form, "sample_width", requestForm.get("sample_width"));
  appendOptional(form, "weight_dtype", requestForm.get("weight_dtype"));

  const jobId = randomUUID();
  jobs.set(jobId, { status: "queued", createdAt: Date.now() });
  void runRenderJob(jobId, appUrl, form);

  return jsonResponse({ jobId, status: "queued" }, { status: 202 });
}

export async function GET(request: NextRequest) {
  pruneJobs();

  const jobId = request.nextUrl.searchParams.get("jobId");
  if (!jobId) {
    return jsonResponse({ error: "jobId query parameter is required" }, { status: 400 });
  }

  const job = jobs.get(jobId);
  if (!job) {
    return jsonResponse({ error: "unknown jobId", jobId }, { status: 404 });
  }

  if (job.status === "queued" || job.status === "running") {
    return jsonResponse({ jobId, status: job.status }, { status: 202 });
  }

  if (job.status === "error") {
    return jsonResponse(
      {
        error: "EchoMimic render failed",
        status: job.upstreamStatus,
        detail: job.detail,
        jobId,
      },
      { status: 502 },
    );
  }

  // ArrayBuffer is a valid BlobPart; this gives a single-copy stream-friendly body.
  const blob = new Blob([job.buffer], { type: job.contentType });
  return new Response(blob, {
    headers: {
      "Cache-Control": "no-store, no-transform",
      "Content-Disposition": 'inline; filename="morgan-turn.mp4"',
      "Content-Type": job.contentType,
      "Content-Length": String(job.buffer.byteLength),
      "X-EchoMimic-Elapsed-S": job.elapsedSeconds,
      "X-EchoMimic-Job-Id": job.upstreamJobId,
      "X-EchoMimic-Local-Job-Id": jobId,
    },
  });
}
