#!/usr/bin/env python3
"""Rewrite ``scripts/<path>`` to ``scripts/YYYY-MM/<path>`` after month bucketing.

Only paths that exist in the current ``git ls-files scripts/`` tree are updated
(excluding ``scripts/README.md`` and other non-bucketed files are skipped).
Run from repository root. Intended to be used once per reorganization.
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

SKIP_PREFIX = (
    "scripts/README.md",  # index file stays at ``scripts/README.md``
)


def _build_map(scripts: Path) -> dict[str, str]:
    m: dict[str, str] = {}
    for line in (
        subprocess.check_output(["git", "ls-files", "scripts/"], text=True)
        .strip()
        .splitlines()
    ):
        p = line.strip()
        if not p or p in SKIP_PREFIX:
            continue
        parts = Path(p).parts
        if len(parts) < 3:
            continue
        # scripts / YYYY-MM / ...
        mkey = parts[1]
        if len(mkey) != 7 or mkey[4] != "-":
            continue
        rest = str(Path(*parts[2:]))
        m[rest] = f"{mkey}/{rest}"
    return m


def _should_process(path: Path, repo: Path) -> bool:
    s = str(path)
    if ".archive" in s:
        return False
    if "node_modules" in s or "target/" in s:
        return False
    return True


def _try_replace(content: str, m: dict[str, str]) -> str | None:
    # Longest key first: ``talos/x`` before ``t``
    items = sorted(m.items(), key=lambda kv: -len(kv[0]))
    out = content
    for rest, mpath in items:
        for prefix in ("./scripts/", "scripts/"):
            old = f"{prefix}{rest}"
            if old in out:
                new = f"{prefix}{mpath}"
                out = out.replace(old, new)
    if out == content:
        return None
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path.cwd())
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    repo: Path = args.root.resolve()
    os_scripts = repo / "scripts"
    if not os_scripts.is_dir():
        print("No scripts/ directory", file=sys.stderr)
        return 1
    m = _build_map(os_scripts)
    if not m:
        print("No bucketed paths found under scripts/", file=sys.stderr)
        return 1
    n_files = 0
    for line in subprocess.check_output(["git", "ls-files"], text=True, cwd=repo).splitlines():
        p = (repo / line).resolve()
        if not p.is_file() or not _should_process(p, repo):
            continue
        if p.suffix in {".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico", ".woff", ".woff2", ".eot", ".ttf", ".otf", ".lock"}:
            continue
        if p.suffix in {".bin", ".pyc", ".so", ".dylib", ".a"}:
            continue
        if "/fonts/" in str(p) or p.name.endswith((".otf", ".ttf")):
            continue
        if p.suffix in {".mp3", ".mp4", ".glb", ".gltf", ".bin"}:
            continue
        try:
            data = p.read_text(encoding="utf-8", errors="strict")
        except OSError:
            continue
        except UnicodeDecodeError:
            data = p.read_text(encoding="utf-8", errors="replace")
        new = _try_replace(data, m)
        if not new or new == data:
            continue
        n_files += 1
        if not args.dry_run:
            p.write_text(new, encoding="utf-8", newline="")
    print(f"updated {n_files} files" + (" (dry-run not written)" if args.dry_run else ""))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
