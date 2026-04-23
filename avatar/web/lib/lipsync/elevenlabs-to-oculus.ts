import type { OvrLipSyncViseme } from "@/lib/avatar-state";

/**
 * Convert a single character into its closest OVRLipSync (Oculus 15) viseme.
 *
 * Kept intentionally simple: the ElevenLabs WS alignment stream emits raw
 * characters (not phonemes), so we use a letter-class heuristic that's good
 * enough for lip-shape cues at mouth-sync latencies. Punctuation / whitespace
 * collapse to `sil`.
 *
 * Covers the 15 Oculus visemes: sil, PP, FF, TH, DD, kk, CH, SS, nn, RR,
 * aa, E, I, O, U.
 */
export function charToViseme(ch: string): OvrLipSyncViseme {
  const lower = ch.toLowerCase();
  if (/[a]/.test(lower)) return "aa";
  if (/[e]/.test(lower)) return "E";
  if (/[i]/.test(lower)) return "I";
  if (/[o]/.test(lower)) return "O";
  if (/[u]/.test(lower)) return "U";
  if (/[pbm]/.test(lower)) return "PP";
  if (/[fv]/.test(lower)) return "FF";
  if (/[td]/.test(lower)) return "DD";
  if (/[kghq]/.test(lower)) return "kk";
  if (/[csz]/.test(lower)) return "SS";
  if (/[n]/.test(lower)) return "nn";
  if (/[r]/.test(lower)) return "RR";
  if (/[l]/.test(lower)) return "nn";
  if (/[j]/.test(lower)) return "CH";
  if (/[w]/.test(lower)) return "U";
  return "sil";
}

/**
 * Voice-bridge alignment frame as emitted by `infra/images/voice-bridge`.
 *
 * One frame is sent per utterance and contains a parallel-array payload
 * with character timings (milliseconds, relative to utterance start).
 */
export type AlignmentFrame = {
  type: "alignment";
  atMs: number;
  chars: string[];
  char_start_ms: number[];
  char_end_ms: number[];
  agent: string;
};

export type VisemeTimeline = {
  visemes: OvrLipSyncViseme[];
  vtimes: number[];
  vdurations: number[];
};

export type WordTimeline = {
  words: string[];
  wtimes: number[];
  wdurations: number[];
};

/**
 * Convert an ElevenLabs alignment frame into Oculus-viseme timelines that
 * TalkingHead's `speakAudio({ visemes, vtimes, vdurations })` accepts
 * directly.
 *
 * Collapses runs of identical visemes (common for consecutive vowels) into
 * a single entry spanning the full run; prevents blendshape thrashing at
 * zero value between frames.
 */
export function alignmentToVisemeTimeline(frame: AlignmentFrame): VisemeTimeline {
  const visemes: OvrLipSyncViseme[] = [];
  const vtimes: number[] = [];
  const vdurations: number[] = [];

  for (let i = 0; i < frame.chars.length; i += 1) {
    const ch = frame.chars[i];
    const startMs = frame.char_start_ms[i] ?? 0;
    const endMs = frame.char_end_ms[i] ?? startMs;
    const visemeValue = charToViseme(ch);

    const prev = visemes.length - 1;
    if (prev >= 0 && visemes[prev] === visemeValue) {
      // Extend the previous run rather than emitting a duplicate blink.
      vdurations[prev] = Math.max(vdurations[prev], endMs - vtimes[prev]);
      continue;
    }

    visemes.push(visemeValue);
    vtimes.push(startMs);
    vdurations.push(Math.max(0, endMs - startMs));
  }

  return { visemes, vtimes, vdurations };
}

/**
 * Derive per-word timings from an alignment frame.
 *
 * ElevenLabs gives us character-level timestamps; TalkingHead's `speakAudio`
 * expects a parallel `words` / `wtimes` / `wdurations` view for its
 * subtitle + head-movement cadence. Words are split on whitespace; the
 * first character's start-time anchors the word, the last character's
 * end-time anchors its duration.
 */
export function alignmentToWordTimeline(frame: AlignmentFrame): WordTimeline {
  const words: string[] = [];
  const wtimes: number[] = [];
  const wdurations: number[] = [];

  let current: { chars: string[]; startMs: number; endMs: number } | null = null;

  const flush = () => {
    if (!current || current.chars.length === 0) {
      current = null;
      return;
    }
    words.push(current.chars.join(""));
    wtimes.push(current.startMs);
    wdurations.push(Math.max(0, current.endMs - current.startMs));
    current = null;
  };

  for (let i = 0; i < frame.chars.length; i += 1) {
    const ch = frame.chars[i];
    const startMs = frame.char_start_ms[i] ?? 0;
    const endMs = frame.char_end_ms[i] ?? startMs;

    if (/\s/.test(ch)) {
      flush();
      continue;
    }

    if (!current) {
      current = { chars: [ch], startMs, endMs };
    } else {
      current.chars.push(ch);
      current.endMs = endMs;
    }
  }

  flush();

  return { words, wtimes, wdurations };
}
