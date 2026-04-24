"use client";

import { FormEvent, useRef, useState } from "react";
import Image from "next/image";

type Phase =
  | "idle"
  | "streaming"
  | "synthesizing"
  | "rendering"
  | "ready"
  | "error";

const DOG_PROMPT =
  "A golden retriever dog wearing a suit is talking. Keep the face and muzzle clearly canine, preserve the dog identity, natural dog mouth motion, no human face.";

async function readMorganStream(response: Response, onDelta: (text: string) => void) {
  if (!response.body) {
    throw new Error("Morgan stream did not include a response body.");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split(/\r?\n/);
    buffer = lines.pop() ?? "";

    for (const line of lines) {
      if (!line.startsWith("data:")) {
        continue;
      }
      const payload = line.slice(5).trim();
      if (!payload) {
        continue;
      }
      const event = JSON.parse(payload) as { type: string; text?: string; message?: string };
      if (event.type === "delta" && event.text) {
        onDelta(event.text);
      }
      if (event.type === "error") {
        throw new Error(event.message || "Morgan stream failed.");
      }
    }
  }
}

export default function EchoTurnPage() {
  const [message, setMessage] = useState(
    "Give me a short update on how the Morgan avatar test is going.",
  );
  const [reply, setReply] = useState("");
  const [phase, setPhase] = useState<Phase>("idle");
  const [error, setError] = useState("");
  const [audioUrl, setAudioUrl] = useState("");
  const [videoUrl, setVideoUrl] = useState("");
  const [metrics, setMetrics] = useState<string[]>([]);
  const objectUrls = useRef<string[]>([]);

  function rememberObjectUrl(url: string) {
    objectUrls.current.push(url);
    return url;
  }

  function cleanupObjectUrls() {
    for (const url of objectUrls.current) {
      URL.revokeObjectURL(url);
    }
    objectUrls.current = [];
  }

  async function runTurn(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    cleanupObjectUrls();
    setAudioUrl("");
    setVideoUrl("");
    setReply("");
    setError("");
    setMetrics([]);

    const startedAt = performance.now();
    try {
      setPhase("streaming");
      let finalReply = "";
      const chatResponse = await fetch("/api/echo-turn/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ message }),
      });
      if (!chatResponse.ok) {
        throw new Error(`Morgan stream returned ${chatResponse.status}`);
      }
      await readMorganStream(chatResponse, (chunk) => {
        finalReply += chunk;
        setReply((current) => current + chunk);
      });
      const streamMs = performance.now() - startedAt;

      setPhase("synthesizing");
      const ttsStartedAt = performance.now();
      const ttsResponse = await fetch("/api/echo-turn/tts", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text: finalReply }),
      });
      if (!ttsResponse.ok) {
        throw new Error(`Morgan TTS returned ${ttsResponse.status}`);
      }
      const audioBlob = await ttsResponse.blob();
      const audioObjectUrl = rememberObjectUrl(URL.createObjectURL(audioBlob));
      setAudioUrl(audioObjectUrl);
      const ttsMs = performance.now() - ttsStartedAt;

      setPhase("rendering");
      const renderStartedAt = performance.now();
      const form = new FormData();
      form.set("audio", audioBlob, "morgan-turn.mp3");
      form.set("prompt", DOG_PROMPT);
      const avatarResponse = await fetch("/api/echo-turn/avatar", {
        method: "POST",
        body: form,
      });
      if (!avatarResponse.ok) {
        const detail = await avatarResponse.text();
        throw new Error(`EchoMimic returned ${avatarResponse.status}: ${detail.slice(0, 240)}`);
      }
      const videoBlob = await avatarResponse.blob();
      const videoObjectUrl = rememberObjectUrl(URL.createObjectURL(videoBlob));
      setVideoUrl(videoObjectUrl);
      const renderMs = performance.now() - renderStartedAt;

      setMetrics([
        `Text stream: ${Math.round(streamMs)} ms`,
        `TTS audio: ${Math.round(ttsMs)} ms`,
        `EchoMimic MP4: ${Math.round(renderMs)} ms`,
        `Video bytes: ${(videoBlob.size / 1024).toFixed(1)} KiB`,
      ]);
      setPhase("ready");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Morgan turn failed.");
      setPhase("error");
    }
  }

  const busy = phase === "streaming" || phase === "synthesizing" || phase === "rendering";
  const avatarStatus =
    phase === "rendering"
      ? "Rendering the EchoMimic MP4. Morgan stays visible here while the batch job runs."
      : "Morgan's source image stays visible until the generated MP4 is ready.";

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,#164e63_0%,#020617_36%,#020617_100%)] px-6 py-10 text-white">
      <div className="mx-auto grid max-w-7xl gap-6 lg:grid-cols-[0.95fr_1.05fr]">
        <section className="rounded-[2rem] border border-white/10 bg-white/[0.06] p-6 shadow-2xl shadow-cyan-950/30 backdrop-blur">
          <p className="text-xs uppercase tracking-[0.3em] text-cyan-200/80">
            EchoMimic turn demo
          </p>
          <h1 className="mt-4 text-4xl font-semibold tracking-tight">
            Stream Morgan&apos;s words first, then render the dog-preserving avatar turn.
          </h1>
          <p className="mt-4 text-sm leading-6 text-slate-300">
            This is the fastest bridge from the current batch MP4 model to a conversational web
            experience. WebRTC/LiveKit can carry the live mic and audio loop; EchoMimic currently
            renders a complete MP4 after the turn finishes.
          </p>

          <form onSubmit={runTurn} className="mt-6 flex flex-col gap-4">
            <label className="text-sm font-medium text-slate-200" htmlFor="message">
              Your turn
            </label>
            <textarea
              id="message"
              className="min-h-32 rounded-2xl border border-white/10 bg-slate-950/80 p-4 text-sm leading-6 text-white outline-none ring-cyan-300/0 transition focus:border-cyan-300/40 focus:ring-4 focus:ring-cyan-300/10"
              value={message}
              onChange={(event) => setMessage(event.target.value)}
              disabled={busy}
            />
            <button
              className="rounded-full bg-cyan-300 px-5 py-3 text-sm font-semibold text-slate-950 transition hover:bg-cyan-200 disabled:cursor-not-allowed disabled:opacity-50"
              type="submit"
              disabled={busy}
            >
              {busy ? "Morgan is working..." : "Run one conversational turn"}
            </button>
          </form>

          <div className="mt-6 rounded-2xl border border-white/10 bg-slate-950/70 p-4">
            <p className="text-xs uppercase tracking-[0.24em] text-slate-400">Status</p>
            <p className="mt-2 text-sm text-cyan-100">{phase}</p>
            {metrics.length > 0 ? (
              <div className="mt-4 grid gap-2 text-sm text-slate-300 sm:grid-cols-2">
                {metrics.map((metric) => (
                  <span key={metric} className="rounded-xl bg-white/5 px-3 py-2">
                    {metric}
                  </span>
                ))}
              </div>
            ) : null}
            {error ? <p className="mt-4 text-sm text-red-200">{error}</p> : null}
          </div>
        </section>

        <section className="grid gap-6">
          <div className="rounded-[2rem] border border-white/10 bg-slate-950/70 p-5">
            <div className="flex items-center justify-between gap-3">
              <p className="text-xs uppercase tracking-[0.24em] text-cyan-200/80">
                Streaming reply
              </p>
              <span className="rounded-full border border-white/10 px-3 py-1 text-[11px] uppercase tracking-[0.2em] text-slate-300">
                text first
              </span>
            </div>
            <p className="mt-4 min-h-24 whitespace-pre-wrap text-lg leading-8 text-slate-100">
              {reply || "Morgan's streamed response will appear here."}
            </p>
          </div>

          <div className="rounded-[2rem] border border-white/10 bg-slate-950/70 p-5">
            <div className="flex items-center justify-between gap-3">
              <p className="text-xs uppercase tracking-[0.24em] text-cyan-200/80">
                Voice audio
              </p>
              <span className="rounded-full border border-white/10 px-3 py-1 text-[11px] uppercase tracking-[0.2em] text-slate-300">
                reusable driver
              </span>
            </div>
            {audioUrl ? (
              <audio className="mt-5 w-full" src={audioUrl} controls />
            ) : (
              <p className="mt-4 text-sm text-slate-400">Audio appears after the text stream.</p>
            )}
          </div>

          <div className="rounded-[2rem] border border-white/10 bg-slate-950/70 p-5">
            <div className="flex items-center justify-between gap-3">
              <p className="text-xs uppercase tracking-[0.24em] text-cyan-200/80">
                EchoMimic avatar MP4
              </p>
              <span className="rounded-full border border-amber-200/20 px-3 py-1 text-[11px] uppercase tracking-[0.2em] text-amber-100">
                batch render
              </span>
            </div>
            {videoUrl ? (
              <video
                className="mt-5 max-h-[68vh] w-full rounded-3xl bg-black object-contain"
                src={videoUrl}
                poster="/morgan.jpg"
                controls
                autoPlay
                playsInline
              />
            ) : (
              <div className="relative mt-5 min-h-96 overflow-hidden rounded-3xl border border-white/10 bg-black">
                <Image
                  src="/morgan.jpg"
                  alt="Morgan golden retriever avatar source"
                  fill
                  priority
                  sizes="(min-width: 1024px) 52vw, 100vw"
                  className="object-contain"
                />
                <div className="absolute inset-x-4 bottom-4 rounded-2xl border border-white/10 bg-slate-950/80 p-4 shadow-2xl shadow-black/40 backdrop-blur">
                  <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/80">
                    Waiting for generated video
                  </p>
                  <p className="mt-2 text-sm leading-6 text-slate-200">{avatarStatus}</p>
                </div>
              </div>
            )}
          </div>
        </section>
      </div>
    </main>
  );
}
