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

// TODO(ws-b2): swap to canine rig when hosted — see avatar/web/docs/canine-rig-spec.md
export const MORGAN_DEFAULT_GLB_URL =
  `https://models.readyplayer.me/${MORGAN_RPM_ID}.glb` +
  `?morphTargets=${encodeURIComponent(MORGAN_MORPH_TARGETS)}`;

/**
 * Morgan anthro-canine rig (WS-B2).
 *
 * Activates once the asset ships at the hosted URL below and passes the
 * morph-target validation described in `avatar/web/docs/canine-rig-spec.md`
 * (ARKit 52 + Oculus visemes + canine add-ons). Commented out until then so
 * the TalkingHead runtime keeps falling back to the RPM stand-in above.
 */
// export const MORGAN_CANINE_GLB_URL =
//   "https://cdn.5dlabs.ai/avatars/morgan-canine-v1.glb";

export const MORGAN_DEFAULT_BODY: "F" | "M" = "F";
