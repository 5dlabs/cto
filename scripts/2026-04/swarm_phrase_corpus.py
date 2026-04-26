#!/usr/bin/env python3
"""Index multiple SWARM site artifacts into SQLite and rank phrase candidates."""

from __future__ import annotations

import argparse
import json
import sqlite3
from dataclasses import dataclass
from pathlib import Path

from swarm_phrase_index import (
    DEFAULT_KEY,
    PhraseOccurrence,
    dedupe_occurrences,
    html_nodes_from_text,
    insert_occurrences,
    insert_words,
    key_metrics,
    load_text,
    metric_selection_rows,
    phrase_occurrences_from_exact_lines,
    phrase_occurrences_from_words,
    reset_tables,
    words_from_lines,
    words_from_nodes,
    create_schema,
)


DEFAULT_DB = Path("/Users/jonathon/5dlabs/cto/output/swarm_phrase_index/swarm_puzzle_2_corpus.sqlite")


@dataclass(frozen=True)
class SourceSpec:
    name: str
    path: Path
    kind: str


DEFAULT_SOURCES = [
    SourceSpec("visible_live", Path("/tmp/swarm-visible-flat.txt"), "text"),
    SourceSpec("html_live", Path("/tmp/live-swarm-v1.html"), "html"),
    SourceSpec("html_hidden", Path("/tmp/hidden-swarm-v1.html"), "html"),
    SourceSpec("html_repo_main", Path("/tmp/SWARM-V1-website/swarm-v1.html"), "html"),
    SourceSpec("html_mobile_fixed", Path("/tmp/SWARM-V1-website/Mobile Fixed"), "html"),
]


def normalize_source_name(name: str) -> str:
    return name.replace("-", "_")


def prefixed_set_name(source_name: str, suffix: str) -> str:
    return f"{normalize_source_name(source_name)}__{suffix}"


def source_occurrences(source: SourceSpec) -> tuple[list, dict[str, list[PhraseOccurrence]]]:
    raw = load_text(source.path)
    if source.kind == "html":
        nodes = html_nodes_from_text(raw)
        words = words_from_nodes(nodes)
        exact_lines = [text for _, text in nodes]
    else:
        exact_lines = [line for line in raw.splitlines() if line.strip()]
        words = words_from_lines(exact_lines)

    sets: dict[str, list[PhraseOccurrence]] = {}
    sets[prefixed_set_name(source.name, "all_trigrams")] = phrase_occurrences_from_words(
        words,
        prefixed_set_name(source.name, "all_trigrams"),
        title_like_only=False,
    )
    sets[prefixed_set_name(source.name, "title_trigrams")] = phrase_occurrences_from_words(
        words,
        prefixed_set_name(source.name, "title_trigrams"),
        title_like_only=True,
    )
    sets[prefixed_set_name(source.name, "exact_line_3words")] = phrase_occurrences_from_exact_lines(
        exact_lines,
        prefixed_set_name(source.name, "exact_line_3words"),
        title_like_only=False,
    )
    sets[prefixed_set_name(source.name, "title_exact_line_3words")] = phrase_occurrences_from_exact_lines(
        exact_lines,
        prefixed_set_name(source.name, "title_exact_line_3words"),
        title_like_only=True,
    )

    deduped: dict[str, list[PhraseOccurrence]] = {}
    for set_name, occurrences in sets.items():
        deduped_name = f"{set_name}_dedup"
        deduped[deduped_name] = dedupe_occurrences(occurrences, deduped_name)
    sets.update(deduped)
    return words, sets


def aggregate_global_exact_sets(source_sets: dict[str, list[PhraseOccurrence]]) -> dict[str, list[PhraseOccurrence]]:
    global_exact: list[PhraseOccurrence] = []
    global_title_exact: list[PhraseOccurrence] = []
    for set_name, occurrences in source_sets.items():
        if set_name.endswith("__exact_line_3words_dedup"):
            global_exact.extend(occurrences)
        if set_name.endswith("__title_exact_line_3words_dedup"):
            global_title_exact.extend(occurrences)
    return {
        "global__exact_line_3words_dedup": dedupe_occurrences(global_exact, "global__exact_line_3words_dedup"),
        "global__title_exact_line_3words_dedup": dedupe_occurrences(
            global_title_exact,
            "global__title_exact_line_3words_dedup",
        ),
    }


def phrase_source_coverage_rows(source_sets: dict[str, list[PhraseOccurrence]]) -> list[tuple[str, int, str]]:
    coverage: dict[str, set[str]] = {}
    for set_name, occurrences in source_sets.items():
        if not set_name.endswith("__exact_line_3words_dedup"):
            continue
        source_name = set_name.split("__", 1)[0]
        for occurrence in occurrences:
            coverage.setdefault(occurrence.phrase, set()).add(source_name)
    return [
        (phrase, len(source_names), ", ".join(sorted(source_names)))
        for phrase, source_names in sorted(coverage.items(), key=lambda item: (-len(item[1]), item[0]))
    ]


def write_report(
    path: Path,
    key: str,
    sources: list[SourceSpec],
    metrics: dict[str, str],
    source_sets: dict[str, list[PhraseOccurrence]],
    coverage_rows: list[tuple[str, int, str]],
    selection_rows: list[tuple[str, str, int, int, str]],
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# SWARM Puzzle Phrase Corpus",
        "",
        "## Key",
        f"- `{key}`",
        "",
        "## Sources",
    ]
    for source in sources:
        lines.append(f"- `{source.name}` (`{source.kind}`): `{source.path}`")
    lines.append("")
    lines.append("## Key Metrics")
    for name, value in metrics.items():
        lines.append(f"- `{name}`: `{value}`")
    lines.append("")
    lines.append("## Exact 3-Word Phrase Coverage")
    for phrase, source_count, source_names in coverage_rows[:50]:
        lines.append(f"- `{phrase}` -> {source_count} source(s): {source_names}")
    lines.append("")
    lines.append("## Global Exact-Line Selections")
    for metric_name, phrase_set, selected_index, phrase_count, phrase in selection_rows:
        if phrase_set not in {"global__exact_line_3words_dedup", "global__title_exact_line_3words_dedup"}:
            continue
        lines.append(
            f"- `{metric_name}` on `{phrase_set}` -> `{phrase}` "
            f"(index `{selected_index}` of `{phrase_count}`)"
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def build_corpus_index(sources: list[SourceSpec], db_path: Path, key: str, report_path: Path) -> dict[str, object]:
    source_sets: dict[str, list[PhraseOccurrence]] = {}
    source_words: dict[str, list] = {}
    for source in sources:
        words, sets = source_occurrences(source)
        source_words[source.name] = words
        source_sets.update(sets)

    source_sets.update(aggregate_global_exact_sets(source_sets))
    metrics = key_metrics(key)
    selection_rows = metric_selection_rows(metrics, source_sets)
    coverage_rows = phrase_source_coverage_rows(source_sets)

    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    try:
        create_schema(conn)
        reset_tables(conn)
        conn.executemany(
            "INSERT INTO inputs (name, value) VALUES (?, ?)",
            [("key", key)] + [(f"source::{source.name}", str(source.path)) for source in sources],
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
        conn.commit()
    finally:
        conn.close()

    write_report(report_path, key, sources, metrics, source_sets, coverage_rows, selection_rows)

    return {
        "db_path": str(db_path),
        "report_path": str(report_path),
        "source_count": len(sources),
        "top_coverage": coverage_rows[:15],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", type=Path, default=DEFAULT_DB, help="Output SQLite database path")
    parser.add_argument("--report", type=Path, default=DEFAULT_DB.with_suffix(".md"), help="Output markdown report path")
    parser.add_argument("--key", default=DEFAULT_KEY, help="Puzzle key string")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    summary = build_corpus_index(DEFAULT_SOURCES, args.db, args.key, args.report)
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
