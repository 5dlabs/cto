#!/usr/bin/env python3
"""Watch Gmail for SWARM puzzle emails, solve them with Codex, and submit."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


DEFAULT_ACCOUNT = "jonathon@5dlabs.ai"
DEFAULT_QUERY = 'in:inbox from:Canteen@user.luma-mail.com "Puzzle #"'
DEFAULT_MAX_RESULTS = 20
DEFAULT_INTERVAL_SECONDS = 300
BF_CHARS = "<>+-.,[]"
TERMINAL_STATUSES = {"submitted", "ignored_not_puzzle"}

SCRIPT_DIR = Path(__file__).resolve().parent
WORKSPACE_ROOT = SCRIPT_DIR.parent
SCHEMA_PATH = SCRIPT_DIR / "swarm_puzzle_solver.schema.json"
DEFAULT_RUNTIME_ROOT = Path.home() / ".swarm-puzzle-agent"
GIST_URL_RE = re.compile(r"https://gist\.github\.com/[^\s)>\"]+")
GIST_ID_RE = re.compile(r"/([0-9a-fA-F]+)(?:[/?#]|$)")
PUZZLE_SUBJECT_RE = re.compile(r"Puzzle\s*#\d+", re.IGNORECASE)
DIGIT_COUNT_RE = re.compile(r"(\d+)\s*digit", re.IGNORECASE)
PREPEND_CLUE_RE = re.compile(r"We(?:'|’)re looking for .*?answer\.", re.IGNORECASE | re.DOTALL)


@dataclass
class CandidateMessage:
    message_id: str
    subject: str
    sender: str
    date: str


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def load_state(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {"messages": {}}
    with path.open() as handle:
        data = json.load(handle)
    if "messages" not in data or not isinstance(data["messages"], dict):
        data["messages"] = {}
    return data


def save_state(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w") as handle:
        json.dump(data, handle, indent=2, sort_keys=True)
        handle.write("\n")


def run_cmd(cmd: list[str], *, input_text: str | None = None, check: bool = True) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        cmd,
        input=input_text,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and proc.returncode != 0:
        raise RuntimeError(
            f"Command failed ({proc.returncode}): {' '.join(cmd)}\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
        )
    return proc


def ensure_tool(name: str) -> None:
    proc = run_cmd(["/bin/zsh", "-lc", f"command -v {name}"], check=False)
    if proc.returncode != 0:
        raise RuntimeError(f"Required tool not found in PATH: {name}")


def gog_json(args: list[str]) -> dict[str, Any]:
    proc = run_cmd(["gog", "-j", *args])
    return json.loads(proc.stdout)


def search_candidates(account: str, query: str, max_results: int) -> list[CandidateMessage]:
    payload = gog_json(["-a", account, "gmail", "search", f"--max={max_results}", query])
    threads = payload.get("threads") or []
    candidates: list[CandidateMessage] = []
    for item in threads:
        candidates.append(
            CandidateMessage(
                message_id=item["id"],
                subject=item.get("subject", ""),
                sender=item.get("from", ""),
                date=item.get("date", ""),
            )
        )
    return candidates


def get_message(account: str, message_id: str) -> dict[str, Any]:
    return gog_json(["-a", account, "gmail", "get", message_id])


def is_puzzle_message(message: dict[str, Any]) -> bool:
    headers = message.get("headers") or {}
    subject = headers.get("subject", "")
    body = message.get("body", "")
    if GIST_URL_RE.search(body) and PUZZLE_SUBJECT_RE.search(subject):
        return True
    if "submit the answer through the cli" in body.lower() and GIST_URL_RE.search(body):
        return True
    return False


def extract_gist_urls(text: str) -> list[str]:
    urls = []
    seen: set[str] = set()
    for match in GIST_URL_RE.findall(text):
        clean = match.rstrip(").,")
        if clean not in seen:
            seen.add(clean)
            urls.append(clean)
    return urls


def extract_expected_digit_count(text: str) -> int | None:
    match = DIGIT_COUNT_RE.search(text)
    if match:
        return int(match.group(1))
    return None


def gist_id_from_url(url: str) -> str:
    match = GIST_ID_RE.search(url)
    if not match:
        raise RuntimeError(f"Could not extract gist ID from URL: {url}")
    return match.group(1)


def looks_like_brainfuck(text: str) -> bool:
    stripped = "".join(ch for ch in text if not ch.isspace())
    if not stripped:
        return False
    bf_only = "".join(ch for ch in stripped if ch in BF_CHARS)
    return len(bf_only) / len(stripped) > 0.8 and "[" in bf_only and "]" in bf_only


def run_brainfuck(program_text: str, input_bytes: list[int] | None = None, step_limit: int = 2_000_000) -> str:
    program = [ch for ch in program_text if ch in BF_CHARS]
    jumps: dict[int, int] = {}
    stack: list[int] = []
    for index, char in enumerate(program):
        if char == "[":
            stack.append(index)
        elif char == "]":
            if not stack:
                raise RuntimeError("Unbalanced brainfuck program: unexpected ]")
            start = stack.pop()
            jumps[start] = index
            jumps[index] = start
    if stack:
        raise RuntimeError("Unbalanced brainfuck program: missing ]")

    tape: dict[int, int] = {0: 0}
    pointer = 0
    pc = 0
    steps = 0
    output_chars: list[str] = []
    inputs = list(input_bytes or [])
    input_index = 0

    while pc < len(program):
        steps += 1
        if steps > step_limit:
            raise RuntimeError("Brainfuck execution exceeded step limit.")

        char = program[pc]
        current = tape.get(pointer, 0)
        if char == ">":
            pointer += 1
        elif char == "<":
            pointer -= 1
        elif char == "+":
            tape[pointer] = (current + 1) % 256
        elif char == "-":
            tape[pointer] = (current - 1) % 256
        elif char == ".":
            output_chars.append(chr(current))
        elif char == ",":
            if input_index < len(inputs):
                tape[pointer] = inputs[input_index] % 256
                input_index += 1
            else:
                tape[pointer] = 0
        elif char == "[":
            if current == 0:
                pc = jumps[pc]
        elif char == "]":
            if current != 0:
                pc = jumps[pc]
        pc += 1

    return "".join(output_chars)


def extract_prepend_clue(text: str) -> str | None:
    match = PREPEND_CLUE_RE.search(text)
    if match:
        return match.group(0)
    return None


def heuristic_solver(body: str, gist_payloads: list[dict[str, Any]], expected_digits: int | None) -> dict[str, Any] | None:
    files: list[str] = []
    for payload in gist_payloads:
        for file_meta in (payload.get("files") or {}).values():
            content = file_meta.get("content") or ""
            if content:
                files.append(content)

    if not files or not all(looks_like_brainfuck(content) for content in files):
        return None

    clue = extract_prepend_clue(body) or ""
    input_hint = expected_digits if expected_digits is not None and 0 <= expected_digits <= 255 else None
    candidate_inputs = [[]]
    if input_hint is not None:
        candidate_inputs.insert(0, [input_hint])

    candidate_programs = []
    for content in files:
        if expected_digits is not None and expected_digits >= 0:
            candidate_programs.append(("digit_plus_prefix", "+" * expected_digits + content))
        candidate_programs.append(content)
        if clue:
            candidate_programs.append(("email_clue_prefix", clue + content))
        else:
            candidate_programs.append(("plain", content))

    expanded_programs: list[tuple[str, str]] = []
    for item in candidate_programs:
        if isinstance(item, tuple):
            expanded_programs.append(item)
        else:
            expanded_programs.append(("plain", item))

    for program_name, program in expanded_programs:
        for inputs in candidate_inputs:
            try:
                output = run_brainfuck(program, inputs)
            except RuntimeError:
                continue
            canonical = canonicalize_submission_text(output)
            if canonical and matches_digit_hint(canonical, expected_digits):
                return {
                    "status": "solved",
                    "answer": canonical,
                    "confidence": 0.99,
                    "reasoning_summary": f"Solved with the built-in Brainfuck heuristic path using {program_name}.",
                }

    return None


def fetch_gist(gist_id: str) -> dict[str, Any]:
    request = urllib.request.Request(
        f"https://api.github.com/gists/{gist_id}",
        headers={"User-Agent": "swarm-puzzle-runner"},
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            return json.load(response)
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Failed to fetch gist {gist_id}: {exc}") from exc


def write_bundle(bundle_dir: Path, message: dict[str, Any], gist_payloads: list[dict[str, Any]]) -> None:
    bundle_dir.mkdir(parents=True, exist_ok=True)

    with (bundle_dir / "email.json").open("w") as handle:
        json.dump(message, handle, indent=2, sort_keys=True)
        handle.write("\n")

    body = message.get("body", "")
    (bundle_dir / "email_body.txt").write_text(body)

    headers = message.get("headers") or {}
    with (bundle_dir / "headers.json").open("w") as handle:
        json.dump(headers, handle, indent=2, sort_keys=True)
        handle.write("\n")

    gist_dir = bundle_dir / "gists"
    gist_dir.mkdir(exist_ok=True)

    manifest: list[dict[str, Any]] = []
    for payload in gist_payloads:
        gist_id = payload["id"]
        with (gist_dir / f"{gist_id}.json").open("w") as handle:
            json.dump(payload, handle, indent=2, sort_keys=True)
            handle.write("\n")
        for file_name, file_meta in (payload.get("files") or {}).items():
            content = file_meta.get("content") or ""
            safe_name = file_name.replace("/", "_")
            target = gist_dir / f"{gist_id}__{safe_name}"
            target.write_text(content)
            manifest.append(
                {
                    "gist_id": gist_id,
                    "file_name": file_name,
                    "path": str(target),
                    "size": file_meta.get("size"),
                }
            )

    with (bundle_dir / "gist_manifest.json").open("w") as handle:
        json.dump(manifest, handle, indent=2, sort_keys=True)
        handle.write("\n")


def load_bundle(bundle_dir: Path) -> tuple[dict[str, Any], str, list[dict[str, Any]]]:
    with (bundle_dir / "email.json").open() as handle:
        message = json.load(handle)
    body = (bundle_dir / "email_body.txt").read_text()
    gist_dir = bundle_dir / "gists"
    gist_payloads = []
    for gist_json in sorted(gist_dir.glob("*.json")):
        with gist_json.open() as handle:
            gist_payloads.append(json.load(handle))
    return message, body, gist_payloads


def build_solver_prompt(bundle_dir: Path, expected_digits: int | None) -> str:
    lines = [
        "Solve the SWARM puzzle from the provided email and gist files.",
        f"Read these files under: {bundle_dir}",
        f"- {bundle_dir / 'email_body.txt'}",
        f"- {bundle_dir / 'email.json'}",
        f"- {bundle_dir / 'headers.json'}",
        f"- {bundle_dir / 'gist_manifest.json'}",
        f"- all files under {bundle_dir / 'gists'}",
        "Use the email instructions and gist contents together.",
        "Do not submit anything. Do not modify any files.",
        "Return the final answer only through the JSON schema.",
    ]
    if expected_digits is not None:
        lines.append(
            f"The email appears to require {expected_digits} digits in the visible answer, but preserve punctuation exactly as printed."
        )
    lines.append("Do not strip punctuation, decimal points, or other formatting from the final answer.")
    return "\n".join(lines)


def solve_with_codex(
    bundle_dir: Path,
    runtime_root: Path,
    workspace_root: Path,
    expected_digits: int | None,
    model: str | None,
) -> dict[str, Any]:
    output_path = bundle_dir / "solver_result.json"
    prompt = build_solver_prompt(bundle_dir, expected_digits)
    cmd = [
        "codex",
        "exec",
        "--skip-git-repo-check",
        "--color",
        "never",
        "-s",
        "read-only",
        "-C",
        str(workspace_root),
        "--add-dir",
        str(runtime_root),
        "--output-schema",
        str(SCHEMA_PATH),
        "-o",
        str(output_path),
    ]
    if model:
        cmd.extend(["-m", model])
    cmd.append(prompt)

    proc = run_cmd(cmd, check=False)
    (bundle_dir / "solver_stdout.log").write_text(proc.stdout)
    (bundle_dir / "solver_stderr.log").write_text(proc.stderr)
    if proc.returncode != 0:
        raise RuntimeError(
            f"codex exec failed ({proc.returncode}); see {bundle_dir / 'solver_stderr.log'}"
        )
    with output_path.open() as handle:
        return json.load(handle)


def canonicalize_submission_text(answer: str | None) -> str | None:
    if answer is None:
        return None
    raw = answer.replace("\r\n", "\n").strip("\r\n")
    if not raw:
        return None
    lines = [line for line in raw.split("\n") if line.strip() != ""]
    if len(lines) == 1:
        return lines[0]
    return raw


def matches_digit_hint(answer: str, expected_digits: int | None) -> bool:
    if expected_digits is None:
        return True
    digits = re.sub(r"\D", "", answer)
    return len(digits) == expected_digits


def submit_answer(answer: str) -> subprocess.CompletedProcess[str]:
    return run_cmd(["swarm", "submit-puzzle"], input_text=f"{answer}\n\n", check=False)


def ensure_swarm_ready() -> None:
    config_path = Path.home() / ".swarm" / "config.yaml"
    if not config_path.exists():
        raise RuntimeError("SWARM CLI is not configured. Set up `swarm login` first.")


def solve_bundle(bundle_dir: Path, runtime_root: Path, workspace_root: Path, model: str | None) -> dict[str, Any]:
    message, body, gist_payloads = load_bundle(bundle_dir)
    expected_digits = extract_expected_digit_count(body)
    result = heuristic_solver(body, gist_payloads, expected_digits)
    if result is None:
        result = solve_with_codex(bundle_dir, runtime_root, workspace_root, expected_digits, model)
    else:
        with (bundle_dir / "solver_result.json").open("w") as handle:
            json.dump(result, handle, indent=2, sort_keys=True)
            handle.write("\n")

    normalized = canonicalize_submission_text(result.get("answer"))
    return {
        "subject": (message.get("headers") or {}).get("subject"),
        "status": "dry_run_ready" if normalized and matches_digit_hint(normalized, expected_digits) else "failed_unsolved",
        "answer": normalized,
        "bundle_dir": str(bundle_dir),
        "last_error": None if normalized and matches_digit_hint(normalized, expected_digits) else f"Solver did not return a valid answer: {result}",
    }


def process_message(
    *,
    account: str,
    message_id: str,
    state: dict[str, Any],
    state_path: Path,
    runtime_root: Path,
    workspace_root: Path,
    model: str | None,
    dry_run: bool,
) -> dict[str, Any]:
    entry = state["messages"].setdefault(message_id, {})
    entry["last_attempted_at"] = utc_now()
    entry["attempts"] = int(entry.get("attempts", 0)) + 1

    message = get_message(account, message_id)
    headers = message.get("headers") or {}
    body = message.get("body", "")
    subject = headers.get("subject", "")
    gist_urls = extract_gist_urls(body)
    expected_digits = extract_expected_digit_count(body)

    entry.update(
        {
            "account": account,
            "subject": subject,
            "sender": headers.get("from", ""),
            "date": headers.get("date", ""),
            "gist_urls": gist_urls,
            "expected_digits": expected_digits,
        }
    )

    if not is_puzzle_message(message):
        entry["status"] = "ignored_not_puzzle"
        save_state(state_path, state)
        return entry

    if not gist_urls:
        entry["status"] = "failed_no_gist"
        entry["last_error"] = "No gist URL found in message body."
        save_state(state_path, state)
        return entry

    gist_payloads = [fetch_gist(gist_id_from_url(url)) for url in gist_urls]

    bundle_dir = runtime_root / "bundles" / message_id
    write_bundle(bundle_dir, message, gist_payloads)
    entry["bundle_dir"] = str(bundle_dir)

    result = heuristic_solver(body, gist_payloads, expected_digits)
    if result is None:
        result = solve_with_codex(bundle_dir, runtime_root, workspace_root, expected_digits, model)
    else:
        with (bundle_dir / "solver_result.json").open("w") as handle:
            json.dump(result, handle, indent=2, sort_keys=True)
            handle.write("\n")
    entry["solver_result"] = result

    normalized = canonicalize_submission_text(result.get("answer"))
    if result.get("status") != "solved" or not normalized or not matches_digit_hint(normalized, expected_digits):
        entry["status"] = "failed_unsolved"
        entry["last_error"] = f"Solver did not return a valid answer: {result}"
        save_state(state_path, state)
        return entry

    entry["answer"] = normalized

    if dry_run:
        entry["status"] = "dry_run_ready"
        save_state(state_path, state)
        return entry

    ensure_swarm_ready()
    submit = submit_answer(normalized)
    (bundle_dir / "submit_stdout.log").write_text(submit.stdout)
    (bundle_dir / "submit_stderr.log").write_text(submit.stderr)

    if submit.returncode != 0 or "Puzzle answer submitted." not in submit.stdout:
        entry["status"] = "failed_submit"
        entry["last_error"] = f"swarm submit-puzzle failed: {submit.stdout}\n{submit.stderr}"
        save_state(state_path, state)
        return entry

    entry["status"] = "submitted"
    entry["submitted_at"] = utc_now()
    save_state(state_path, state)
    return entry


def poll_once(
    *,
    account: str,
    query: str,
    max_results: int,
    state: dict[str, Any],
    state_path: Path,
    runtime_root: Path,
    workspace_root: Path,
    model: str | None,
    dry_run: bool,
    message_id: str | None,
) -> list[dict[str, Any]]:
    targets: list[str]
    if message_id:
        targets = [message_id]
    else:
        candidates = search_candidates(account, query, max_results)
        targets = [candidate.message_id for candidate in candidates]

    results = []
    for candidate_id in targets:
        current = state["messages"].get(candidate_id, {})
        if current.get("status") in TERMINAL_STATUSES:
            continue
        results.append(
            process_message(
                account=account,
                message_id=candidate_id,
                state=state,
                state_path=state_path,
                runtime_root=runtime_root,
                workspace_root=workspace_root,
                model=model,
                dry_run=dry_run,
            )
        )
    return results


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--account", default=DEFAULT_ACCOUNT, help="Gmail account for gog.")
    parser.add_argument("--query", default=DEFAULT_QUERY, help="Gmail query for candidate messages.")
    parser.add_argument("--max-results", type=int, default=DEFAULT_MAX_RESULTS, help="Max messages to inspect per poll.")
    parser.add_argument("--interval", type=int, default=DEFAULT_INTERVAL_SECONDS, help="Watch interval in seconds.")
    parser.add_argument("--watch", action="store_true", help="Poll in a loop.")
    parser.add_argument("--dry-run", action="store_true", help="Solve but do not submit.")
    parser.add_argument("--message-id", help="Process a specific Gmail message ID.")
    parser.add_argument("--runtime-root", default=str(DEFAULT_RUNTIME_ROOT), help="Runtime state directory.")
    parser.add_argument("--workspace-root", default=str(WORKSPACE_ROOT), help="Workspace for codex exec.")
    parser.add_argument("--model", help="Optional model override for codex exec.")
    parser.add_argument("--bundle-dir", help="Solve an existing saved bundle without polling Gmail or submitting.")
    return parser.parse_args()


def print_summary(results: list[dict[str, Any]]) -> None:
    if not results:
        print("No new puzzle emails processed.")
        return
    for item in results:
        print(
            json.dumps(
                {
                    "subject": item.get("subject"),
                    "status": item.get("status"),
                    "answer": item.get("answer"),
                    "bundle_dir": item.get("bundle_dir"),
                    "last_error": item.get("last_error"),
                },
                sort_keys=True,
            )
        )


def main() -> int:
    args = parse_args()
    runtime_root = Path(args.runtime_root).expanduser().resolve()
    workspace_root = Path(args.workspace_root).expanduser().resolve()
    state_path = runtime_root / "state.json"

    if not args.bundle_dir:
        ensure_tool("gog")
    ensure_tool("codex")
    if not args.bundle_dir and not args.dry_run:
        ensure_tool("swarm")

    runtime_root.mkdir(parents=True, exist_ok=True)
    (runtime_root / "bundles").mkdir(exist_ok=True)
    state = load_state(state_path)

    if args.bundle_dir:
        result = solve_bundle(Path(args.bundle_dir).expanduser().resolve(), runtime_root, workspace_root, args.model)
        print_summary([result])
        return 0

    while True:
        try:
            results = poll_once(
                account=args.account,
                query=args.query,
                max_results=args.max_results,
                state=state,
                state_path=state_path,
                runtime_root=runtime_root,
                workspace_root=workspace_root,
                model=args.model,
                dry_run=args.dry_run,
                message_id=args.message_id,
            )
            print_summary(results)
        except Exception as exc:  # noqa: BLE001
            print(f"swarm_puzzle_runner error: {exc}", file=sys.stderr)
            return 1

        if not args.watch or args.message_id:
            return 0
        time.sleep(args.interval)


if __name__ == "__main__":
    raise SystemExit(main())
