from __future__ import annotations

import argparse
import json
from pathlib import Path

from morgan_avatar_agent.latency import load_turn_records, summarize_records


def find_latest_run(base_dir: Path) -> Path:
    candidates = sorted(base_dir.glob("*-latency.ndjson"), key=lambda path: path.stat().st_mtime)
    if not candidates:
        raise FileNotFoundError(f"No latency files found in {base_dir}")
    return candidates[-1]


def _fmt(value: float | int | None, unit: str = "s") -> str:
    if value is None:
        return "-"
    if isinstance(value, int):
        return str(value)
    return f"{value:.3f}{unit}"


def print_summary(source: Path, summary: dict) -> None:
    print(f"Source: {source}\n")

    print("=== Overall ===")
    print(f"  Turns recorded:   {summary['turn_count']}")
    print(f"  Turns measured:   {summary['measured_turn_count']}")
    print(f"  Interrupted:      {summary['interrupted_turn_count']}")
    print(f"  EOT->Audio p50:   {_fmt(summary.get('p50_eot_to_first_audio_s'))}")
    print(f"  EOT->Audio p95:   {_fmt(summary.get('p95_eot_to_first_audio_s'))}")
    print(f"  Fastest:          {_fmt(summary.get('fastest_eot_to_first_audio_s'))}")
    print(f"  Slowest:          {_fmt(summary.get('slowest_eot_to_first_audio_s'))}")

    print("\n=== Greeting ===")
    print(f"  Count:            {summary.get('greeting_count', 0)}")
    print(f"  Latency p50:      {_fmt(summary.get('greeting_latency_p50_s'))}")

    print("\n=== Conversational Turns ===")
    print(f"  Count:            {summary.get('conversational_turn_count', 0)}")
    print(f"  EOT->Audio p50:   {_fmt(summary.get('conversational_p50_eot_s'))}")
    print(f"  EOT->Audio p95:   {_fmt(summary.get('conversational_p95_eot_s'))}")

    print("\n=== Per-Component ===")
    for label, attr in [
        ("EOU delay", "end_of_utterance_delay_s"),
        ("STT delay", "transcription_delay_s"),
        ("LLM TTFT", "llm_ttft_s"),
        ("TTS TTFB", "tts_ttfb_s"),
    ]:
        count = summary.get(f"{attr}_count", 0)
        p50 = _fmt(summary.get(f"{attr}_p50"))
        p95 = _fmt(summary.get(f"{attr}_p95"))
        print(f"  {label:14s}  n={count:>3}  p50={p50:>8}  p95={p95:>8}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Summarize Morgan agent latency logs.")
    parser.add_argument("--input", type=Path, help="Path to a *-latency.ndjson file.")
    parser.add_argument(
        "--runs-dir",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "runs",
        help="Directory containing latency logs.",
    )
    parser.add_argument("--json", action="store_true", help="Output raw JSON instead of table.")
    args = parser.parse_args()

    source = args.input or find_latest_run(args.runs_dir)
    records = load_turn_records(source)
    summary = summarize_records(records)

    if args.json:
        print(json.dumps({"source": str(source), "summary": summary}, indent=2, sort_keys=True))
    else:
        print_summary(source, summary)


if __name__ == "__main__":
    main()
