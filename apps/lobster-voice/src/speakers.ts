import type { VoiceConfig } from "./voices";

export type SpeakerId =
  | "narrator"
  | "optimist"
  | "pessimist"
  | "designer"
  | "voter-1"
  | "voter-2"
  | "voter-3"
  | "voter-4"
  | "voter-5"
  | "alert"
  | "system"
  | "compiler";

export interface SpeakerConfig {
  id: SpeakerId;
  label: string;
  voice: VoiceConfig;
}

// All speakers use ElevenLabs exclusively (eleven_flash_v2_5).
// Each speaker has a distinct ElevenLabs voice for variety.
const SPEAKERS: Record<SpeakerId, SpeakerConfig> = {
  narrator: {
    id: "narrator",
    label: "Narrator (Rachel — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "21m00Tcm4TlvDq8ikWAM",   // Rachel — calm, clear narration
      model: "eleven_flash_v2_5",
      stability: 0.7,
      similarityBoost: 0.75,
      style: 0.2,
    },
  },
  optimist: {
    id: "optimist",
    label: "Optimist (Charlie — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "IKne3meq5aSn9XLyUdCD",   // Charlie
      model: "eleven_flash_v2_5",
      stability: 0.5,
      similarityBoost: 0.8,
      style: 0.4,
    },
  },
  pessimist: {
    id: "pessimist",
    label: "Pessimist (Clyde — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "2EiwWnXFnvU5JabPnv8n",   // Clyde — gruff, skeptical tone
      model: "eleven_flash_v2_5",
      stability: 0.6,
      similarityBoost: 0.75,
      style: 0.3,
    },
  },
  designer: {
    id: "designer",
    label: "Designer (Bella — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "EXAVITQu4vr4xnSDxMaL",   // Bella — soft, creative
      model: "eleven_flash_v2_5",
      stability: 0.55,
      similarityBoost: 0.8,
      style: 0.35,
    },
  },
  "voter-1": {
    id: "voter-1",
    label: "Architect (Daniel — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "onwK4e9ZLuTAKqWW03F9",   // Daniel
      model: "eleven_flash_v2_5",
      stability: 0.65,
      similarityBoost: 0.75,
      style: 0.3,
    },
  },
  "voter-2": {
    id: "voter-2",
    label: "Pragmatist (Domi — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "AZnzlk1XvdvUeBnXmlld",   // Domi
      model: "eleven_flash_v2_5",
      stability: 0.55,
      similarityBoost: 0.75,
      style: 0.3,
    },
  },
  "voter-3": {
    id: "voter-3",
    label: "Minimalist (Fin — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "D38z5RcWu1voky8WS1ja",   // Fin
      model: "eleven_flash_v2_5",
      stability: 0.65,
      similarityBoost: 0.7,
      style: 0.2,
    },
  },
  "voter-4": {
    id: "voter-4",
    label: "Operator (Matilda — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "XrExE9yKIg1WjnnlVkGX",   // Matilda
      model: "eleven_flash_v2_5",
      stability: 0.5,
      similarityBoost: 0.8,
      style: 0.35,
    },
  },
  "voter-5": {
    id: "voter-5",
    label: "Strategist (Josh — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "TxGEqnHWrfWFTfGW9XjX",   // Josh
      model: "eleven_flash_v2_5",
      stability: 0.55,
      similarityBoost: 0.75,
      style: 0.3,
    },
  },
  alert: {
    id: "alert",
    label: "Alert (Antoni — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "ErXwobaYiN019PkySvjV",   // Antoni — sharp, urgent
      model: "eleven_flash_v2_5",
      stability: 0.45,
      similarityBoost: 0.8,
      style: 0.5,
    },
  },
  system: {
    id: "system",
    label: "System (Elli — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "MF3mGyEYCl7XYWbV9V6O",   // Elli — neutral, system-like
      model: "eleven_flash_v2_5",
      stability: 0.7,
      similarityBoost: 0.7,
      style: 0.15,
    },
  },
  compiler: {
    id: "compiler",
    label: "Compiler (Adam — ElevenLabs)",
    voice: {
      provider: "elevenlabs",
      voiceId: "pNInz6obpgDQGcFmaJgB",   // Adam
      model: "eleven_flash_v2_5",
      stability: 0.6,
      similarityBoost: 0.75,
      style: 0.25,
    },
  },
};

const ALL_IDS = Object.keys(SPEAKERS) as SpeakerId[];

export function getSpeaker(id: string): SpeakerConfig | undefined {
  return SPEAKERS[id as SpeakerId];
}

export function isSpeakerId(id: string): id is SpeakerId {
  return id in SPEAKERS;
}

export function allSpeakerIds(): SpeakerId[] {
  return ALL_IDS;
}
