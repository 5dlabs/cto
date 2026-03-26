#!/usr/bin/env python3
"""
BirdEye parity harness for dex_feed gRPC endpoints.

Requires grpcurl installed locally.
Usage:
  python3 scripts/birdeye_parity_harness.py \
    --local localhost:50051 \
    --reference api.birdeye.your-endpoint:443 \
    --token So11111111111111111111111111111111111111112
"""

from __future__ import annotations

import argparse
import json
import statistics
import subprocess
import sys
from typing import Any


METHODS = [
    ("dex_feed.DexQuery/GetPrice", lambda token: {"token": token}),
    ("dex_feed.DexQuery/GetTokenOverview", lambda token: {"token": token}),
    (
        "dex_feed.DexQuery/GetPriceHistory",
        lambda token: {
            "token": token,
            "interval": "INTERVAL_1M",
            "time_from": 0,
            "time_to": 4102444800,
        },
    ),
]


def run_grpcurl(
    target: str,
    method: str,
    payload: dict[str, Any],
    insecure: bool,
    import_path: str,
    proto_file: str,
) -> dict[str, Any]:
    cmd = ["grpcurl"]
    if insecure:
        cmd.append("-insecure")
    cmd.extend(["-import-path", import_path, "-proto", proto_file])
    cmd.extend(
        [
            "-plaintext" if target.startswith("localhost:") else "",
            "-d",
            json.dumps(payload),
            target,
            method,
        ]
    )
    cmd = [c for c in cmd if c]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"grpcurl failed for {target} {method}: {proc.stderr.strip()}")
    return json.loads(proc.stdout or "{}")


def extract_price(resp: dict[str, Any]) -> float | None:
    if "price" in resp and isinstance(resp["price"], dict):
        return _to_float(resp["price"].get("priceUsd"))
    if "priceUsd" in resp:
        return _to_float(resp["priceUsd"])
    return None


def _to_float(value: Any) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--local", required=True, help="Local gRPC endpoint host:port")
    parser.add_argument("--reference", required=True, help="Reference (BirdEye) gRPC endpoint host:port")
    parser.add_argument("--token", required=True, help="Token mint")
    parser.add_argument("--insecure", action="store_true", help="Pass -insecure to grpcurl")
    parser.add_argument(
        "--import-path",
        default="crates/dex-indexer/proto",
        help="Path for grpcurl proto imports",
    )
    parser.add_argument(
        "--proto",
        default="dex_feed.proto",
        help="Proto filename for grpcurl invocation",
    )
    args = parser.parse_args()

    diffs: list[float] = []
    print("method,local_price,reference_price,relative_error")

    for method, payload_fn in METHODS:
        payload = payload_fn(args.token)
        local = run_grpcurl(
            args.local,
            method,
            payload,
            args.insecure,
            args.import_path,
            args.proto,
        )
        ref = run_grpcurl(
            args.reference,
            method,
            payload,
            args.insecure,
            args.import_path,
            args.proto,
        )
        local_price = extract_price(local)
        ref_price = extract_price(ref)
        rel = None
        if local_price is not None and ref_price not in (None, 0.0):
            rel = abs(local_price - ref_price) / abs(ref_price)
            diffs.append(rel)
        print(f"{method},{local_price},{ref_price},{rel}")

    if diffs:
        print(f"median_relative_error={statistics.median(diffs):.6f}")
        print(f"p95_relative_error={statistics.quantiles(diffs, n=100)[94]:.6f}" if len(diffs) > 1 else "")
    else:
        print("no comparable price points found")
        return 2
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"harness_failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
