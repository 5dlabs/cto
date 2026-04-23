/**
 * Morgan avatar defaults (WS-B1).
 *
 * Ready Player Me stand-in so the TalkingHead + HeadAudio runtime has a real
 * avatar to drive until the canine anthro rig lands in WS-B2.
 *
 * TalkingHead needs these morph targets baked into the GLB for correct
 * viseme + eye animation; RPM only bakes requested morphs, so omissions
 * silently break lip-sync or gaze.
 */

const MORGAN_RPM_ID = "64bfa15f0e72c63d7c3934a6";

const MORGAN_MORPH_TARGETS = [
  "ARKit",
  "Oculus Visemes",
  "mouthOpen",
  "mouthSmile",
  "eyesClosed",
  "eyesLookUp",
  "eyesLookDown",
].join(",");

export const MORGAN_DEFAULT_GLB_URL =
  `https://models.readyplayer.me/${MORGAN_RPM_ID}.glb` +
  `?morphTargets=${encodeURIComponent(MORGAN_MORPH_TARGETS)}`;

export const MORGAN_DEFAULT_BODY: "F" | "M" = "F";
