#!/usr/bin/env python3
"""Fix relative markdown links under docs/ after a month-bucket reorganization.

The historical layout was ``docs/<path>``. A link like ``../intake/w`` from
``docs/foo`` targets the repo path ``intake/w``. After moving to
``docs/YYYY-MM/<path>``, that link must be rewritten to ``../../intake/w`` from
``docs/YYYY-MM/foo``.

In-repo document links are also normalized against the canonical
``<path-without-month>`` map so that ``../other`` between documents in different
month folders still works.
"""
from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

link_re = re.compile(r"(?<!\!)\[([^\]]*)\]\(([^)]+)\)")
month_re = re.compile(r"^\d{4}-\d{2}$")


def build_canon_map(docs: Path) -> dict[str, str]:
    m: dict[str, str] = {}
    for f in docs.rglob("*"):
        if not f.is_file():
            continue
        rel = f.relative_to(docs)
        parts = rel.parts
        if not parts or not month_re.match(parts[0]):
            continue
        if len(parts) < 2:
            continue
        rest = str(Path(*parts[1:])).replace("\\", "/")
        m[rest] = str(rel).replace("\\", "/")
    return m


def _split_anchor(href: str) -> tuple[str, str]:
    if "#" in href and not href.startswith(("http://", "https://")):
        a, b = href.split("#", 1)
        return a.strip(), "#" + b
    return href.strip(), ""


def should_skip_href(href: str) -> bool:
    if not href or href.startswith(("#", "http://", "https://", "mailto:")):
        return True
    if href.startswith(("{", "<", "{{")) or "}}" in href:
        return True
    if href.strip().startswith("`"):
        return True
    return False


def repair_file(
    path: Path,
    *,
    docs: Path,
    repo: Path,
    canon_to_new: dict[str, str],
) -> bool:
    rel = path.relative_to(docs)
    parts = rel.parts
    if len(parts) < 2 or not month_re.match(parts[0]):
        return False
    logical_src = Path(*parts[1:])
    # Keep historical resolution repo-relative; absolute ``docs/`` breaks ``../`` normpath.
    old_full_rel = Path("docs", *logical_src.parts)
    text = path.read_text(encoding="utf-8", errors="replace")

    def repl(m: re.Match[str]) -> str:
        label, raw = m.group(1), m.group(2).strip()
        if should_skip_href(raw):
            return m.group(0)
        path_part, anc = _split_anchor(raw)
        if not path_part:
            return m.group(0)
        if should_skip_href(path_part):
            return m.group(0)

        current = docs / rel
        # Already valid
        direct = (current.parent / path_part)
        if direct.is_file():
            return m.group(0)
        if direct.is_dir() and not str(path_part).endswith((".md", ".mdx", ".html", ".yml", ".yaml", ".ts", ".rs", ".sh")):
            return m.group(0)

        # 1) Target is another document under docs/ (use historical docs layout)
        try:
            histor = old_full_rel.parent / path_part
            n = os.path.normpath(histor.as_posix())
        except Exception:  # noqa: BLE001
            return m.group(0)
        if n in ("", ".", "/") or n.startswith("..") or os.path.isabs(n):
            return m.group(0)
        can = str(Path(n)).replace("\\", "/")
        if can in canon_to_new:
            new_abs = docs / Path(canon_to_new[can])
            r = os.path.relpath(new_abs, start=current.parent)
            if r == path_part and not anc:
                return m.group(0)
            return f"[{label}]({r}{anc})"

        # 2) Target is outside ``docs/`` in the repo (../intake/..., ../apps/...), still from historical path
        try:
            p_norm = os.path.normpath((old_full_rel.parent / path_part).as_posix())
        except Exception:  # noqa: BLE001
            return m.group(0)
        if p_norm in ("", ".", "/") or p_norm.startswith("..") or os.path.isabs(p_norm):
            return m.group(0)
        target = (repo / p_norm).resolve()
        try:
            target.relative_to(repo.resolve())
        except ValueError:
            return m.group(0)
        if not target.exists():
            return m.group(0)
        r2 = os.path.relpath(target, start=current.parent)
        if r2 == path_part and not anc:
            return m.group(0)
        return f"[{label}]({r2}{anc})"

    new_text, count = link_re.subn(repl, text)
    if not count or new_text == text:
        return False
    path.write_text(new_text, encoding="utf-8", newline="")
    return True


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path.cwd())
    args = ap.parse_args()
    repo: Path = args.root.resolve()
    docs = repo / "docs"
    if not docs.is_dir():
        print("No docs/ directory", file=sys.stderr)
        return 1
    canon = build_canon_map(docs)
    n = 0
    for p in sorted(docs.rglob("*.md")):
        if repair_file(p, docs=docs, repo=repo, canon_to_new=canon):
            n += 1
    print(f"updated {n} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
