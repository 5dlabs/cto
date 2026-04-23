import { describe, it, expect } from "vitest";
import {
  mapAgentStateToAvatar,
  mapSessionStateToAgentState,
  type AgentState,
  type AgentStateMapping,
} from "../avatar-state-mapping";
import type { AvatarSessionState } from "../avatar-state";

const ALL_AGENT_STATES: readonly AgentState[] = [
  "idle",
  "connecting",
  "listening",
  "thinking",
  "speaking",
  "error",
];

const ALL_SESSION_STATES: readonly AvatarSessionState[] = [
  "idle",
  "connecting",
  "connected",
  "listening",
  "speaking",
  "reconnecting",
  "error",
  "disconnecting",
];

describe("mapAgentStateToAvatar — spec table", () => {
  const cases: Array<[AgentState, AgentStateMapping]> = [
    [
      "idle",
      {
        voiceState: "idle",
        gesture: { name: "idle", intensity: 0.3 },
        cueSource: "none",
      },
    ],
    [
      "connecting",
      {
        voiceState: "connecting",
        gesture: { name: "think", intensity: 0.35 },
        cueSource: "none",
      },
    ],
    [
      "listening",
      {
        voiceState: "listening",
        gesture: { name: "listen", intensity: 0.7 },
        cueSource: "none",
      },
    ],
    [
      "thinking",
      {
        voiceState: "listening",
        gesture: { name: "think", intensity: 0.6 },
        cueSource: "none",
      },
    ],
    [
      "speaking",
      {
        voiceState: "speaking",
        gesture: { name: "speak", intensity: 0.9 },
        cueSource: "elevenlabs-alignment",
      },
    ],
    [
      "error",
      {
        voiceState: "error",
        gesture: { name: "acknowledge", intensity: 0.2 },
        cueSource: "none",
      },
    ],
  ];

  for (const [input, expected] of cases) {
    it(`maps ${input} to the canonical row`, () => {
      expect(mapAgentStateToAvatar(input)).toEqual(expected);
    });
  }

  it("snapshot of full table", () => {
    const table = Object.fromEntries(
      ALL_AGENT_STATES.map((s) => [s, mapAgentStateToAvatar(s)] as const),
    );
    expect(table).toMatchInlineSnapshot(`
      {
        "connecting": {
          "cueSource": "none",
          "gesture": {
            "intensity": 0.35,
            "name": "think",
          },
          "voiceState": "connecting",
        },
        "error": {
          "cueSource": "none",
          "gesture": {
            "intensity": 0.2,
            "name": "acknowledge",
          },
          "voiceState": "error",
        },
        "idle": {
          "cueSource": "none",
          "gesture": {
            "intensity": 0.3,
            "name": "idle",
          },
          "voiceState": "idle",
        },
        "listening": {
          "cueSource": "none",
          "gesture": {
            "intensity": 0.7,
            "name": "listen",
          },
          "voiceState": "listening",
        },
        "speaking": {
          "cueSource": "elevenlabs-alignment",
          "gesture": {
            "intensity": 0.9,
            "name": "speak",
          },
          "voiceState": "speaking",
        },
        "thinking": {
          "cueSource": "none",
          "gesture": {
            "intensity": 0.6,
            "name": "think",
          },
          "voiceState": "listening",
        },
      }
    `);
  });
});

describe("mapAgentStateToAvatar — purity & totality", () => {
  it("is referentially stable across calls (same input → equal output)", () => {
    for (const s of ALL_AGENT_STATES) {
      const a = mapAgentStateToAvatar(s);
      const b = mapAgentStateToAvatar(s);
      expect(a).toEqual(b);
    }
  });

  it("does not share mutable state between calls", () => {
    const a = mapAgentStateToAvatar("speaking");
    a.gesture.intensity = 999;
    a.voiceState = "idle";
    const b = mapAgentStateToAvatar("speaking");
    expect(b.gesture.intensity).toBe(0.9);
    expect(b.voiceState).toBe("speaking");
  });

  it("handles every AgentState without throwing", () => {
    for (const s of ALL_AGENT_STATES) {
      expect(() => mapAgentStateToAvatar(s)).not.toThrow();
    }
  });
});

describe("mapSessionStateToAgentState", () => {
  const cases: Array<[AvatarSessionState, AgentState]> = [
    ["idle", "idle"],
    ["connecting", "connecting"],
    ["connected", "idle"],
    ["listening", "listening"],
    ["speaking", "speaking"],
    ["reconnecting", "connecting"],
    ["error", "error"],
    ["disconnecting", "idle"],
  ];

  for (const [input, expected] of cases) {
    it(`maps session ${input} → agent ${expected}`, () => {
      expect(mapSessionStateToAgentState(input)).toBe(expected);
    });
  }

  it("covers all 8 session states", () => {
    const covered = new Set(cases.map(([s]) => s));
    for (const s of ALL_SESSION_STATES) {
      expect(covered.has(s)).toBe(true);
    }
  });

  it("is pure across calls", () => {
    for (const s of ALL_SESSION_STATES) {
      expect(mapSessionStateToAgentState(s)).toBe(mapSessionStateToAgentState(s));
    }
  });
});

describe("composition: session → agent → avatar", () => {
  it("produces valid AgentStateMapping for every session state", () => {
    for (const s of ALL_SESSION_STATES) {
      const mapping = mapAgentStateToAvatar(mapSessionStateToAgentState(s));
      expect(mapping.voiceState).toBeDefined();
      expect(mapping.gesture.name).toBeDefined();
      expect(mapping.cueSource).toBeDefined();
    }
  });
});
