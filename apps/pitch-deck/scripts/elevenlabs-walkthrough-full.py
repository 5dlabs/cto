#!/usr/bin/env python3
"""
Generate one MP3 for the full pitch-deck walkthrough narration (ElevenLabs TTS).

Requires:
  ELEVEN_API_KEY
  MORGAN_ELEVEN_VOICE_ID or ELEVEN_VOICE_ID

Optional:
  ELEVEN_MODEL_ID (default eleven_flash_v2_5)
  --out PATH
  --no-play

Chunk size stays under ElevenLabs per-request limits; chunks are concatenated with ffmpeg.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request

# Spoken script — aligned with docs/video-walkthrough-script.md (say column).
NARRATION = """
Hi, I'm Jonathon. This is the 5D Labs pre-seed deck: what we built, why it matters, and what we're raising. Follow along at pitch dot five d labs dot ai.

Cover. We're an AI-native venture studio. The headline is: we built the stack that builds companies — now it's the product. Pre-seed, Delaware C-corp, seven hundred fifty thousand dollars. On the cover: two paying customers, about two hundred forty K annual contract value, multiple revenue streams on one platform, and real infra savings versus a cloud-only AI delivery model. Same stack: CTO implementations, bare-metal rev-share, trading, and advisory.

Problem. When code is cheap, coordination wins — aim, orchestration, and learning fast. We use stronger models for planning and cheaper open and local stacks, including very competitive options, for iteration. Without a system you get sprawl and cloud tax. One loop: decide, ship, learn.

One machine. This isn't three random bets — it's one OS: build, fund, ship. CTO is the wedge; trading is internal capital; ventures and OpenClaw share playbooks. One loop — if you drop a piece, the whole thing weakens.

Origin. We built this for ourselves first: we wanted a Solana trading stack with a small team before models were really there — so we built CTO to ship anyway. Serious time on CTO; now the infra is real. Scratch to production to selling the wedge.

How it works. Decide — thesis and signals. Deploy — agents on CTO and OpenClaw. Learn — runtime and market feed the roadmap. Compound — reuse prompts, workflows, infra.

CTO. SDLC plus ops through Morgan; self-healing delivery; multi-CLI; bare metal for sovereignty and cost — and, not or. Serious teams self-host under AGPL.

Private cloud. This isn't a ChatGPT wrapper — it's an AWS-shaped private cloud on your metal. Five D Plan, Code, Git, and Edge — Cloudflare tunnels and edge. The table is a sample map: data, store, inference, observe, deploy, vault, edge. Full catalog lives on five d labs dot ai under CTO services.

Differentiation. Plan before code: Lobster, Stitch and Linear, voice and listen. Humans in the loop without micromanagement. The ROI line on the slide says it: fewer hires, lower infra bill, faster cycles.

Trading. In-house chains and edge; bootstrap only — not an external fund. Investor money is not trading principal.

Traction and partnerships. Sigma One as a full CTO reference; Bloq at about twenty K a month. Partners: servers dot com, I D three dot net, Latitude. In discussion with Cherry Servers — not closed — and MiniMax approached us to partner. Stack: seventeen plus bare-metal sites, four chains, twenty-two agents. Founder velocity and Pocket-era infra at over a billion requests a day peak.

Market. Beachhead is crypto-native teams; we expand to anyone burning cloud and headcount. Moat: bare metal plus full automation versus cloud-only agents.

Business model. Four revenue streams on one stack: CTO subscriptions, rev-share, trading P and L, advisory.

Go-to-market. Open-source OpenClaw slice funnels to CTO. Freemium desktop with a local kind cluster, limited agents, feature flags into tiers. Long-term, desktop is the main commercial surface.

Use of funds. Seven fifty K goes to engineers, founder runway, edge, lab, models, and buffer — about eighteen months to cash-flow positive.

Morgan. Avatar and voice — same agent as intake. OpenClaw, LiveKit, LemonSlice; Lemon Squeezy for commerce when ready. There's a Talk to Morgan button on the site.

Founder. Twenty plus years in ops, Victoria. At Pocket, Head of Infra at scale. Blocknative; Coinmiles from senior engineer to CTO in three months.

The ask. Seven fifty K post-money SAFE; cap versus comps in conversation; we care about profitability; M and A over IPO as the default story. You can export PDF, PowerPoint, or Google Slides from the deck chrome. Book a call from the link on the slide.

Thanks for watching — pitch dot five d labs dot ai.
""".strip()

MAX_CHUNK = 3500
MODEL_DEFAULT = "eleven_flash_v2_5"


def chunk_text(text: str, max_len: int) -> list[str]:
    text = " ".join(text.split())
    if len(text) <= max_len:
        return [text]
    chunks: list[str] = []
    rest = text
    while rest:
        if len(rest) <= max_len:
            chunks.append(rest.strip())
            break
        cut = rest.rfind(". ", 0, max_len)
        if cut < max_len * 0.4:
            cut = rest.rfind(" ", 0, max_len)
        if cut < 1:
            cut = max_len
        else:
            cut += 1
        chunks.append(rest[:cut].strip())
        rest = rest[cut:].strip()
    return chunks


def tts_chunk(
    text: str,
    *,
    api_key: str,
    voice_id: str,
    model_id: str,
) -> bytes:
    body = json.dumps(
        {
            "text": text,
            "model_id": model_id,
            "voice_settings": {
                "stability": 0.5,
                "similarity_boost": 0.75,
                "style": 0.3,
                "use_speaker_boost": True,
            },
        }
    ).encode()
    url = f"https://api.elevenlabs.io/v1/text-to-speech/{voice_id}"
    req = urllib.request.Request(
        url,
        data=body,
        method="POST",
        headers={
            "xi-api-key": api_key,
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        return resp.read()


def ffmpeg_concat_mp3(paths: list[str], out_path: str) -> None:
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".txt", delete=False, encoding="utf-8"
    ) as f:
        for p in paths:
            # concat demuxer wants: file 'path'
            safe = p.replace("'", "'\\''")
            f.write(f"file '{safe}'\n")
        list_path = f.name
    try:
        subprocess.run(
            [
                "ffmpeg",
                "-y",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                list_path,
                "-c",
                "copy",
                out_path,
            ],
            check=True,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as e:
        raise SystemExit(
            "ffmpeg not found. Install ffmpeg (e.g. brew install ffmpeg) to merge chunks."
        ) from e
    except subprocess.CalledProcessError as e:
        sys.stderr.write(e.stderr or "")
        raise SystemExit(f"ffmpeg failed: {e.returncode}") from e
    finally:
        try:
            os.unlink(list_path)
        except OSError:
            pass


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--out",
        default="pitch-deck-full-walkthrough.mp3",
        help="Output MP3 path",
    )
    ap.add_argument("--no-play", action="store_true", help="Do not afplay when done")
    args = ap.parse_args()

    api_key = os.environ.get("ELEVEN_API_KEY", "").strip()
    voice_id = (
        os.environ.get("MORGAN_ELEVEN_VOICE_ID") or os.environ.get("ELEVEN_VOICE_ID") or ""
    ).strip()
    model_id = os.environ.get("ELEVEN_MODEL_ID", MODEL_DEFAULT).strip()

    if not api_key:
        print("Set ELEVEN_API_KEY", file=sys.stderr)
        sys.exit(1)
    if not voice_id:
        print("Set MORGAN_ELEVEN_VOICE_ID or ELEVEN_VOICE_ID", file=sys.stderr)
        sys.exit(1)

    chunks = chunk_text(NARRATION, MAX_CHUNK)
    print(f"Narration: {len(NARRATION)} chars → {len(chunks)} TTS chunk(s)", file=sys.stderr)

    tmp_paths: list[str] = []
    try:
        for i, chunk in enumerate(chunks):
            print(f"  Synthesizing chunk {i + 1}/{len(chunks)}…", file=sys.stderr)
            try:
                audio = tts_chunk(
                    chunk, api_key=api_key, voice_id=voice_id, model_id=model_id
                )
            except urllib.error.HTTPError as e:
                body = e.read().decode(errors="replace")[:500]
                print(f"ElevenLabs HTTP {e.code}: {body}", file=sys.stderr)
                sys.exit(1)
            fd, path = tempfile.mkstemp(suffix=".mp3", prefix="walkthrough-")
            os.close(fd)
            with open(path, "wb") as f:
                f.write(audio)
            tmp_paths.append(path)

        out_abs = os.path.abspath(args.out)
        os.makedirs(os.path.dirname(out_abs) or ".", exist_ok=True)

        if len(tmp_paths) == 1:
            os.replace(tmp_paths[0], out_abs)
            tmp_paths.clear()
        else:
            ffmpeg_concat_mp3(tmp_paths, out_abs)

        print(f"Wrote: {out_abs}", file=sys.stderr)
        if not args.no_play and sys.platform == "darwin":
            subprocess.run(["afplay", out_abs], check=False)
    finally:
        for p in tmp_paths:
            try:
                os.unlink(p)
            except OSError:
                pass


if __name__ == "__main__":
    main()
