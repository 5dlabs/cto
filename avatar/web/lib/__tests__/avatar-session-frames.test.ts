import { describe, it, expect } from "vitest";
import {
  AVATAR_SESSION_PROTOCOL,
  isAvatarSessionFrame,
  isErrorFrame,
  isSessionStateFrame,
  type ErrorFrame,
  type SessionStateFrame,
} from "../avatar-state";

const validSessionState: SessionStateFrame = {
  protocol: AVATAR_SESSION_PROTOCOL,
  type: "SESSION_STATE",
  session_id: "sess-123",
  state: "listening",
  agent_name: "rex",
  timestamp_ms: 1_700_000_000_000,
};

const validError: ErrorFrame = {
  protocol: AVATAR_SESSION_PROTOCOL,
  type: "ERROR",
  session_id: "sess-123",
  code: "LIVEKIT_DISCONNECTED",
  message: "room connection lost",
  recoverable: true,
  timestamp_ms: 1_700_000_000_000,
};

describe("isSessionStateFrame", () => {
  it("accepts a valid SESSION_STATE frame", () => {
    expect(isSessionStateFrame(validSessionState)).toBe(true);
  });

  it("rejects wrong protocol tag", () => {
    expect(
      isSessionStateFrame({ ...validSessionState, protocol: "cto-avatar-state/v1" }),
    ).toBe(false);
  });

  it("rejects wrong type discriminator", () => {
    expect(isSessionStateFrame({ ...validSessionState, type: "ERROR" })).toBe(false);
  });

  it("rejects missing required field", () => {
    const missing: Record<string, unknown> = { ...validSessionState };
    delete missing.agent_name;
    expect(isSessionStateFrame(missing)).toBe(false);
  });

  it("rejects invalid state enum value", () => {
    expect(isSessionStateFrame({ ...validSessionState, state: "bogus" })).toBe(false);
  });

  it("rejects non-object input", () => {
    expect(isSessionStateFrame(null)).toBe(false);
    expect(isSessionStateFrame("nope")).toBe(false);
  });
});

describe("isErrorFrame", () => {
  it("accepts a valid ERROR frame", () => {
    expect(isErrorFrame(validError)).toBe(true);
  });

  it("rejects wrong protocol tag", () => {
    expect(isErrorFrame({ ...validError, protocol: "cto-avatar-state/v1" })).toBe(false);
  });

  it("rejects wrong type discriminator", () => {
    expect(isErrorFrame({ ...validError, type: "SESSION_STATE" })).toBe(false);
  });

  it("rejects missing required field", () => {
    const missing: Record<string, unknown> = { ...validError };
    delete missing.recoverable;
    expect(isErrorFrame(missing)).toBe(false);
  });

  it("rejects wrong type for recoverable", () => {
    expect(isErrorFrame({ ...validError, recoverable: "yes" })).toBe(false);
  });
});

describe("isAvatarSessionFrame", () => {
  it("accepts a valid SESSION_STATE frame", () => {
    expect(isAvatarSessionFrame(validSessionState)).toBe(true);
  });

  it("accepts a valid ERROR frame", () => {
    expect(isAvatarSessionFrame(validError)).toBe(true);
  });

  it("rejects frames with wrong protocol", () => {
    expect(isAvatarSessionFrame({ ...validSessionState, protocol: "other/v1" })).toBe(false);
  });

  it("rejects unknown frame shape", () => {
    expect(
      isAvatarSessionFrame({ protocol: AVATAR_SESSION_PROTOCOL, type: "UNKNOWN" }),
    ).toBe(false);
  });
});
