#!/usr/bin/env python3
"""Algorithmically search SWARM-local site content for a key-unlocked 3-word phrase."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import urllib.parse
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from swarm_phrase_index import (
    DEFAULT_KEY,
    WORD_RE,
    PhraseOccurrence,
    create_schema,
    dedupe_occurrences,
    html_nodes_from_text,
    insert_occurrences,
    insert_words,
    normalize_phrase,
    phrase_occurrences_from_exact_lines,
    phrase_occurrences_from_words,
    reset_tables,
    words_from_lines,
    words_from_nodes,
)
from swarm_phrase_verify import (
    DEFAULT_ARTIFACT_DIR,
    DEFAULT_BASE_URL,
    DEFAULT_LOCAL_REPO,
    DEFAULT_MAP_PATH,
    DEFAULT_VISIBLE_HOME,
    PARTNER_LIKE_TERMS,
    SourceArtifact,
    build_hidden_commit_sources,
    build_live_sources,
    build_local_sources,
)


DEFAULT_OUTPUT_DIR = Path("/Users/jonathon/5dlabs/cto/output/swarm_unlock_search")
DEFAULT_DB = DEFAULT_OUTPUT_DIR / "swarm_unlock_search.sqlite"
DEFAULT_REPORT = DEFAULT_OUTPUT_DIR / "swarm_unlock_search.md"
HTML_COMMENT_RE = re.compile(r"<!--([\s\S]*?)-->")
SCRIPT_RE = re.compile(r"<script\b[^>]*>([\s\S]*?)</script>", re.IGNORECASE)
URL_ATTR_RE = re.compile(r"""(?:href|src|action)=["']([^"'#]+)["']""", re.IGNORECASE)
ID_ATTR_RE = re.compile(r"""id=["']([^"']+)["']""", re.IGNORECASE)
CLASS_ATTR_RE = re.compile(r"""class=["']([^"']+)["']""", re.IGNORECASE)
STRING_RE = re.compile(r"""(?:"([^"\\]*(?:\\.[^"\\]*)*)"|'([^'\\]*(?:\\.[^'\\]*)*)')""")


@dataclass(frozen=True)
class CorpusSpec:
    name: str
    source_name: str
    corpus_kind: str
    lines: list[str]
    title_like_only: bool = False


def key_materials(key: str) -> dict[str, list[int]]:
    from swarm_phrase_index import base58_decode, base58_digit_values

    decoded = base58_decode(key)
    digits = base58_digit_values(key)
    return {
        "base58_digits": digits,
        "base58_digits_rev": list(reversed(digits)),
        "decoded_bytes": list(decoded),
        "decoded_bytes_rev": list(reversed(decoded)),
        "decoded_sha1": list(hashlib.sha1(decoded).digest()),
        "decoded_sha256": list(hashlib.sha256(decoded).digest()),
        "key_sha1": list(hashlib.sha1(key.encode("utf-8")).digest()),
        "key_sha256": list(hashlib.sha256(key.encode("utf-8")).digest()),
    }


def normalize_lines(text: str) -> list[str]:
    return [line.strip() for line in text.splitlines() if line.strip()]


def script_strings(text: str) -> list[str]:
    strings: list[str] = []
    for script in SCRIPT_RE.findall(text):
        for double, single in STRING_RE.findall(script):
            value = bytes((double or single), "utf-8").decode("unicode_escape")
            value = " ".join(value.split())
            if value:
                strings.append(value)
    return strings


def html_comments(text: str) -> list[str]:
    return [" ".join(comment.split()) for comment in HTML_COMMENT_RE.findall(text) if comment.strip()]


def html_attrs(text: str, base_url: str) -> list[str]:
    pieces: list[str] = []
    for item in ID_ATTR_RE.findall(text):
        pieces.append(item)
    for class_blob in CLASS_ATTR_RE.findall(text):
        pieces.extend(token for token in class_blob.split() if token)
    for raw_url in URL_ATTR_RE.findall(text):
        resolved = urllib.parse.urljoin(base_url, raw_url)
        parsed = urllib.parse.urlparse(resolved)
        line_parts = [parsed.netloc]
        line_parts.extend(bit for bit in parsed.path.split("/") if bit)
        if parsed.query:
            for key, values in urllib.parse.parse_qs(parsed.query).items():
                line_parts.append(key)
                line_parts.extend(values)
        pieces.append(" ".join(line_parts))
    return pieces


def build_corpora_for_source(source: SourceArtifact, base_url: str) -> list[CorpusSpec]:
    corpora: list[CorpusSpec] = []
    if source.kind == "html":
        visible_lines = [text for _, text in html_nodes_from_text(source.text)]
        corpora.append(CorpusSpec(f"{source.name}__visible_lines", source.name, "visible_lines", visible_lines))
        corpora.append(
            CorpusSpec(
                f"{source.name}__visible_title_lines",
                source.name,
                "visible_title_lines",
                visible_lines,
                title_like_only=True,
            )
        )
        comment_lines = html_comments(source.text)
        if comment_lines:
            corpora.append(CorpusSpec(f"{source.name}__comments", source.name, "comments", comment_lines))
        attr_lines = html_attrs(source.text, base_url)
        if attr_lines:
            corpora.append(CorpusSpec(f"{source.name}__attrs", source.name, "attrs", attr_lines))
        script_lines = script_strings(source.text)
        if script_lines:
            corpora.append(CorpusSpec(f"{source.name}__script_strings", source.name, "script_strings", script_lines))
        corpora.append(CorpusSpec(f"{source.name}__raw_lines", source.name, "raw_lines", normalize_lines(source.text)))
    else:
        lines = normalize_lines(source.text)
        corpora.append(CorpusSpec(f"{source.name}__text_lines", source.name, "text_lines", lines))
    return corpora


def corpus_words(corpus: CorpusSpec) -> list:
    return words_from_lines(corpus.lines)


def corpus_phrase_sets(corpus: CorpusSpec) -> dict[str, list[PhraseOccurrence]]:
    words = corpus_words(corpus)
    phrase_sets: dict[str, list[PhraseOccurrence]] = {}
    trigram_name = f"{corpus.name}__trigrams"
    exact_name = f"{corpus.name}__exact_3words"
    phrase_sets[trigram_name] = phrase_occurrences_from_words(words, trigram_name, title_like_only=corpus.title_like_only)
    phrase_sets[exact_name] = phrase_occurrences_from_exact_lines(corpus.lines, exact_name, title_like_only=corpus.title_like_only)
    deduped = {}
    for set_name, occurrences in phrase_sets.items():
        deduped[f"{set_name}_dedup"] = dedupe_occurrences(occurrences, f"{set_name}_dedup")
    phrase_sets.update(deduped)
    return phrase_sets


def partner_like(phrase: str) -> bool:
    normalized_tokens = normalize_phrase(phrase).split()
    return any(token in PARTNER_LIKE_TERMS for token in normalized_tokens)


def sequence_windows(values: list[int], width: int) -> Iterable[list[int]]:
    if len(values) < width:
        return []
    return (values[index : index + width] for index in range(len(values) - width + 1))


def selected_words_from_values(words: list, values: list[int], mode: str) -> list[str]:
    if not words or len(words) < 3 or not values:
        return []
    word_count = len(words)
    selected: list[str] = []
    if mode == "direct":
        for value in values:
            selected.append(words[value % word_count].token)
    elif mode == "prefix_sum":
        total = 0
        for value in values:
            total += value
            selected.append(words[total % word_count].token)
    elif mode == "pair_sum":
        for pair in sequence_windows(values, 2):
            idx = (pair[0] << 8 | pair[1]) % word_count
            selected.append(words[idx].token)
    elif mode == "triple_sum":
        for triple in sequence_windows(values, 3):
            idx = ((triple[0] << 16) | (triple[1] << 8) | triple[2]) % word_count
            selected.append(words[idx].token)
    return selected


def selected_phrases(words: list[str]) -> list[str]:
    phrases: list[str] = []
    for index in range(len(words) - 2):
        phrases.append(" ".join(words[index : index + 3]))
    return phrases


def create_unlock_tables(conn: sqlite3.Connection) -> None:
    conn.executescript(
        """
        CREATE TABLE IF NOT EXISTS unlock_sources (
            name TEXT PRIMARY KEY,
            source_class TEXT NOT NULL,
            kind TEXT NOT NULL,
            locator TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS unlock_corpora (
            name TEXT PRIMARY KEY,
            source_name TEXT NOT NULL,
            corpus_kind TEXT NOT NULL,
            line_count INTEGER NOT NULL,
            word_count INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS unlock_phrase_presence (
            corpus_name TEXT NOT NULL,
            phrase_set TEXT NOT NULL,
            phrase TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS unlock_method_hits (
            material_name TEXT NOT NULL,
            mode TEXT NOT NULL,
            corpus_name TEXT NOT NULL,
            phrase TEXT NOT NULL,
            hit_count INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS unlock_candidate_scores (
            phrase TEXT PRIMARY KEY,
            normalized_phrase TEXT NOT NULL,
            total_method_hits INTEGER NOT NULL,
            distinct_methods INTEGER NOT NULL,
            corpus_coverage INTEGER NOT NULL,
            source_coverage INTEGER NOT NULL,
            homepage_exact INTEGER NOT NULL,
            homepage_trigram INTEGER NOT NULL,
            live_visible INTEGER NOT NULL,
            partner_like INTEGER NOT NULL
        );
        """
    )


def write_report(
    report_path: Path,
    key: str,
    summary: dict[str, object],
    ranked_candidates: list[tuple],
) -> None:
    report_path.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# SWARM Unlock Search",
        "",
        "## Summary",
        f"- Key: `{key}`",
        f"- Source count: `{summary['source_count']}`",
        f"- Corpus count: `{summary['corpus_count']}`",
        f"- Phrase universe: `{summary['phrase_count']}` distinct trigrams/exact phrases",
        f"- Unlock hits: `{summary['unlock_hit_count']}` matched method hits",
        "",
        "## Top Candidates",
        "| Phrase | Method hits | Distinct methods | Corpus coverage | Source coverage | Homepage exact | Homepage trigram | Live visible | Partner-like |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
    ]
    for row in ranked_candidates[:20]:
        (
            phrase,
            total_method_hits,
            distinct_methods,
            corpus_coverage,
            source_coverage,
            homepage_exact,
            homepage_trigram,
            live_visible,
            is_partner_like,
        ) = row
        lines.append(
            f"| `{phrase}` | {total_method_hits} | {distinct_methods} | {corpus_coverage} | {source_coverage} | "
            f"{homepage_exact} | {homepage_trigram} | {live_visible} | `{bool(is_partner_like)}` |"
        )
    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def run_search(
    *,
    base_url: str,
    key: str,
    visible_home: Path,
    local_repo: Path,
    artifact_dir: Path,
    map_path: Path,
    db_path: Path,
    report_path: Path,
) -> dict[str, object]:
    live_sources, _, _ = build_live_sources(base_url, artifact_dir, map_path)
    git_sources, _ = build_hidden_commit_sources(base_url, artifact_dir)
    local_sources = build_local_sources(artifact_dir, local_repo, visible_home)
    sources = local_sources + live_sources + git_sources

    corpora: list[CorpusSpec] = []
    for source in sources:
        corpora.extend(build_corpora_for_source(source, base_url))

    phrase_universe: dict[str, set[str]] = {}
    source_presence: dict[str, set[str]] = {}
    homepage_exact: set[str] = set()
    homepage_trigram: set[str] = set()
    live_visible: set[str] = set()
    corpus_word_cache: dict[str, list] = {}
    phrase_presence_rows: list[tuple[str, str, str]] = []
    phrase_sets_by_corpus: dict[str, dict[str, list[PhraseOccurrence]]] = {}

    for corpus in corpora:
        words = corpus_words(corpus)
        corpus_word_cache[corpus.name] = words
        phrase_sets = corpus_phrase_sets(corpus)
        phrase_sets_by_corpus[corpus.name] = phrase_sets
        for set_name, occurrences in phrase_sets.items():
            if not set_name.endswith("_dedup"):
                continue
            for occurrence in occurrences:
                phrase_presence_rows.append((corpus.name, set_name, occurrence.phrase))
                phrase_universe.setdefault(occurrence.phrase, set()).add(corpus.name)
                source_presence.setdefault(occurrence.phrase, set()).add(corpus.source_name)
                if corpus.name == "homepage_visible__text_lines" and set_name.endswith("__exact_3words_dedup"):
                    homepage_exact.add(occurrence.phrase)
                if corpus.name == "homepage_visible__text_lines" and set_name.endswith("__trigrams_dedup"):
                    homepage_trigram.add(occurrence.phrase)
                if corpus.corpus_kind in {"visible_lines", "visible_title_lines"} and corpus.source_name.startswith("live"):
                    live_visible.add(occurrence.phrase)

    materials = key_materials(key)
    method_hits: dict[tuple[str, str, str, str], int] = {}
    candidate_totals: dict[str, int] = {}
    candidate_methods: dict[str, set[str]] = {}

    for material_name, values in materials.items():
        for mode in ["direct", "prefix_sum", "pair_sum", "triple_sum"]:
            for corpus in corpora:
                words = corpus_word_cache[corpus.name]
                selected = selected_words_from_values(words, values, mode)
                for phrase in selected_phrases(selected):
                    if phrase not in phrase_universe:
                        continue
                    key_tuple = (material_name, mode, corpus.name, phrase)
                    method_hits[key_tuple] = method_hits.get(key_tuple, 0) + 1
                    candidate_totals[phrase] = candidate_totals.get(phrase, 0) + 1
                    candidate_methods.setdefault(phrase, set()).add(f"{material_name}:{mode}")

    ranked_rows: list[tuple] = []
    for phrase, total_hits in candidate_totals.items():
        ranked_rows.append(
            (
                phrase,
                total_hits,
                len(candidate_methods.get(phrase, set())),
                len(phrase_universe.get(phrase, set())),
                len(source_presence.get(phrase, set())),
                int(phrase in homepage_exact),
                int(phrase in homepage_trigram),
                int(phrase in live_visible),
                int(partner_like(phrase)),
            )
        )

    ranked_rows.sort(
        key=lambda row: (
            -row[1],
            -row[2],
            -row[5],
            -row[6],
            -row[7],
            row[8],
            -row[3],
            -row[4],
            row[0],
        )
    )

    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    try:
        create_schema(conn)
        create_unlock_tables(conn)
        reset_tables(conn)
        for table in [
            "unlock_sources",
            "unlock_corpora",
            "unlock_phrase_presence",
            "unlock_method_hits",
            "unlock_candidate_scores",
        ]:
            conn.execute(f"DELETE FROM {table}")
        conn.executemany(
            "INSERT INTO inputs (name, value) VALUES (?, ?)",
            [
                ("key", key),
                ("base_url", base_url),
                ("visible_home", str(visible_home)),
                ("local_repo", str(local_repo)),
            ],
        )
        conn.executemany(
            "INSERT INTO unlock_sources (name, source_class, kind, locator) VALUES (?, ?, ?, ?)",
            [(source.name, source.source_class, source.kind, source.locator) for source in sources],
        )
        conn.executemany(
            "INSERT INTO unlock_corpora (name, source_name, corpus_kind, line_count, word_count) VALUES (?, ?, ?, ?, ?)",
            [
                (
                    corpus.name,
                    corpus.source_name,
                    corpus.corpus_kind,
                    len(corpus.lines),
                    len(corpus_word_cache[corpus.name]),
                )
                for corpus in corpora
            ],
        )
        for source in sources:
            insert_words(conn, source.name, corpus_word_cache.get(f"{source.name}__text_lines", []))
        all_occurrences: list[PhraseOccurrence] = []
        for phrase_sets in phrase_sets_by_corpus.values():
            for occurrences in phrase_sets.values():
                all_occurrences.extend(occurrences)
        insert_occurrences(conn, all_occurrences)
        conn.executemany(
            "INSERT INTO unlock_phrase_presence (corpus_name, phrase_set, phrase) VALUES (?, ?, ?)",
            phrase_presence_rows,
        )
        conn.executemany(
            "INSERT INTO unlock_method_hits (material_name, mode, corpus_name, phrase, hit_count) VALUES (?, ?, ?, ?, ?)",
            [(*key_tuple, hit_count) for key_tuple, hit_count in method_hits.items()],
        )
        conn.executemany(
            """
            INSERT INTO unlock_candidate_scores (
                phrase, normalized_phrase, total_method_hits, distinct_methods, corpus_coverage,
                source_coverage, homepage_exact, homepage_trigram, live_visible, partner_like
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            [(row[0], normalize_phrase(row[0]), *row[1:]) for row in ranked_rows],
        )
        conn.commit()
    finally:
        conn.close()

    summary = {
        "db_path": str(db_path),
        "report_path": str(report_path),
        "source_count": len(sources),
        "corpus_count": len(corpora),
        "phrase_count": len(phrase_universe),
        "unlock_hit_count": sum(candidate_totals.values()),
        "top_candidates": [
            {
                "phrase": row[0],
                "total_method_hits": row[1],
                "distinct_methods": row[2],
                "corpus_coverage": row[3],
                "source_coverage": row[4],
                "homepage_exact": bool(row[5]),
                "homepage_trigram": bool(row[6]),
                "live_visible": bool(row[7]),
                "partner_like": bool(row[8]),
            }
            for row in ranked_rows[:15]
        ],
    }
    write_report(report_path, key, summary, ranked_rows)
    return summary


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", default=DEFAULT_BASE_URL)
    parser.add_argument("--key", default=DEFAULT_KEY)
    parser.add_argument("--visible-home", type=Path, default=DEFAULT_VISIBLE_HOME)
    parser.add_argument("--local-repo", type=Path, default=DEFAULT_LOCAL_REPO)
    parser.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    parser.add_argument("--map-path", type=Path, default=DEFAULT_MAP_PATH)
    parser.add_argument("--db", type=Path, default=DEFAULT_DB)
    parser.add_argument("--report", type=Path, default=DEFAULT_REPORT)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    summary = run_search(
        base_url=args.base_url,
        key=args.key,
        visible_home=args.visible_home,
        local_repo=args.local_repo,
        artifact_dir=args.artifact_dir,
        map_path=args.map_path,
        db_path=args.db,
        report_path=args.report,
    )
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
