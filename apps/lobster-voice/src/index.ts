import { appendFileSync, copyFileSync, mkdirSync, readFileSync, renameSync, rmSync, unlinkSync, writeFileSync } from "fs";
import { basename, dirname, join } from "path";
import { tmpdir } from "os";
import { randomBytes } from "crypto";
import { execFileSync } from "child_process";

import { getVoiceConfig, resolveLevel } from "./voices";
import type { VoiceConfig, VoiceLevel, TtsProvider } from "./voices";
import { getSpeaker, isSpeakerId, allSpeakerIds } from "./speakers";
import type { SpeakerId } from "./speakers";
import { acquireLock, releaseLock, listLocks } from "./process-lock";
import { synthesize } from "./elevenlabs";
import { synthesizeOpenAI } from "./openai-tts";
import { synthesizeXai } from "./xai-tts";
import { getCachedPath, putCached, pruneExpired } from "./cache";
import { playAudio } from "./player";
import { humanizeStep, humanizeGate, humanizeRaw } from "./humanize";
import { llmHumanize } from "./llm-humanize";
import { summarizeForSpeech } from "./summarize";
import { pronounce } from "./pronunciation";

function parseArgs(argv: string[]): {
  command: string;
  positional: string[];
  flags: Record<string, string>;
} {
  const args = argv.slice(2);
  const command = args[0] ?? "help";
  const positional: string[] = [];
  const flags: Record<string, string> = {};

  let i = 1;
  while (i < args.length) {
    const arg = args[i];
    if (arg.startsWith("--")) {
      const key = arg.slice(2);
      const next = args[i + 1];
      if (next && !next.startsWith("--")) {
        flags[key] = next;
        i += 2;
      } else {
        flags[key] = "true";
        i++;
      }
    } else {
      positional.push(arg);
      i++;
    }
  }

  return { command, positional, flags };
}

async function applyLlm(text: string): Promise<string> {
  if (process.env.LOBSTER_VOICE_LLM !== "1") return text;
  const rewritten = await llmHumanize(text);
  return rewritten ?? text;
}

function resolveVoice(flags: Record<string, string>, level: VoiceLevel): VoiceConfig {
  const speakerFlag = flags.speaker;
  if (speakerFlag && isSpeakerId(speakerFlag)) {
    return getSpeaker(speakerFlag)!.voice;
  }
  return getVoiceConfig(level);
}

const PROVIDER_API_KEYS: Record<TtsProvider, string> = {
  elevenlabs: "ELEVEN_API_KEY",
  openai: "OPENAI_API_KEY",
  xai: "XAI_API_KEY",
};

const FALLBACK_ORDER: TtsProvider[] = ["elevenlabs", "openai", "xai"];

function getApiKey(provider: TtsProvider): string | undefined {
  return process.env[PROVIDER_API_KEYS[provider]];
}

async function synthesizeWithProvider(text: string, voice: VoiceConfig, apiKey: string): Promise<Buffer> {
  switch (voice.provider) {
    case "elevenlabs":
      return synthesize({ text, voice, apiKey });
    case "openai":
      return synthesizeOpenAI(text, voice, apiKey);
    case "xai":
      return synthesizeXai(text, voice, apiKey);
    default: {
      const _exhaustive: never = voice.provider;
      throw new Error(`Unknown TTS provider: ${_exhaustive}`);
    }
  }
}

function buildFallbackChain(primary: TtsProvider): TtsProvider[] {
  return [primary, ...FALLBACK_ORDER.filter((p) => p !== primary)];
}

function fallbackVoice(provider: TtsProvider): VoiceConfig {
  switch (provider) {
    case "elevenlabs":
      return { provider: "elevenlabs", voiceId: "pFZP5JQG7iQjIQuC4Bku", model: "eleven_flash_v2_5", stability: 0.55, similarityBoost: 0.75, style: 0.3 };
    case "openai":
      return { provider: "openai", voiceId: "alloy", model: "tts-1", speed: 1.0 };
    case "xai":
      return { provider: "xai", voiceId: "eve", language: "en" };
  }
}

const CHUNK_LIMIT = 4000;

type RenderStatusState = "pending" | "running" | "complete" | "failed";

interface TranscriptSegment {
  speaker: SpeakerId;
  text: string;
  label?: string;
}

interface TranscriptDocument {
  kind?: string;
  sessionId?: string;
  generatedAt?: string;
  segments: TranscriptSegment[];
}

interface RenderStatus {
  status: RenderStatusState;
  startedAt?: string;
  finishedAt?: string;
  outputPath?: string;
  segmentCount?: number;
  chunkCount?: number;
  error?: string;
}

function nowIso(): string {
  return new Date().toISOString();
}

function ensureParentDir(filePath: string): void {
  mkdirSync(dirname(filePath), { recursive: true });
}

function writeJsonFile(filePath: string, value: unknown): void {
  ensureParentDir(filePath);
  const tmpPath = `${filePath}.${randomBytes(4).toString("hex")}.tmp`;
  writeFileSync(tmpPath, `${JSON.stringify(value, null, 2)}\n`);
  renameSync(tmpPath, filePath);
}

function logRender(logPath: string | undefined, message: string): void {
  const line = `[${nowIso()}] ${message}`;
  process.stderr.write(`lobster-voice: ${message}\n`);
  if (!logPath) return;
  ensureParentDir(logPath);
  appendFileSync(logPath, `${line}\n`);
}

function writeRenderStatus(statusPath: string | undefined, status: RenderStatus): void {
  if (!statusPath) return;
  writeJsonFile(statusPath, status);
}

function normalizeTranscriptSegment(raw: unknown, index: number): TranscriptSegment {
  if (!raw || typeof raw !== "object") {
    throw new Error(`render-transcript: segment ${index + 1} is not an object`);
  }

  const candidate = raw as Record<string, unknown>;
  const speaker = typeof candidate.speaker === "string" ? candidate.speaker : "";
  const text = typeof candidate.text === "string" ? candidate.text.trim() : "";
  const label = typeof candidate.label === "string" && candidate.label.trim().length > 0
    ? candidate.label.trim()
    : undefined;

  if (!isSpeakerId(speaker)) {
    throw new Error(`render-transcript: segment ${index + 1} has invalid speaker "${speaker}"`);
  }
  if (!text) {
    throw new Error(`render-transcript: segment ${index + 1} is missing text`);
  }

  return { speaker, text, label };
}

function readTranscriptDocument(inputPath: string): TranscriptDocument {
  const raw = readFileSync(inputPath, "utf-8");
  const parsed = JSON.parse(raw) as unknown;

  if (Array.isArray(parsed)) {
    return {
      generatedAt: nowIso(),
      segments: parsed.map((segment, index) => normalizeTranscriptSegment(segment, index)),
    };
  }

  if (!parsed || typeof parsed !== "object") {
    throw new Error("render-transcript: input must be an array or object with segments");
  }

  const record = parsed as Record<string, unknown>;
  if (!Array.isArray(record.segments)) {
    throw new Error("render-transcript: object input must include a segments array");
  }

  return {
    kind: typeof record.kind === "string" ? record.kind : undefined,
    sessionId: typeof record.sessionId === "string" ? record.sessionId : undefined,
    generatedAt: typeof record.generatedAt === "string" ? record.generatedAt : nowIso(),
    segments: record.segments.map((segment, index) => normalizeTranscriptSegment(segment, index)),
  };
}

function escapeFfmpegListPath(filePath: string): string {
  return filePath.replace(/'/g, `'\\''`);
}

function maybeTestAudio(): Buffer | null {
  const fixturePath = process.env.LOBSTER_VOICE_TEST_MP3;
  if (!fixturePath) return null;
  return readFileSync(fixturePath);
}

function chunkText(text: string): string[] {
  if (text.length <= CHUNK_LIMIT) return [text];

  const chunks: string[] = [];
  let remaining = text;

  while (remaining.length > CHUNK_LIMIT) {
    let cut = remaining.lastIndexOf(". ", CHUNK_LIMIT);
    if (cut < CHUNK_LIMIT * 0.3) cut = remaining.lastIndexOf(".\n", CHUNK_LIMIT);
    if (cut < CHUNK_LIMIT * 0.3) cut = remaining.lastIndexOf("\n", CHUNK_LIMIT);
    if (cut < CHUNK_LIMIT * 0.3) cut = remaining.lastIndexOf(" ", CHUNK_LIMIT);
    if (cut < 1) cut = CHUNK_LIMIT;
    else cut += 1;

    chunks.push(remaining.slice(0, cut).trim());
    remaining = remaining.slice(cut).trim();
  }

  if (remaining) chunks.push(remaining);
  return chunks;
}

async function resolveChunkAudioPath(text: string, voice: VoiceConfig, logPath?: string): Promise<string> {
  const ttsText = pronounce(text);
  const chain = buildFallbackChain(voice.provider);
  const testAudio = maybeTestAudio();

  for (const provider of chain) {
    const effectiveVoice = provider === voice.provider ? voice : fallbackVoice(provider);
    const cachedPath = getCachedPath(effectiveVoice.voiceId, ttsText);
    if (cachedPath) {
      return cachedPath;
    }

    if (testAudio) {
      return putCached(effectiveVoice.voiceId, ttsText, testAudio);
    }

    const apiKey = getApiKey(provider);
    if (!apiKey) continue;

    try {
      const audio = await synthesizeWithProvider(ttsText, effectiveVoice, apiKey);
      return putCached(effectiveVoice.voiceId, ttsText, audio);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      logRender(logPath, `${provider} failed for transcript chunk (${msg}), trying next provider`);
    }
  }

  throw new Error("all TTS providers exhausted for chunk");
}

async function speakChunk(text: string, voice: VoiceConfig, level: VoiceLevel, speaker?: SpeakerId): Promise<void> {
  try {
    const audioPath = await resolveChunkAudioPath(text, voice);
    const ok = playAudio(audioPath);
    if (ok) return;
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    process.stderr.write(`lobster-voice: ${msg} — silent skip\n`);
    return;
  }

  process.stderr.write("lobster-voice: audio playback failed — silent skip\n");
}

async function speakWithVoice(text: string, voice: VoiceConfig, level: VoiceLevel, speaker?: SpeakerId): Promise<void> {
  const chunks = chunkText(text);
  for (const chunk of chunks) {
    await speakChunk(chunk, voice, level, speaker);
  }
}

async function handleRogueAlert(speaker: string, existingPid: number): Promise<void> {
  const alertSpeaker = getSpeaker("alert")!;
  const msg = `Warning: rogue process detected. A second ${speaker} is already running with process ID ${existingPid}.`;
  process.stderr.write(`lobster-voice: ROGUE DETECTED — ${speaker} already active as PID ${existingPid}\n`);
  await speakWithVoice(msg, alertSpeaker.voice, "error");
}

async function cmdSpeak(positional: string[], flags: Record<string, string>): Promise<void> {
  const rawText = positional.join(" ");
  if (!rawText) {
    process.stderr.write("Usage: lobster-voice speak <text> [--level normal|error|wait] [--speaker <id>]\n");
    process.exit(1);
  }

  const level = resolveLevel(flags.level);
  const voice = resolveVoice(flags, level);

  if (flags.speaker && isSpeakerId(flags.speaker)) {
    const rogue = acquireLock(flags.speaker);
    if (rogue) await handleRogueAlert(rogue.speaker, rogue.existingPid);
  }

  const text = await applyLlm(humanizeRaw(rawText));
  await speakWithVoice(text, voice, level);

  if (flags.speaker && isSpeakerId(flags.speaker)) {
    releaseLock(flags.speaker);
  }
}

async function cmdStep(positional: string[], flags: Record<string, string>): Promise<void> {
  const [workflow, step] = positional;
  if (!workflow || !step) {
    process.stderr.write("Usage: lobster-voice step <workflow> <step> [--level ...] [--project ...] [--context ...] [--speaker <id>]\n");
    process.exit(1);
  }

  const level = resolveLevel(flags.level);
  const voice = resolveVoice(flags, level);
  const context = [flags.project ? `project ${flags.project}` : "", flags.context ?? ""]
    .filter(Boolean)
    .join(". ");

  if (flags.speaker && isSpeakerId(flags.speaker)) {
    const rogue = acquireLock(flags.speaker);
    if (rogue) await handleRogueAlert(rogue.speaker, rogue.existingPid);
  }

  let text = humanizeStep(workflow, step, context || undefined);
  text = await applyLlm(text);
  await speakWithVoice(text, voice, level);

  if (flags.speaker && isSpeakerId(flags.speaker)) {
    releaseLock(flags.speaker);
  }
}

async function cmdGate(positional: string[], flags: Record<string, string>): Promise<void> {
  const [workflow, gate, result] = positional;
  if (!workflow || !gate) {
    process.stderr.write("Usage: lobster-voice gate <workflow> <gate> <result> [--level ...] [--context ...] [--speaker <id>]\n");
    process.exit(1);
  }

  const rawResult = result ?? "unknown";
  const resultLc = rawResult.toLowerCase();
  let level = resolveLevel(flags.level);
  if (resultLc.includes("fail") || resultLc.includes("error")) level = "error";
  if (resultLc.includes("wait") || resultLc.includes("retry")) level = "wait";

  const voice = resolveVoice(flags, level);

  if (flags.speaker && isSpeakerId(flags.speaker)) {
    const rogue = acquireLock(flags.speaker);
    if (rogue) await handleRogueAlert(rogue.speaker, rogue.existingPid);
  }

  let text = humanizeGate(workflow, gate, rawResult, flags.context);
  text = await applyLlm(text);
  await speakWithVoice(text, voice, level);

  if (flags.speaker && isSpeakerId(flags.speaker)) {
    releaseLock(flags.speaker);
  }
}

async function cmdAlert(positional: string[], flags: Record<string, string>): Promise<void> {
  const rawText = positional.join(" ");
  if (!rawText) {
    process.stderr.write("Usage: lobster-voice alert <text> [--level error|wait]\n");
    process.exit(1);
  }

  const level = resolveLevel(flags.level || "error");
  const voice = resolveVoice(flags, level);
  const text = await applyLlm(humanizeRaw(rawText));
  await speakWithVoice(text, voice, level);
}

async function cmdDebate(positional: string[], flags: Record<string, string>): Promise<void> {
  const [speakerArg, ...rest] = positional;
  const rawText = rest.join(" ");

  if (!speakerArg || !isSpeakerId(speakerArg)) {
    process.stderr.write(
      `Usage: lobster-voice debate <speaker> <text> [--summarize]\n` +
      `Speakers: ${allSpeakerIds().join(", ")}\n`,
    );
    process.exit(1);
  }

  if (!rawText) {
    process.stderr.write("debate: no text provided\n");
    process.exit(1);
  }

  const speaker = getSpeaker(speakerArg)!;
  const rogue = acquireLock(speakerArg);
  if (rogue) await handleRogueAlert(rogue.speaker, rogue.existingPid);

  let text = rawText;
  if (flags.summarize === "true") {
    text = summarizeForSpeech(text, speakerArg as SpeakerId);
  }

  text = humanizeRaw(text);
  text = await applyLlm(text);
  await speakWithVoice(text, speaker.voice, "normal", speakerArg as SpeakerId);

  releaseLock(speakerArg);
}

async function cmdRenderTranscript(flags: Record<string, string>): Promise<void> {
  const inputPath = flags.input;
  const outputPath = flags.output;
  const statusPath = flags.status;
  const logPath = flags.log;

  if (!inputPath || !outputPath) {
    process.stderr.write("Usage: lobster-voice render-transcript --input <json> --output <mp3> [--status <json>] [--log <path>]\n");
    process.exit(1);
  }

  const startedAt = nowIso();
  let transcript: TranscriptDocument;
  try {
    transcript = readTranscriptDocument(inputPath);
  } catch (err) {
    const error = err instanceof Error ? err.message : String(err);
    writeRenderStatus(statusPath, {
      status: "failed",
      startedAt,
      finishedAt: nowIso(),
      outputPath,
      segmentCount: 0,
      chunkCount: 0,
      error,
    });
    logRender(logPath, `render-transcript failed before synthesis: ${error}`);
    throw err;
  }
  const segments = transcript.segments.filter((segment) => segment.text.trim().length > 0);

  writeRenderStatus(statusPath, {
    status: "running",
    startedAt,
    outputPath,
    segmentCount: segments.length,
    chunkCount: 0,
  });
  logRender(logPath, `render-transcript started for ${inputPath} → ${outputPath}`);

  if (segments.length === 0) {
    const error = "render-transcript: no transcript segments to render";
    writeRenderStatus(statusPath, {
      status: "failed",
      startedAt,
      finishedAt: nowIso(),
      outputPath,
      segmentCount: 0,
      chunkCount: 0,
      error,
    });
    throw new Error(error);
  }

  ensureParentDir(outputPath);
  const tmpOutput = join(dirname(outputPath), `.${basename(outputPath)}.${randomBytes(4).toString("hex")}.tmp.mp3`);
  const tempDir = join(tmpdir(), `lobster-voice-render-${randomBytes(6).toString("hex")}`);
  mkdirSync(tempDir, { recursive: true });

  let chunkCount = 0;

  try {
    const audioFiles: string[] = [];
    for (const [index, segment] of segments.entries()) {
      const speaker = getSpeaker(segment.speaker);
      if (!speaker) {
        throw new Error(`render-transcript: unknown speaker "${segment.speaker}"`);
      }

      const text = humanizeRaw(segment.text);
      const chunks = chunkText(text);
      chunkCount += chunks.length;
      logRender(logPath, `rendering segment ${index + 1}/${segments.length} (${segment.speaker}) in ${chunks.length} chunk(s)`);

      for (const chunk of chunks) {
        audioFiles.push(await resolveChunkAudioPath(chunk, speaker.voice, logPath));
      }
    }

    if (audioFiles.length === 0) {
      throw new Error("render-transcript: no audio chunks were produced");
    }

    if (audioFiles.length === 1) {
      copyFileSync(audioFiles[0], tmpOutput);
    } else {
      const ffmpegList = join(tempDir, "concat.txt");
      const concatBody = audioFiles.map((filePath) => `file '${escapeFfmpegListPath(filePath)}'`).join("\n");
      writeFileSync(ffmpegList, `${concatBody}\n`);
      execFileSync(
        "ffmpeg",
        ["-y", "-f", "concat", "-safe", "0", "-i", ffmpegList, "-ac", "1", "-ar", "22050", "-b:a", "128k", tmpOutput],
        { stdio: ["ignore", "pipe", "pipe"] },
      );
    }

    renameSync(tmpOutput, outputPath);
    const completeStatus: RenderStatus = {
      status: "complete",
      startedAt,
      finishedAt: nowIso(),
      outputPath,
      segmentCount: segments.length,
      chunkCount,
    };
    writeRenderStatus(statusPath, completeStatus);
    logRender(logPath, `render-transcript complete (${chunkCount} chunk(s))`);
  } catch (err) {
    try { unlinkSync(tmpOutput); } catch { /* ignore */ }
    const error = err instanceof Error ? err.message : String(err);
    writeRenderStatus(statusPath, {
      status: "failed",
      startedAt,
      finishedAt: nowIso(),
      outputPath,
      segmentCount: segments.length,
      chunkCount,
      error,
    });
    logRender(logPath, `render-transcript failed: ${error}`);
    throw err;
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

function cmdLocks(): void {
  const locks = listLocks();
  if (locks.length === 0) {
    process.stdout.write("No active voice locks.\n");
    return;
  }

  process.stdout.write("Active voice locks:\n");
  for (const lock of locks) {
    const status = lock.alive ? "ALIVE" : "STALE";
    const speakerInfo = getSpeaker(lock.speaker);
    const label = speakerInfo?.label ?? lock.speaker;
    process.stdout.write(`  ${lock.speaker} (${label}) — PID ${lock.pid} [${status}]\n`);
  }
}

function cmdPrune(): void {
  const count = pruneExpired();
  if (count > 0) {
    process.stderr.write(`lobster-voice: pruned ${count} expired cache entries\n`);
  }
}

function cmdHelp(): void {
  const help = `lobster-voice — ElevenLabs TTS for Lobster workflow narration

Commands:
  speak <text> [--level normal|error|wait] [--speaker <id>]
    Speak arbitrary text. When --speaker is set, uses that speaker's unique voice.

  step <workflow> <step> [--level ...] [--project ...] [--context ...] [--speaker <id>]
    Announce a workflow step with a human-friendly description.

  gate <workflow> <gate> <result> [--level ...] [--context ...] [--speaker <id>]
    Announce a gate check result (auto-escalates level on failure).

  alert <text> [--level error|wait]
    Urgent announcement (defaults to error voice).

  debate <speaker> <text> [--summarize]
    Speak debate content with the speaker's unique voice.
    Acquires a process lock and checks for rogue duplicates.
    --summarize extracts key points from long text (~500 chars max).

  render-transcript --input <json> --output <mp3> [--status <json>] [--log <path>]
    Render a durable MP3 from a multi-speaker transcript JSON file.
    Supports either a raw array of segments or an object with { segments, kind, sessionId, generatedAt }.

  locks
    List all active voice process locks and their status.

  prune
    Remove expired audio cache entries.

  help
    Show this message.

Speakers:
  narrator    Pipeline narration (Nova — OpenAI)
  optimist    Debate optimist (Charlie — ElevenLabs)
  pessimist   Debate pessimist (Ara — xAI)
  designer    Design deliberation voice (Shimmer — OpenAI)
  voter-1     Architect voter (Daniel — ElevenLabs)
  voter-2     Pragmatist voter (Shimmer — OpenAI)
  voter-3     Minimalist voter (Sal — xAI)
  voter-4     Operator voter (Matilda — ElevenLabs)
  voter-5     Strategist voter (Leo — xAI)
  alert       Error/rogue alerts (Rex — xAI)
  system      Wait/system states (Alloy — OpenAI)
  compiler    Brief compilation (Adam — ElevenLabs)

Environment:
  ELEVEN_API_KEY             ElevenLabs API key
  OPENAI_API_KEY             OpenAI API key (TTS + optional LLM rewriting)
  XAI_API_KEY                xAI API key (TTS with expressive tags)
  LOBSTER_VOICE_LLM=1        Enable LLM rewriting of messages
  LOBSTER_VOICE_LLM_MODEL    LLM model for rewriting (default: gpt-4o-mini)
  WORKSPACE                  Repo root for cache/locks directory (default: cwd)

TTS Provider:
  All speakers use ElevenLabs exclusively (eleven_flash_v2_5 model).
  OpenAI and xAI are retained as fallbacks but all primary voices are ElevenLabs.
`;
  process.stdout.write(help);
}

async function main(): Promise<void> {
  const { command, positional, flags } = parseArgs(process.argv);

  switch (command) {
    case "speak":
      await cmdSpeak(positional, flags);
      break;
    case "step":
      await cmdStep(positional, flags);
      break;
    case "gate":
      await cmdGate(positional, flags);
      break;
    case "alert":
      await cmdAlert(positional, flags);
      break;
    case "debate":
      await cmdDebate(positional, flags);
      break;
    case "render-transcript":
      await cmdRenderTranscript(flags);
      break;
    case "locks":
      cmdLocks();
      break;
    case "prune":
      cmdPrune();
      break;
    case "help":
    case "--help":
    case "-h":
      cmdHelp();
      break;
    default:
      process.stderr.write(`Unknown command: ${command}\n`);
      cmdHelp();
      process.exit(1);
  }
}

main().catch((err) => {
  process.stderr.write(`lobster-voice: ${err}\n`);
  process.exit(1);
});
