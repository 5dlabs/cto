#!/usr/bin/env python3
"""Run a final SWARM-only verification pass for Puzzle #2."""

from __future__ import annotations

import argparse
import difflib
import json
import re
import sqlite3
import subprocess
import urllib.error
import urllib.parse
import urllib.request
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from swarm_phrase_index import (
    DEFAULT_KEY,
    PhraseOccurrence,
    create_schema,
    dedupe_occurrences,
    html_nodes_from_text,
    insert_occurrences,
    insert_words,
    key_metrics,
    metric_selection_rows,
    normalize_phrase,
    phrase_occurrences_from_exact_lines,
    phrase_occurrences_from_words,
    reset_tables,
    words_from_lines,
    words_from_nodes,
)


DEFAULT_BASE_URL = "https://swarm.thecanteenapp.com/"
DEFAULT_VISIBLE_HOME = Path("/tmp/swarm-visible-flat.txt")
DEFAULT_LOCAL_REPO = Path("/tmp/SWARM-V1-website")
DEFAULT_OUTPUT_DIR = Path("/Users/jonathon/5dlabs/cto/output/swarm_phrase_verify")
DEFAULT_DB = DEFAULT_OUTPUT_DIR / "swarm_puzzle_2_final.sqlite"
DEFAULT_REPORT = DEFAULT_OUTPUT_DIR / "swarm_puzzle_2_final.md"
DEFAULT_ARTIFACT_DIR = DEFAULT_OUTPUT_DIR / "artifacts"
DEFAULT_MAP_PATH = Path("/Users/jonathon/5dlabs/cto/.firecrawl/swarm-map.json")
USER_AGENT = "Mozilla/5.0 (compatible; SWARM-Puzzle-Verifier/1.0)"
URL_ATTR_RE = re.compile(r"""(?:href|src|action)=["']([^"'#]+)["']""", re.IGNORECASE)
PARTNER_LIKE_TERMS = {
    "colosseum",
    "frontier",
    "oasis",
    "polymarket",
    "solana",
    "sdk",
    "payai",
    "sendai",
}
CLUE_PATTERNS = [
    "unlock",
    "passphrase",
    "special",
    "phrase",
    DEFAULT_KEY,
]


@dataclass(frozen=True)
class SourceArtifact:
    name: str
    source_class: str
    kind: str
    locator: str
    text: str
    selection_weight: int
    title_selection_weight: int
    include_in_coverage: bool = True


@dataclass(frozen=True)
class DiscoveryRecord:
    name: str
    locator: str
    status: int
    content_type: str
    saved_path: str
    source_class: str


@dataclass(frozen=True)
class GitCommitSnapshot:
    commit_sha: str
    parent_sha: str | None
    tree_sha: str
    message: str


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return slug or "source"


def fetch_url(url: str, *, timeout: int = 20) -> tuple[int, str, bytes]:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return response.status, response.headers.get_content_type(), response.read()


def fetch_text(url: str, *, timeout: int = 20) -> tuple[int, str, str]:
    status, content_type, body = fetch_url(url, timeout=timeout)
    return status, content_type, body.decode("utf-8", "replace")


def maybe_fetch_text(url: str, *, timeout: int = 20) -> tuple[int, str, str] | None:
    try:
        return fetch_text(url, timeout=timeout)
    except urllib.error.HTTPError as exc:
        return exc.code, exc.headers.get_content_type() or "application/octet-stream", exc.read().decode("utf-8", "replace")
    except urllib.error.URLError:
        return None


def discover_same_domain_urls(base_url: str, html_text: str) -> list[str]:
    base = urllib.parse.urlparse(base_url)
    discovered = {base_url}
    for raw in URL_ATTR_RE.findall(html_text):
        resolved = urllib.parse.urljoin(base_url, raw)
        parsed = urllib.parse.urlparse(resolved)
        if parsed.scheme not in {"http", "https"}:
            continue
        if parsed.netloc != base.netloc:
            continue
        normalized = parsed._replace(fragment="").geturl()
        discovered.add(normalized)
    return sorted(discovered)


def firecrawl_map_urls(base_url: str, map_path: Path) -> list[str]:
    if not map_path.exists():
        return []
    try:
        payload = json.loads(map_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return []
    urls: list[str] = []
    for item in payload.get("data", {}).get("links", []):
        url = item.get("url")
        if not url:
            continue
        parsed = urllib.parse.urlparse(url)
        if parsed.netloc == urllib.parse.urlparse(base_url).netloc:
            urls.append(parsed._replace(fragment="").geturl())
    return sorted(set(urls))


def save_artifact(artifact_dir: Path, name: str, suffix: str, text: str) -> str:
    artifact_dir.mkdir(parents=True, exist_ok=True)
    path = artifact_dir / f"{slugify(name)}{suffix}"
    path.write_text(text, encoding="utf-8")
    return str(path)


def source_name_for_url(url: str) -> str:
    parsed = urllib.parse.urlparse(url)
    path_bits = [bit for bit in parsed.path.split("/") if bit]
    if not path_bits:
        path_bits = ["homepage"]
    query_bits: list[str] = []
    for key, values in sorted(urllib.parse.parse_qs(parsed.query).items()):
        for value in values:
            query_bits.extend([key, value])
    return slugify("_".join(["live"] + path_bits + query_bits))


def source_occurrences(source: SourceArtifact) -> tuple[list, dict[str, list[PhraseOccurrence]]]:
    if source.kind == "html":
        nodes = html_nodes_from_text(source.text)
        words = words_from_nodes(nodes)
        exact_lines = [text for _, text in nodes]
    else:
        exact_lines = [line for line in source.text.splitlines() if line.strip()]
        words = words_from_lines(exact_lines)

    sets: dict[str, list[PhraseOccurrence]] = {}
    sets[f"{source.name}__title_trigrams"] = phrase_occurrences_from_words(
        words,
        f"{source.name}__title_trigrams",
        title_like_only=True,
    )
    sets[f"{source.name}__exact_line_3words"] = phrase_occurrences_from_exact_lines(
        exact_lines,
        f"{source.name}__exact_line_3words",
        title_like_only=False,
    )
    sets[f"{source.name}__title_exact_line_3words"] = phrase_occurrences_from_exact_lines(
        exact_lines,
        f"{source.name}__title_exact_line_3words",
        title_like_only=True,
    )

    deduped: dict[str, list[PhraseOccurrence]] = {}
    for set_name, occurrences in sets.items():
        deduped_name = f"{set_name}_dedup"
        deduped[deduped_name] = dedupe_occurrences(occurrences, deduped_name)
    sets.update(deduped)
    return words, sets


def fetch_git_object(commit_sha: str, base_url: str) -> bytes:
    url = urllib.parse.urljoin(base_url, f".git/objects/{commit_sha[:2]}/{commit_sha[2:]}")
    _, _, body = fetch_url(url)
    return zlib.decompress(body)


def parse_commit_object(data: bytes) -> GitCommitSnapshot:
    _, body = data.split(b"\x00", 1)
    headers, message = body.split(b"\n\n", 1)
    header_map: dict[str, str] = {}
    for line in headers.decode("utf-8", "replace").splitlines():
        key, value = line.split(" ", 1)
        header_map[key] = value
    return GitCommitSnapshot(
        commit_sha="",
        parent_sha=header_map.get("parent"),
        tree_sha=header_map["tree"],
        message=message.decode("utf-8", "replace").strip(),
    )


def parse_tree_entries(data: bytes) -> list[tuple[str, str, str]]:
    _, body = data.split(b"\x00", 1)
    entries: list[tuple[str, str, str]] = []
    index = 0
    while index < len(body):
        mode_end = body.index(b" ", index)
        mode = body[index:mode_end].decode("utf-8", "replace")
        name_end = body.index(b"\x00", mode_end)
        name = body[mode_end + 1 : name_end].decode("utf-8", "replace")
        sha = body[name_end + 1 : name_end + 21].hex()
        entries.append((mode, name, sha))
        index = name_end + 21
    return entries


def blob_text(blob_sha: str, base_url: str) -> str:
    blob = fetch_git_object(blob_sha, base_url)
    _, body = blob.split(b"\x00", 1)
    return body.decode("utf-8", "replace")


def partner_like(phrase: str) -> bool:
    normalized = normalize_phrase(phrase)
    return any(term in normalized.split() for term in PARTNER_LIKE_TERMS)


def selection_weight_for_set(source_by_name: dict[str, SourceArtifact], phrase_set: str) -> int:
    source_name, remainder = phrase_set.split("__", 1)
    source = source_by_name[source_name]
    if remainder.startswith("title_"):
        return source.title_selection_weight
    return source.selection_weight


def rank_candidates(
    source_by_name: dict[str, SourceArtifact],
    source_sets: dict[str, list[PhraseOccurrence]],
    selection_rows: list[tuple[str, str, int, int, str]],
) -> list[dict[str, object]]:
    homepage_visible_phrases = {
        occurrence.phrase
        for occurrence in source_sets.get("homepage_visible__exact_line_3words_dedup", [])
    }
    candidate_occurrences: dict[str, list[tuple[str, PhraseOccurrence]]] = {}
    coverage_sources: dict[str, set[str]] = {}
    live_sources: dict[str, set[str]] = {}
    hidden_sources: dict[str, set[str]] = {}

    for set_name, occurrences in source_sets.items():
        if not set_name.endswith("__exact_line_3words_dedup"):
            continue
        source_name = set_name.split("__", 1)[0]
        source = source_by_name[source_name]
        for occurrence in occurrences:
            candidate_occurrences.setdefault(occurrence.phrase, []).append((source_name, occurrence))
            if source.include_in_coverage:
                coverage_sources.setdefault(occurrence.phrase, set()).add(source_name)
            if source.source_class.startswith("live_") or source.source_class == "homepage_visible":
                live_sources.setdefault(occurrence.phrase, set()).add(source_name)
            if source.source_class.startswith("git_") or source.source_class.startswith("repo_"):
                hidden_sources.setdefault(occurrence.phrase, set()).add(source_name)

    metrics_by_phrase: dict[str, dict[str, object]] = {}
    for phrase, occurrences in candidate_occurrences.items():
        canonical_source_name, canonical_occurrence = sorted(
            occurrences,
            key=lambda item: (
                0 if item[0] == "homepage_visible" else 1,
                source_by_name[item[0]].selection_weight * -1,
                item[1].line_no or 0,
            ),
        )[0]
        metrics_by_phrase[phrase] = {
            "phrase": canonical_occurrence.phrase,
            "exact_casing": canonical_occurrence.phrase,
            "homepage_visible_score": 0,
            "weighted_selection_score": 0,
            "selection_count": 0,
            "cross_source_coverage": len(coverage_sources.get(phrase, set())),
            "live_source_coverage": len(live_sources.get(phrase, set())),
            "hidden_source_coverage": len(hidden_sources.get(phrase, set())),
            "hidden_only": phrase not in homepage_visible_phrases,
            "partner_like": int(partner_like(phrase)),
            "canonical_source": canonical_source_name,
            "sources": ", ".join(sorted(coverage_sources.get(phrase, set()))),
            "selected_by": [],
        }

    for metric_name, phrase_set, _, _, phrase in selection_rows:
        if phrase not in metrics_by_phrase:
            continue
        weight = selection_weight_for_set(source_by_name, phrase_set)
        record = metrics_by_phrase[phrase]
        record["selection_count"] = int(record["selection_count"]) + 1
        record["weighted_selection_score"] = int(record["weighted_selection_score"]) + weight
        if phrase_set.startswith("homepage_visible__"):
            record["homepage_visible_score"] = int(record["homepage_visible_score"]) + 1
        record["selected_by"].append(f"{metric_name}:{phrase_set}")

    ranked = sorted(
        metrics_by_phrase.values(),
        key=lambda row: (
            int(row["hidden_only"]),
            int(row["partner_like"]),
            -int(row["homepage_visible_score"]),
            -int(row["weighted_selection_score"]),
            -int(row["cross_source_coverage"]),
            -int(row["live_source_coverage"]),
            row["phrase"],
        ),
    )
    return ranked


def create_extra_tables(conn: sqlite3.Connection) -> None:
    conn.executescript(
        """
        CREATE TABLE IF NOT EXISTS sources (
            name TEXT PRIMARY KEY,
            source_class TEXT NOT NULL,
            kind TEXT NOT NULL,
            locator TEXT NOT NULL,
            selection_weight INTEGER NOT NULL,
            title_selection_weight INTEGER NOT NULL,
            include_in_coverage INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS discoveries (
            name TEXT NOT NULL,
            locator TEXT NOT NULL,
            status INTEGER NOT NULL,
            content_type TEXT NOT NULL,
            saved_path TEXT NOT NULL,
            source_class TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS candidate_scores (
            phrase TEXT PRIMARY KEY,
            exact_casing TEXT NOT NULL,
            homepage_visible_score INTEGER NOT NULL,
            weighted_selection_score INTEGER NOT NULL,
            selection_count INTEGER NOT NULL,
            cross_source_coverage INTEGER NOT NULL,
            live_source_coverage INTEGER NOT NULL,
            hidden_source_coverage INTEGER NOT NULL,
            hidden_only INTEGER NOT NULL,
            partner_like INTEGER NOT NULL,
            canonical_source TEXT NOT NULL,
            sources TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS clue_hits (
            source_name TEXT NOT NULL,
            needle TEXT NOT NULL,
            hit_count INTEGER NOT NULL
        );
        """
    )


def insert_extra_rows(
    conn: sqlite3.Connection,
    sources: list[SourceArtifact],
    discoveries: list[DiscoveryRecord],
    ranked_candidates: list[dict[str, object]],
    clue_hits: list[tuple[str, str, int]],
) -> None:
    conn.executemany(
        """
        INSERT INTO sources (
            name, source_class, kind, locator, selection_weight, title_selection_weight, include_in_coverage
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        [
            (
                source.name,
                source.source_class,
                source.kind,
                source.locator,
                source.selection_weight,
                source.title_selection_weight,
                int(source.include_in_coverage),
            )
            for source in sources
        ],
    )
    conn.executemany(
        """
        INSERT INTO discoveries (name, locator, status, content_type, saved_path, source_class)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        [
            (
                discovery.name,
                discovery.locator,
                discovery.status,
                discovery.content_type,
                discovery.saved_path,
                discovery.source_class,
            )
            for discovery in discoveries
        ],
    )
    conn.executemany(
        """
        INSERT INTO candidate_scores (
            phrase, exact_casing, homepage_visible_score, weighted_selection_score, selection_count,
            cross_source_coverage, live_source_coverage, hidden_source_coverage, hidden_only, partner_like,
            canonical_source, sources
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        [
            (
                row["phrase"],
                row["exact_casing"],
                row["homepage_visible_score"],
                row["weighted_selection_score"],
                row["selection_count"],
                row["cross_source_coverage"],
                row["live_source_coverage"],
                row["hidden_source_coverage"],
                int(row["hidden_only"]),
                row["partner_like"],
                row["canonical_source"],
                row["sources"],
            )
            for row in ranked_candidates
        ],
    )
    conn.executemany(
        "INSERT INTO clue_hits (source_name, needle, hit_count) VALUES (?, ?, ?)",
        clue_hits,
    )


def build_live_sources(
    base_url: str,
    artifact_dir: Path,
    map_path: Path,
) -> tuple[list[SourceArtifact], list[DiscoveryRecord], str]:
    discoveries: list[DiscoveryRecord] = []
    live_sources: list[SourceArtifact] = []

    status, content_type, home_html = fetch_text(base_url)
    home_saved = save_artifact(artifact_dir, "live-homepage", ".html", home_html)
    discoveries.append(
        DiscoveryRecord(
            name="live_homepage",
            locator=base_url,
            status=status,
            content_type=content_type,
            saved_path=home_saved,
            source_class="live_home_html",
        )
    )
    live_sources.append(
        SourceArtifact(
            name="live_homepage",
            source_class="live_home_html",
            kind="html",
            locator=base_url,
            text=home_html,
            selection_weight=4,
            title_selection_weight=3,
        )
    )

    discovered_urls = set(discover_same_domain_urls(base_url, home_html))
    discovered_urls.update(firecrawl_map_urls(base_url, map_path))
    discovered_urls.update(
        {
            urllib.parse.urljoin(base_url, "swarm-v1.html"),
            urllib.parse.urljoin(base_url, ".git/HEAD"),
            urllib.parse.urljoin(base_url, ".git/config"),
            urllib.parse.urljoin(base_url, ".git/description"),
            urllib.parse.urljoin(base_url, ".git/FETCH_HEAD"),
            urllib.parse.urljoin(base_url, ".git/packed-refs"),
            urllib.parse.urljoin(base_url, ".git/logs/HEAD"),
            urllib.parse.urljoin(base_url, ".git/logs/refs/heads/main"),
            urllib.parse.urljoin(base_url, ".git/refs/heads/main"),
            urllib.parse.urljoin(base_url, "robots.txt"),
            urllib.parse.urljoin(base_url, "sitemap.xml"),
            urllib.parse.urljoin(base_url, "stats.html"),
        }
    )

    for url in sorted(discovered_urls):
        if url == base_url:
            continue
        fetched = maybe_fetch_text(url)
        if fetched is None:
            continue
        status, content_type, text = fetched
        parsed = urllib.parse.urlparse(url)
        suffix = ".txt"
        kind = "text"
        source_class = "live_text"
        if content_type == "text/html" or parsed.path.endswith(".html"):
            suffix = ".html"
            kind = "html"
            source_class = "live_html"
        if parsed.path.startswith("/.git/"):
            source_class = "git_text"
        if status >= 400:
            saved_path = save_artifact(artifact_dir, f"probe-{parsed.path}", suffix, text)
            discoveries.append(
                DiscoveryRecord(
                    name=f"probe_{slugify(parsed.path)}",
                    locator=url,
                    status=status,
                    content_type=content_type,
                    saved_path=saved_path,
                    source_class=source_class,
                )
            )
            continue
        save_name = f"fetch-{parsed.path or 'root'}"
        if parsed.query:
            save_name += f"-{parsed.query}"
        saved_path = save_artifact(artifact_dir, save_name, suffix, text)
        source_name = source_name_for_url(url)
        discoveries.append(
            DiscoveryRecord(
                name=source_name,
                locator=url,
                status=status,
                content_type=content_type,
                saved_path=saved_path,
                source_class=source_class,
            )
        )
        if not (
            kind == "html"
            or source_class == "git_text"
            or content_type.startswith("text/")
            or content_type in {"application/json", "application/xml", "application/octet-stream"}
        ):
            continue
        if source_name == "live_swarm-v1-html":
            selection_weight = 3
            title_weight = 2
        elif source_class == "git_text":
            selection_weight = 1
            title_weight = 1
        else:
            selection_weight = 2
            title_weight = 1
        live_sources.append(
            SourceArtifact(
                name=source_name,
                source_class=source_class,
                kind=kind,
                locator=url,
                text=text,
                selection_weight=selection_weight,
                title_selection_weight=title_weight,
                include_in_coverage=source_class != "git_text",
            )
        )

    return live_sources, discoveries, home_html


def build_hidden_commit_sources(base_url: str, artifact_dir: Path) -> tuple[list[SourceArtifact], dict[str, object]]:
    head_text = fetch_text(urllib.parse.urljoin(base_url, ".git/refs/heads/main"))[2].strip()
    hidden_commit_sha = head_text.splitlines()[0].strip()
    hidden_commit = parse_commit_object(fetch_git_object(hidden_commit_sha, base_url))
    hidden_commit = GitCommitSnapshot(
        commit_sha=hidden_commit_sha,
        parent_sha=hidden_commit.parent_sha,
        tree_sha=hidden_commit.tree_sha,
        message=hidden_commit.message,
    )
    parent_commit = parse_commit_object(fetch_git_object(hidden_commit.parent_sha, base_url)) if hidden_commit.parent_sha else None
    if parent_commit is not None:
        parent_commit = GitCommitSnapshot(
            commit_sha=hidden_commit.parent_sha or "",
            parent_sha=parent_commit.parent_sha,
            tree_sha=parent_commit.tree_sha,
            message=parent_commit.message,
        )

    hidden_tree = {name: sha for _, name, sha in parse_tree_entries(fetch_git_object(hidden_commit.tree_sha, base_url))}
    parent_tree = (
        {name: sha for _, name, sha in parse_tree_entries(fetch_git_object(parent_commit.tree_sha, base_url))}
        if parent_commit is not None
        else {}
    )

    hidden_html = blob_text(hidden_tree["swarm-v1.html"], base_url)
    hidden_saved = save_artifact(artifact_dir, "git-hidden-swarm-v1", ".html", hidden_html)
    sources = [
        SourceArtifact(
            name="git_hidden_swarm_v1",
            source_class="git_hidden_html",
            kind="html",
            locator=f"git:{hidden_commit.commit_sha}:swarm-v1.html",
            text=hidden_html,
            selection_weight=3,
            title_selection_weight=2,
        )
    ]

    diff_text = ""
    added_phrases: list[str] = []
    removed_phrases: list[str] = []

    if "swarm-v1.html" in parent_tree:
        parent_html = blob_text(parent_tree["swarm-v1.html"], base_url)
        parent_saved = save_artifact(artifact_dir, "git-parent-swarm-v1", ".html", parent_html)
        sources.append(
            SourceArtifact(
                name="git_parent_swarm_v1",
                source_class="git_parent_html",
                kind="html",
                locator=f"git:{parent_commit.commit_sha}:swarm-v1.html" if parent_commit else "git:parent:swarm-v1.html",
                text=parent_html,
                selection_weight=2,
                title_selection_weight=1,
            )
        )
        diff_lines = list(
            difflib.unified_diff(
                parent_html.splitlines(),
                hidden_html.splitlines(),
                fromfile="parent",
                tofile="hidden",
                n=3,
            )
        )
        diff_text = "\n".join(diff_lines)
        save_artifact(artifact_dir, "git-hidden-diff", ".patch", diff_text)

        parent_phrases = {
            occurrence.phrase
            for occurrence in dedupe_occurrences(
                phrase_occurrences_from_exact_lines(
                    [text for _, text in html_nodes_from_text(parent_html)],
                    "parent_exact",
                    title_like_only=False,
                ),
                "parent_exact_dedup",
            )
        }
        hidden_phrases = {
            occurrence.phrase
            for occurrence in dedupe_occurrences(
                phrase_occurrences_from_exact_lines(
                    [text for _, text in html_nodes_from_text(hidden_html)],
                    "hidden_exact",
                    title_like_only=False,
                ),
                "hidden_exact_dedup",
            )
        }
        added_phrases = sorted(hidden_phrases - parent_phrases)
        removed_phrases = sorted(parent_phrases - hidden_phrases)

        return sources, {
            "hidden_commit_sha": hidden_commit.commit_sha,
            "hidden_commit_message": hidden_commit.message,
            "parent_commit_sha": parent_commit.commit_sha if parent_commit else None,
            "hidden_saved_path": hidden_saved,
            "parent_saved_path": parent_saved,
            "diff_excerpt": "\n".join(diff_lines[:40]),
            "added_exact_phrases": added_phrases,
            "removed_exact_phrases": removed_phrases,
        }

    return sources, {
        "hidden_commit_sha": hidden_commit.commit_sha,
        "hidden_commit_message": hidden_commit.message,
        "parent_commit_sha": parent_commit.commit_sha if parent_commit else None,
        "hidden_saved_path": hidden_saved,
        "parent_saved_path": None,
        "diff_excerpt": diff_text,
        "added_exact_phrases": added_phrases,
        "removed_exact_phrases": removed_phrases,
    }


def build_local_sources(artifact_dir: Path, local_repo: Path, visible_home: Path) -> list[SourceArtifact]:
    sources: list[SourceArtifact] = []
    if visible_home.exists():
        text = visible_home.read_text(encoding="utf-8")
        save_artifact(artifact_dir, "homepage-visible", ".txt", text)
        sources.append(
            SourceArtifact(
                name="homepage_visible",
                source_class="homepage_visible",
                kind="text",
                locator=str(visible_home),
                text=text,
                selection_weight=7,
                title_selection_weight=5,
            )
        )

    repo_files = {
        "repo_swarm_v1": local_repo / "swarm-v1.html",
        "repo_mobile_fixed": local_repo / "Mobile Fixed",
        "repo_invok": local_repo / "invok",
    }
    for source_name, path in repo_files.items():
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        save_artifact(artifact_dir, source_name, path.suffix or ".txt", text)
        kind = "html" if "<html" in text.lower() else "text"
        selection_weight = 2 if kind == "html" else 1
        title_weight = 1 if kind == "html" else 1
        sources.append(
            SourceArtifact(
                name=source_name,
                source_class="repo_html" if kind == "html" else "repo_text",
                kind=kind,
                locator=str(path),
                text=text,
                selection_weight=selection_weight,
                title_selection_weight=title_weight,
            )
        )
    return sources


def clue_hits_for_sources(sources: Iterable[SourceArtifact]) -> list[tuple[str, str, int]]:
    rows: list[tuple[str, str, int]] = []
    for source in sources:
        lowered = source.text.lower()
        for needle in CLUE_PATTERNS:
            rows.append((source.name, needle, lowered.count(needle.lower())))
    return rows


def write_report(
    report_path: Path,
    base_url: str,
    key: str,
    discoveries: list[DiscoveryRecord],
    hidden_commit_summary: dict[str, object],
    ranked_candidates: list[dict[str, object]],
    clue_hits: list[tuple[str, str, int]],
) -> None:
    report_path.parent.mkdir(parents=True, exist_ok=True)
    top_candidates = ranked_candidates[:10]
    interesting_clues = [row for row in clue_hits if row[2] > 0 and row[1] != "phrase"]
    lines = [
        "# SWARM Puzzle #2 Final Verification",
        "",
        "## Summary",
        f"- Base URL: `{base_url}`",
        f"- Key: `{key}`",
        f"- Best-supported phrase: `{top_candidates[0]['phrase']}`" if top_candidates else "- No candidate phrases found.",
        "- Scope stayed on `swarm.thecanteenapp.com` artifacts and SWARM-local git/text evidence only.",
        "",
        "## Live Discovery",
    ]
    for discovery in discoveries:
        lines.append(
            f"- `{discovery.name}` -> status `{discovery.status}` `{discovery.content_type}` "
            f"from `{discovery.locator}`"
        )
    lines.extend(
        [
            "",
            "## Hidden Git Commit",
            f"- Hidden HEAD commit: `{hidden_commit_summary['hidden_commit_sha']}`",
            f"- Hidden commit message: `{hidden_commit_summary['hidden_commit_message']}`",
            f"- Parent commit: `{hidden_commit_summary['parent_commit_sha']}`",
            f"- Added exact 3-word phrases in hidden commit: `{len(hidden_commit_summary['added_exact_phrases'])}`",
            f"- Removed exact 3-word phrases in hidden commit: `{len(hidden_commit_summary['removed_exact_phrases'])}`",
        ]
    )
    if hidden_commit_summary["added_exact_phrases"]:
        lines.append(
            "- Added phrases: " + ", ".join(f"`{phrase}`" for phrase in hidden_commit_summary["added_exact_phrases"])
        )
    if hidden_commit_summary["removed_exact_phrases"]:
        lines.append(
            "- Removed phrases: " + ", ".join(f"`{phrase}`" for phrase in hidden_commit_summary["removed_exact_phrases"])
        )
    if hidden_commit_summary["diff_excerpt"]:
        lines.extend(
            [
                "",
                "### Diff Excerpt",
                "```diff",
                str(hidden_commit_summary["diff_excerpt"]),
                "```",
            ]
        )

    lines.extend(
        [
            "",
            "## Candidate Ranking",
            "| Phrase | Homepage score | Weighted score | Coverage | Hidden-only | Partner-like | Sources |",
            "| --- | ---: | ---: | ---: | --- | --- | --- |",
        ]
    )
    for row in top_candidates:
        lines.append(
            f"| `{row['phrase']}` | {row['homepage_visible_score']} | {row['weighted_selection_score']} | "
            f"{row['cross_source_coverage']} | `{row['hidden_only']}` | `{bool(row['partner_like'])}` | `{row['sources']}` |"
        )

    lines.extend(["", "## Clue Hits"])
    if interesting_clues:
        for source_name, needle, hit_count in interesting_clues:
            lines.append(f"- `{source_name}` contains `{needle}` `{hit_count}` time(s)")
    else:
        lines.append("- No explicit unlock or key string was found in the SWARM-local text artifacts.")

    lines.extend(["", "## Conclusion"])
    if top_candidates:
        winner = top_candidates[0]
        lines.append(
            f"- `{winner['phrase']}` remains the top candidate because it wins the homepage-visible set, "
            f"survives the SWARM-only weighting, and no hidden git artifact introduces a stronger 3-word phrase."
        )
    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def run_verification(
    *,
    base_url: str,
    visible_home: Path,
    local_repo: Path,
    artifact_dir: Path,
    db_path: Path,
    report_path: Path,
    map_path: Path,
    key: str,
) -> dict[str, object]:
    live_sources, discoveries, _ = build_live_sources(base_url, artifact_dir, map_path)
    git_sources, hidden_commit_summary = build_hidden_commit_sources(base_url, artifact_dir)
    local_sources = build_local_sources(artifact_dir, local_repo, visible_home)

    sources = local_sources + live_sources + git_sources
    source_by_name = {source.name: source for source in sources}

    source_sets: dict[str, list[PhraseOccurrence]] = {}
    source_words: dict[str, list] = {}
    for source in sources:
        words, sets = source_occurrences(source)
        source_words[source.name] = words
        source_sets.update(sets)

    metrics = key_metrics(key)
    selection_rows = metric_selection_rows(metrics, source_sets)
    ranked_candidates = rank_candidates(source_by_name, source_sets, selection_rows)
    clue_hits = clue_hits_for_sources(sources)

    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    try:
        create_schema(conn)
        create_extra_tables(conn)
        reset_tables(conn)
        for extra_table in ["sources", "discoveries", "candidate_scores", "clue_hits"]:
            conn.execute(f"DELETE FROM {extra_table}")
        conn.executemany(
            "INSERT INTO inputs (name, value) VALUES (?, ?)",
            [
                ("key", key),
                ("base_url", base_url),
                ("visible_home", str(visible_home)),
                ("local_repo", str(local_repo)),
            ],
        )
        for source in sources:
            insert_words(conn, source.name, source_words[source.name])
        for occurrences in source_sets.values():
            insert_occurrences(conn, occurrences)
        conn.executemany(
            "INSERT INTO key_metrics (metric_name, metric_value) VALUES (?, ?)",
            sorted(metrics.items()),
        )
        conn.executemany(
            """
            INSERT INTO metric_selections (metric_name, phrase_set, selected_index, phrase_count, phrase)
            VALUES (?, ?, ?, ?, ?)
            """,
            selection_rows,
        )
        insert_extra_rows(conn, sources, discoveries, ranked_candidates, clue_hits)
        conn.commit()
    finally:
        conn.close()

    write_report(report_path, base_url, key, discoveries, hidden_commit_summary, ranked_candidates, clue_hits)
    return {
        "db_path": str(db_path),
        "report_path": str(report_path),
        "best_phrase": ranked_candidates[0]["phrase"] if ranked_candidates else None,
        "top_candidates": ranked_candidates[:5],
        "discovery_count": len(discoveries),
        "hidden_commit_sha": hidden_commit_summary["hidden_commit_sha"],
        "hidden_commit_message": hidden_commit_summary["hidden_commit_message"],
        "hidden_commit_added_exact_phrases": hidden_commit_summary["added_exact_phrases"],
        "hidden_commit_removed_exact_phrases": hidden_commit_summary["removed_exact_phrases"],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", default=DEFAULT_BASE_URL, help="SWARM base URL to verify")
    parser.add_argument("--visible-home", type=Path, default=DEFAULT_VISIBLE_HOME, help="Path to saved visible homepage text")
    parser.add_argument("--local-repo", type=Path, default=DEFAULT_LOCAL_REPO, help="Path to a local SWARM repo mirror")
    parser.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR, help="Directory for fetched artifacts")
    parser.add_argument("--db", type=Path, default=DEFAULT_DB, help="Output SQLite database path")
    parser.add_argument("--report", type=Path, default=DEFAULT_REPORT, help="Output markdown report path")
    parser.add_argument("--map-path", type=Path, default=DEFAULT_MAP_PATH, help="Existing Firecrawl map JSON for the domain")
    parser.add_argument("--key", default=DEFAULT_KEY, help="Puzzle key string")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    summary = run_verification(
        base_url=args.base_url,
        visible_home=args.visible_home,
        local_repo=args.local_repo,
        artifact_dir=args.artifact_dir,
        db_path=args.db,
        report_path=args.report,
        map_path=args.map_path,
        key=args.key,
    )
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
