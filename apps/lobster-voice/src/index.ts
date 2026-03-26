import { writeFileSync, unlinkSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";
import { randomBytes } from "crypto";

import { getVoiceConfig, resolveLevel } from "./voices";
import type { VoiceConfig, VoiceLevel, TtsProvider } from "./voices";
import { getSpeaker, isSpeakerId, allSpeakerIds } from "./speakers";
import type { SpeakerId } from "./speakers";
import { acquireLock, releaseLock, listLocks } from "./process-lock";
import { synthesize } from "./elevenlabs";
import { synthesizeOpenAI } from "./openai-tts";
import { synthesizeXai } from "./xai-tts";
import { getCached, putCached, pruneExpired } from "./cache";
import { playAudio } from "./player";
import { humanizeStep, humanizeGate, humanizeRaw } from "./humanize";
import { llmHumanize } from "./llm-humanize";
import { summarizeForSpeech } from "./summarize";

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

async function speakChunk(text: string, voice: VoiceConfig, level: VoiceLevel, speaker?: SpeakerId): Promise<void> {
  const cached = getCached(voice.voiceId, text);
  if (cached) {
    const tmpFile = join(tmpdir(), `lv-${randomBytes(4).toString("hex")}.mp3`);
    writeFileSync(tmpFile, cached);
    const ok = playAudio(tmpFile);
    try { unlinkSync(tmpFile); } catch { /* ignore */ }
    if (ok) return;
  }

  const chain = buildFallbackChain(voice.provider);

  for (const provider of chain) {
    const apiKey = getApiKey(provider);
    if (!apiKey) continue;

    const effectiveVoice = provider === voice.provider ? voice : fallbackVoice(provider);

    try {
      const audio = await synthesizeWithProvider(text, effectiveVoice, apiKey);
      const cachedPath = putCached(effectiveVoice.voiceId, text, audio);
      const ok = playAudio(cachedPath);
      if (ok) return;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      process.stderr.write(`lobster-voice: ${provider} failed (${msg}), trying next provider\n`);
    }
  }

  process.stderr.write(`lobster-voice: all TTS providers exhausted for chunk — silent skip\n`);
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

TTS Provider Cascade:
  Each speaker has a primary provider. If that provider fails or has no key,
  the system cascades: ElevenLabs → OpenAI → xAI. Silent skip if all fail.
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
