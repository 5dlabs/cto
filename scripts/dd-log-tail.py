#!/usr/bin/env python3
"""Tail a log file and ship lines to Datadog Logs API (HTTP intake).

Strips ANSI escape codes, parses tracing_subscriber format into structured
attributes (timestamp, level, target, span, message), and ships as JSON.
"""
import os, sys, time, json, re, requests

DD_API_KEY = os.environ.get("DD_API_KEY")
DD_SITE = os.environ.get("DD_SITE", "us5.datadoghq.com")
LOG_FILE = os.environ.get("LOG_FILE", "logs/controller.log")
SERVICE = os.environ.get("DD_SERVICE", "controller")
SOURCE = os.environ.get("DD_SOURCE", "rust")
HOSTNAME = os.environ.get("DD_HOSTNAME", "local-dev")
BATCH_SIZE = 10
FLUSH_INTERVAL = 5  # seconds

INTAKE_URL = f"https://http-intake.logs.{DD_SITE}/api/v2/logs"

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
# tracing_subscriber default format:
#   2026-04-11T04:16:18.757319Z  INFO agent_controller: Starting...
#   2026-04-11T05:36:22.313181Z  INFO request{method=GET uri=/health ...}: tower_http::trace::on_response: finished...
TRACING_RE = re.compile(
    r"^(?P<timestamp>\d{4}-\d{2}-\d{2}T[\d:.]+Z)\s+"
    r"(?P<level>\w+)\s+"
    r"(?P<target>[^\s:]+(?:\{[^}]*\})?)\s*:\s*"
    r"(?P<body>.*)$"
)

DD_LEVEL_MAP = {
    "TRACE": "debug",
    "DEBUG": "debug",
    "INFO": "info",
    "WARN": "warn",
    "WARNING": "warn",
    "ERROR": "error",
    "FATAL": "critical",
}


def strip_ansi(text):
    return ANSI_RE.sub("", text)


def parse_line(raw):
    """Parse a tracing_subscriber log line into structured attributes."""
    clean = strip_ansi(raw).rstrip()
    if not clean:
        return None

    m = TRACING_RE.match(clean)
    if m:
        level_raw = m.group("level").upper()
        target = m.group("target")
        body = m.group("body")

        # Split target{span}: module::path: message
        span = ""
        module = ""
        if "{" in target:
            tname, span = target.split("{", 1)
            span = span.rstrip("}")
            target = tname
        parts = body.split(": ", 1)
        if len(parts) == 2 and "::" in parts[0]:
            module = parts[0]
            body = parts[1]

        return {
            "timestamp": m.group("timestamp"),
            "level": level_raw,
            "dd_status": DD_LEVEL_MAP.get(level_raw, "info"),
            "target": target,
            "span": span,
            "module": module,
            "body": body,
            "message_clean": clean,
        }

    return {"message_clean": clean, "dd_status": "info"}


def send_batch(lines):
    if not lines:
        return
    payload = []
    for line in lines:
        parsed = parse_line(line)
        if not parsed:
            continue

        entry = {
            "ddsource": SOURCE,
            "ddtags": f"env:local,team:cto",
            "hostname": HOSTNAME,
            "service": SERVICE,
            "message": parsed.get("message_clean", strip_ansi(line).rstrip()),
            "status": parsed.get("dd_status", "info"),
        }
        # Attach structured attributes
        attrs = {}
        for key in ("timestamp", "level", "target", "span", "module", "body"):
            val = parsed.get(key)
            if val:
                attrs[key] = val
        if attrs:
            entry["rust"] = attrs

        payload.append(entry)

    if not payload:
        return
    try:
        r = requests.post(
            INTAKE_URL,
            json=payload,
            headers={
                "DD-API-KEY": DD_API_KEY,
                "Content-Type": "application/json",
            },
            timeout=10,
        )
        if r.status_code == 202:
            print(f"  → Shipped {len(payload)} log lines", file=sys.stderr)
        else:
            print(f"  ✗ DD returned {r.status_code}: {r.text[:120]}", file=sys.stderr)
    except Exception as e:
        print(f"  ✗ Send failed: {e}", file=sys.stderr)

def tail_and_ship(path):
    print(f"Tailing {path} → DD Logs ({DD_SITE}), service={SERVICE}", file=sys.stderr)
    with open(path, "r") as f:
        # Seek to end
        f.seek(0, 2)
        batch = []
        last_flush = time.time()
        while True:
            line = f.readline()
            if line:
                sys.stdout.write(line)  # also print locally
                sys.stdout.flush()
                batch.append(line)
                if len(batch) >= BATCH_SIZE:
                    send_batch(batch)
                    batch = []
                    last_flush = time.time()
            else:
                if batch and (time.time() - last_flush) >= FLUSH_INTERVAL:
                    send_batch(batch)
                    batch = []
                    last_flush = time.time()
                time.sleep(0.5)

if __name__ == "__main__":
    if not DD_API_KEY:
        print("ERROR: DD_API_KEY not set", file=sys.stderr)
        sys.exit(1)
    path = sys.argv[1] if len(sys.argv) > 1 else LOG_FILE
    tail_and_ship(path)
