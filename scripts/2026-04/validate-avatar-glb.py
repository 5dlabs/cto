#!/usr/bin/env python3
"""Validate an avatar GLB file against the canine rig blendshape contract.

Required morph targets (glTF blendshapes) on the head mesh:
  - 15 Oculus visemes (HeadAudio)
  - 52 Apple ARKit blendshapes
  - 10 Morgan canine-species add-ons

Usage:
    python3 scripts/2026-04/validate-avatar-glb.py path/to/avatar.glb

Exit 0 if all required morph targets are present, 1 otherwise.
"""

from __future__ import annotations

import json
import struct
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Blendshape contract (from CTO #4799 / WS-B2 spec)
# ---------------------------------------------------------------------------

OCULUS_VISEMES = [
    "viseme_sil", "viseme_aa", "viseme_E", "viseme_I", "viseme_O",
    "viseme_U", "viseme_PP", "viseme_FF", "viseme_TH", "viseme_DD",
    "viseme_kk", "viseme_CH", "viseme_SS", "viseme_nn", "viseme_RR",
]

APPLE_ARKIT_52 = [
    "eyeBlinkLeft", "eyeLookDownLeft", "eyeLookInLeft", "eyeLookOutLeft",
    "eyeLookUpLeft", "eyeSquintLeft", "eyeWideLeft", "eyeBlinkRight",
    "eyeLookDownRight", "eyeLookInRight", "eyeLookOutRight", "eyeLookUpRight",
    "eyeSquintRight", "eyeWideRight", "jawForward", "jawLeft", "jawRight",
    "jawOpen", "mouthClose", "mouthFunnel", "mouthPucker", "mouthLeft",
    "mouthRight", "mouthSmileLeft", "mouthSmileRight", "mouthFrownLeft",
    "mouthFrownRight", "mouthDimpleLeft", "mouthDimpleRight",
    "mouthStretchLeft", "mouthStretchRight", "mouthRollLower",
    "mouthRollUpper", "mouthShrugLower", "mouthShrugUpper", "mouthPressLeft",
    "mouthPressRight", "mouthLowerDownLeft", "mouthLowerDownRight",
    "mouthUpperUpLeft", "mouthUpperUpRight", "browDownLeft", "browDownRight",
    "browInnerUp", "browOuterUpLeft", "browOuterUpRight", "cheekPuff",
    "cheekSquintLeft", "cheekSquintRight", "noseSneerLeft", "noseSneerRight",
    "tongueOut",
]

CANINE_ADDONS = [
    "snout_open", "snout_wrinkle", "jowl_flap_left", "jowl_flap_right",
    "tongue_loll", "ear_left_rotate", "ear_right_rotate", "ear_left_droop",
    "ear_right_droop", "nose_twitch",
]

ALL_REQUIRED = set(OCULUS_VISEMES + APPLE_ARKIT_52 + CANINE_ADDONS)
EXPECTED_COUNT = len(ALL_REQUIRED)  # 15 + 52 + 10 = 77


# ---------------------------------------------------------------------------
# GLB binary parser (minimal — just enough to extract the JSON chunk)
# ---------------------------------------------------------------------------

GLB_HEADER_MAGIC = 0x46546C67  # "glTF"


def parse_glb_json(glb_path: Path) -> dict:
    """Extract the top-level JSON chunk from a GLB 2.0 binary."""
    with open(glb_path, "rb") as fh:
        magic, version, length = struct.unpack("<III", fh.read(12))
        if magic != GLB_HEADER_MAGIC:
            raise ValueError(f"not a GLB file (bad magic: {magic:#010x})")
        if version != 2:
            raise ValueError(f"unsupported GLB version {version} (expected 2)")

        # Walk chunks until we hit the JSON chunk (type 0x4E4F534A)
        while fh.tell() < length:
            chunk_length, chunk_type = struct.unpack("<II", fh.read(8))
            if chunk_type == 0x4E4F534A:  # "JSON"
                raw = fh.read(chunk_length)
                return json.loads(raw.decode("utf-8"))
            else:
                fh.seek(chunk_length, 1)

    raise ValueError("GLB file has no JSON chunk")


# ---------------------------------------------------------------------------
# Morph-target collector
# ---------------------------------------------------------------------------


def collect_morph_targets(gltf: dict) -> set[str]:
    """Return every morph target name across all meshes in the glTF."""
    names: set[str] = set()
    for mesh in gltf.get("meshes", []):
        mesh_name = mesh.get("name", "")
        extras = mesh.get("extras", {})
        target_names_extras = extras.get("targetNames", [])

        for prim_idx, primitive in enumerate(mesh.get("primitives", [])):
            targets = primitive.get("targets", [])
            for target_idx, target in enumerate(targets):
                # Method 1: name on the target object itself
                name = target.get("name", "")
                if name:
                    names.add(name)
                    continue
                # Method 2: extras targetNames array on the mesh
                if target_idx < len(target_names_extras):
                    names.add(target_names_extras[target_idx])

        # Also check if mesh itself has name metadata for morph targets
        # (some exporters put target names as mesh-level extras)
        for i, tname in enumerate(target_names_extras):
            if tname and tname in ALL_REQUIRED:
                names.add(tname)

    return names


# ---------------------------------------------------------------------------
# Validation
# ---------------------------------------------------------------------------


def validate(glb_path: Path) -> list[str]:
    """Return a list of missing morph-target names (empty = pass)."""
    gltf = parse_glb_json(glb_path)
    found = collect_morph_targets(gltf)
    missing = ALL_REQUIRED - found
    return sorted(missing)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def main() -> None:
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <avatar.glb>", file=sys.stderr)
        sys.exit(2)

    glb_path = Path(sys.argv[1])
    if not glb_path.is_file():
        print(f"error: file not found: {glb_path}", file=sys.stderr)
        sys.exit(2)

    try:
        missing = validate(glb_path)
    except Exception as exc:
        print(f"error: failed to parse {glb_path}: {exc}", file=sys.stderr)
        sys.exit(2)

    if missing:
        print(
            f"FAIL: {len(missing)}/{EXPECTED_COUNT} required morph targets "
            f"missing from {glb_path.name}:"
        )
        for name in missing:
            print(f"  - {name}")
        sys.exit(1)
    else:
        print(
            f"OK: all {EXPECTED_COUNT} required morph targets present "
            f"in {glb_path.name}"
        )
        sys.exit(0)


if __name__ == "__main__":
    main()
