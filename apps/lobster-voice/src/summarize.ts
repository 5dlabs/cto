import type { SpeakerId } from "./speakers";

const MAX_CHARS = 500;

const CLOSINGS: Partial<Record<SpeakerId, string>> = {
  optimist: "That is my position.",
  pessimist: "Those are my concerns.",
  "voter-1": "That is my assessment.",
  "voter-2": "That is my vote.",
  "voter-3": "That is my recommendation.",
  "voter-4": "That is my evaluation.",
  "voter-5": "That is my verdict.",
  compiler: "That concludes the brief.",
};

/**
 * Extract a speakable summary from potentially long LLM debate output.
 * Pulls DECISION_POINT blocks if present, otherwise takes the opening text.
 */
export function summarizeForSpeech(text: string, speaker: SpeakerId): string {
  if (!text || text.length <= MAX_CHARS) return text;

  const dpBlocks = extractDecisionPoints(text);
  if (dpBlocks) {
    const trimmed = truncate(dpBlocks, MAX_CHARS);
    return `${trimmed} ${CLOSINGS[speaker] ?? ""}`.trim();
  }

  const firstParagraph = text.split(/\n\n/)[0] ?? text;
  const trimmed = truncate(firstParagraph, MAX_CHARS);
  const closing = CLOSINGS[speaker] ?? "";

  return `${trimmed} ${closing}`.trim();
}

function extractDecisionPoints(text: string): string | null {
  const dpRegex = /DECISION_POINT\s*\{[^}]*\}/g;
  const matches = text.match(dpRegex);
  if (!matches || matches.length === 0) return null;

  const summaries = matches.map((block) => {
    const id = block.match(/id:\s*"?([^",\n}]+)/)?.[1] ?? "unknown";
    const desc = block.match(/description:\s*"?([^"\n}]+)/)?.[1] ?? "";
    const option = block.match(/proposing_option:\s*"?([^"\n}]+)/)?.[1] ?? "";
    if (option) return `On ${id}: I propose ${option}. ${desc}`;
    return `On ${id}: ${desc}`;
  });

  return summaries.join(" ");
}

function truncate(text: string, max: number): string {
  if (text.length <= max) return text;

  const cut = text.slice(0, max);
  const lastSentence = cut.lastIndexOf(". ");
  if (lastSentence > max * 0.5) {
    return cut.slice(0, lastSentence + 1);
  }
  return cut.trimEnd() + "...";
}
