#!/usr/bin/env python3
"""Index SWARM puzzle site phrases into SQLite and score candidate selections."""

from __future__ import annotations

import argparse
import hashlib
import html
import json
import re
import sqlite3
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path
from typing import Iterable


BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
DEFAULT_KEY = "7GysZCDoTdz1ECULFnqMpZ3CDnHMziN978u6mXAUYkQk"
DEFAULT_HTML = Path("/tmp/live-swarm-v1.html")
DEFAULT_VISIBLE = Path("/tmp/swarm-visible-flat.txt")
DEFAULT_DB = Path("/Users/jonathon/5dlabs/cto/output/swarm_phrase_index/swarm_puzzle_2.sqlite")
WORD_RE = re.compile(r"[A-Za-z0-9$]+(?:(?:-|['’])[A-Za-z0-9$]+)*")
SCRIPT_STYLE_RE = re.compile(r"<(script|style)\b[^>]*>.*?</\1>", re.IGNORECASE | re.DOTALL)


@dataclass(frozen=True)
class Word:
    token: str
    normalized: str
    line_no: int
    node_index: int
    ordinal: int


@dataclass(frozen=True)
class PhraseOccurrence:
    phrase_set: str
    phrase_index: int
    phrase: str
    normalized_phrase: str
    word1: str
    word2: str
    word3: str
    line_no: int | None
    word_start: int | None
    node_index: int | None
    caps_score: int
    exact_line: int
    context_before: str
    context_after: str


class TextNodeCollector(HTMLParser):
    """Collect visible-ish text nodes from HTML without third-party deps."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.nodes: list[tuple[int, str]] = []
        self._node_index = 0

    def handle_data(self, data: str) -> None:
        text = " ".join(data.split())
        if text:
            self.nodes.append((self._node_index, text))
            self._node_index += 1


def normalize_phrase(value: str) -> str:
    tokens = WORD_RE.findall(value)
    return " ".join(token.lower() for token in tokens)


def is_capitalized_token(token: str) -> bool:
    return token.isupper() or token[0].isupper()


def base58_decode(value: str) -> bytes:
    total = 0
    for char in value:
        total *= 58
        total += BASE58_ALPHABET.index(char)
    leading_zeros = len(value) - len(value.lstrip("1"))
    body = total.to_bytes((total.bit_length() + 7) // 8, "big") if total else b""
    return (b"\x00" * leading_zeros) + body


def base58_digit_values(value: str) -> list[int]:
    return [BASE58_ALPHABET.index(char) for char in value]


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def html_nodes_from_text(raw_html: str) -> list[tuple[int, str]]:
    cleaned = SCRIPT_STYLE_RE.sub(" ", raw_html)
    collector = TextNodeCollector()
    collector.feed(cleaned)
    return collector.nodes


def words_from_nodes(nodes: Iterable[tuple[int, str]]) -> list[Word]:
    words: list[Word] = []
    ordinal = 0
    for line_no, (node_index, text) in enumerate(nodes, start=1):
        for token in WORD_RE.findall(text):
            words.append(
                Word(
                    token=token,
                    normalized=token.lower(),
                    line_no=line_no,
                    node_index=node_index,
                    ordinal=ordinal,
                )
            )
            ordinal += 1
    return words


def words_from_lines(lines: Iterable[str]) -> list[Word]:
    words: list[Word] = []
    ordinal = 0
    for line_no, raw_line in enumerate(lines, start=1):
        line = " ".join(raw_line.split())
        if not line:
            continue
        for token in WORD_RE.findall(line):
            words.append(
                Word(
                    token=token,
                    normalized=token.lower(),
                    line_no=line_no,
                    node_index=line_no,
                    ordinal=ordinal,
                )
            )
            ordinal += 1
    return words


def phrase_occurrences_from_words(words: list[Word], phrase_set: str, *, title_like_only: bool) -> list[PhraseOccurrence]:
    results: list[PhraseOccurrence] = []
    for start in range(len(words) - 2):
        triplet = words[start : start + 3]
        caps_score = sum(1 for word in triplet if is_capitalized_token(word.token))
        if title_like_only and caps_score < 2:
            continue
        before = " ".join(word.token for word in words[max(0, start - 3) : start])
        after = " ".join(word.token for word in words[start + 3 : start + 6])
        phrase = " ".join(word.token for word in triplet)
        results.append(
            PhraseOccurrence(
                phrase_set=phrase_set,
                phrase_index=len(results),
                phrase=phrase,
                normalized_phrase=" ".join(word.normalized for word in triplet),
                word1=triplet[0].token,
                word2=triplet[1].token,
                word3=triplet[2].token,
                line_no=triplet[0].line_no,
                word_start=triplet[0].ordinal,
                node_index=triplet[0].node_index,
                caps_score=caps_score,
                exact_line=0,
                context_before=before,
                context_after=after,
            )
        )
    return results


def phrase_occurrences_from_exact_lines(lines: Iterable[str], phrase_set: str, *, title_like_only: bool) -> list[PhraseOccurrence]:
    results: list[PhraseOccurrence] = []
    for line_no, raw_line in enumerate(lines, start=1):
        line = " ".join(raw_line.split())
        if not line:
            continue
        tokens = WORD_RE.findall(line)
        if len(tokens) != 3:
            continue
        caps_score = sum(1 for token in tokens if is_capitalized_token(token))
        if title_like_only and caps_score < 2:
            continue
        results.append(
            PhraseOccurrence(
                phrase_set=phrase_set,
                phrase_index=len(results),
                phrase=" ".join(tokens),
                normalized_phrase=" ".join(token.lower() for token in tokens),
                word1=tokens[0],
                word2=tokens[1],
                word3=tokens[2],
                line_no=line_no,
                word_start=None,
                node_index=line_no,
                caps_score=caps_score,
                exact_line=1,
                context_before="",
                context_after="",
            )
        )
    return results


def dedupe_occurrences(occurrences: list[PhraseOccurrence], deduped_set_name: str) -> list[PhraseOccurrence]:
    seen: set[str] = set()
    deduped: list[PhraseOccurrence] = []
    for occurrence in occurrences:
        key = occurrence.phrase
        if key in seen:
            continue
        seen.add(key)
        deduped.append(
            PhraseOccurrence(
                phrase_set=deduped_set_name,
                phrase_index=len(deduped),
                phrase=occurrence.phrase,
                normalized_phrase=occurrence.normalized_phrase,
                word1=occurrence.word1,
                word2=occurrence.word2,
                word3=occurrence.word3,
                line_no=occurrence.line_no,
                word_start=occurrence.word_start,
                node_index=occurrence.node_index,
                caps_score=occurrence.caps_score,
                exact_line=occurrence.exact_line,
                context_before=occurrence.context_before,
                context_after=occurrence.context_after,
            )
        )
    return deduped


def key_metrics(key: str) -> dict[str, str]:
    decoded = base58_decode(key)
    digits = base58_digit_values(key)
    metrics: dict[str, int] = {
        "key_length": len(key),
        "ascii_sum": sum(ord(char) for char in key),
        "ascii_weighted_sum": sum((index + 1) * ord(char) for index, char in enumerate(key)),
        "base58_digit_sum": sum(digits),
        "base58_digit_weighted_sum": sum((index + 1) * value for index, value in enumerate(digits)),
        "decoded_byte_sum": sum(decoded),
        "decoded_big": int.from_bytes(decoded, "big"),
        "decoded_little": int.from_bytes(decoded, "little"),
        "decoded_sha1_big": int.from_bytes(hashlib.sha1(decoded).digest(), "big"),
        "decoded_sha256_big": int.from_bytes(hashlib.sha256(decoded).digest(), "big"),
        "key_sha1_big": int.from_bytes(hashlib.sha1(key.encode("utf-8")).digest(), "big"),
        "key_sha256_big": int.from_bytes(hashlib.sha256(key.encode("utf-8")).digest(), "big"),
    }
    return {name: str(value) for name, value in metrics.items()}


def metric_selection_rows(
    metrics: dict[str, str],
    phrase_sets: dict[str, list[PhraseOccurrence]],
) -> list[tuple[str, str, int, int, str]]:
    rows: list[tuple[str, str, int, int, str]] = []
    for metric_name, value_text in metrics.items():
        value = int(value_text)
        for phrase_set_name, occurrences in phrase_sets.items():
            if not occurrences:
                continue
            idx = value % len(occurrences)
            occurrence = occurrences[idx]
            rows.append((metric_name, phrase_set_name, idx, len(occurrences), occurrence.phrase))
    return rows


def create_schema(conn: sqlite3.Connection) -> None:
    conn.executescript(
        """
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS inputs (
            name TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS words (
            source_name TEXT NOT NULL,
            ordinal INTEGER NOT NULL,
            line_no INTEGER NOT NULL,
            node_index INTEGER NOT NULL,
            token TEXT NOT NULL,
            normalized_token TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS phrase_occurrences (
            phrase_set TEXT NOT NULL,
            phrase_index INTEGER NOT NULL,
            phrase TEXT NOT NULL,
            normalized_phrase TEXT NOT NULL,
            word1 TEXT NOT NULL,
            word2 TEXT NOT NULL,
            word3 TEXT NOT NULL,
            line_no INTEGER,
            word_start INTEGER,
            node_index INTEGER,
            caps_score INTEGER NOT NULL,
            exact_line INTEGER NOT NULL,
            context_before TEXT NOT NULL,
            context_after TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS key_metrics (
            metric_name TEXT PRIMARY KEY,
            metric_value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS metric_selections (
            metric_name TEXT NOT NULL,
            phrase_set TEXT NOT NULL,
            selected_index INTEGER NOT NULL,
            phrase_count INTEGER NOT NULL,
            phrase TEXT NOT NULL
        );
        """
    )


def reset_tables(conn: sqlite3.Connection) -> None:
    for table in ["inputs", "words", "phrase_occurrences", "key_metrics", "metric_selections"]:
        conn.execute(f"DELETE FROM {table}")


def insert_words(conn: sqlite3.Connection, source_name: str, words: list[Word]) -> None:
    conn.executemany(
        """
        INSERT INTO words (source_name, ordinal, line_no, node_index, token, normalized_token)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        [
            (source_name, word.ordinal, word.line_no, word.node_index, word.token, word.normalized)
            for word in words
        ],
    )


def insert_occurrences(conn: sqlite3.Connection, occurrences: list[PhraseOccurrence]) -> None:
    conn.executemany(
        """
        INSERT INTO phrase_occurrences (
            phrase_set, phrase_index, phrase, normalized_phrase, word1, word2, word3,
            line_no, word_start, node_index, caps_score, exact_line, context_before, context_after
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        [
            (
                occurrence.phrase_set,
                occurrence.phrase_index,
                occurrence.phrase,
                occurrence.normalized_phrase,
                occurrence.word1,
                occurrence.word2,
                occurrence.word3,
                occurrence.line_no,
                occurrence.word_start,
                occurrence.node_index,
                occurrence.caps_score,
                occurrence.exact_line,
                occurrence.context_before,
                occurrence.context_after,
            )
            for occurrence in occurrences
        ],
    )


def write_report(
    path: Path,
    metrics: dict[str, str],
    phrase_sets: dict[str, list[PhraseOccurrence]],
    selection_rows: list[tuple[str, str, int, int, str]],
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# SWARM Puzzle Phrase Index",
        "",
        "## Key Metrics",
    ]
    for name, value in metrics.items():
        lines.append(f"- `{name}`: `{value}`")
    lines.append("")
    lines.append("## Exact Visible 3-Word Phrases")
    for occurrence in phrase_sets["visible_exact_line_3words_dedup"]:
        lines.append(f"- {occurrence.phrase}")
    lines.append("")
    lines.append("## Metric Selections")
    for metric_name, phrase_set, selected_index, phrase_count, phrase in selection_rows:
        if phrase_set not in {
            "visible_exact_line_3words_dedup",
            "visible_title_exact_line_3words_dedup",
            "visible_title_trigrams_dedup",
        }:
            continue
        lines.append(
            f"- `{metric_name}` on `{phrase_set}` -> `{phrase}` "
            f"(index `{selected_index}` of `{phrase_count}`)"
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def build_index(html_path: Path, visible_path: Path, db_path: Path, key: str, report_path: Path | None) -> dict[str, object]:
    raw_html = load_text(html_path)
    visible_text = load_text(visible_path)

    html_nodes = html_nodes_from_text(raw_html)
    visible_lines = [line for line in visible_text.splitlines() if line.strip()]

    html_words = words_from_nodes(html_nodes)
    visible_words = words_from_lines(visible_lines)

    phrase_sets: dict[str, list[PhraseOccurrence]] = {}
    phrase_sets["html_all_trigrams"] = phrase_occurrences_from_words(html_words, "html_all_trigrams", title_like_only=False)
    phrase_sets["html_title_trigrams"] = phrase_occurrences_from_words(html_words, "html_title_trigrams", title_like_only=True)
    phrase_sets["visible_all_trigrams"] = phrase_occurrences_from_words(visible_words, "visible_all_trigrams", title_like_only=False)
    phrase_sets["visible_title_trigrams"] = phrase_occurrences_from_words(visible_words, "visible_title_trigrams", title_like_only=True)
    phrase_sets["visible_exact_line_3words"] = phrase_occurrences_from_exact_lines(
        visible_lines,
        "visible_exact_line_3words",
        title_like_only=False,
    )
    phrase_sets["visible_title_exact_line_3words"] = phrase_occurrences_from_exact_lines(
        visible_lines,
        "visible_title_exact_line_3words",
        title_like_only=True,
    )

    deduped_sets: dict[str, list[PhraseOccurrence]] = {}
    for name, occurrences in phrase_sets.items():
        deduped_sets[f"{name}_dedup"] = dedupe_occurrences(occurrences, f"{name}_dedup")
    phrase_sets.update(deduped_sets)

    metrics = key_metrics(key)
    selection_rows = metric_selection_rows(metrics, deduped_sets)

    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    try:
        create_schema(conn)
        reset_tables(conn)
        conn.executemany("INSERT INTO inputs (name, value) VALUES (?, ?)", [("key", key), ("html_path", str(html_path)), ("visible_path", str(visible_path))])
        insert_words(conn, "html", html_words)
        insert_words(conn, "visible", visible_words)
        for occurrences in phrase_sets.values():
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

    if report_path is not None:
        write_report(report_path, metrics, phrase_sets, selection_rows)

    return {
        "db_path": str(db_path),
        "report_path": str(report_path) if report_path is not None else None,
        "phrase_counts": {name: len(occurrences) for name, occurrences in phrase_sets.items()},
        "visible_exact_line_phrases": [occurrence.phrase for occurrence in phrase_sets["visible_exact_line_3words_dedup"]],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--html", type=Path, default=DEFAULT_HTML, help="Path to saved SWARM HTML")
    parser.add_argument("--visible", type=Path, default=DEFAULT_VISIBLE, help="Path to saved visible rendered text")
    parser.add_argument("--db", type=Path, default=DEFAULT_DB, help="Output SQLite database path")
    parser.add_argument("--report", type=Path, default=DEFAULT_DB.with_suffix(".md"), help="Output markdown report path")
    parser.add_argument("--key", default=DEFAULT_KEY, help="Puzzle key string")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    summary = build_index(args.html, args.visible, args.db, args.key, args.report)
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
