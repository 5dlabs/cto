#!/usr/bin/env python3
"""CTO intake *.lobster.yaml → @clawdbot/lobster workflow file shape (id + command)."""
from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOW_DIR = REPO_ROOT / "intake" / "workflows"


def inputs_to_args(data: dict) -> None:
    if "inputs" not in data or not isinstance(data["inputs"], list):
        return
    args: dict = {}
    for item in data["inputs"]:
        if not isinstance(item, dict) or not item.get("name"):
            continue
        name = item["name"]
        entry: dict = {}
        if "default" in item:
            entry["default"] = item["default"]
        args[name] = entry if entry else {"default": ""}
    data["args"] = args
    del data["inputs"]


def strip_workflow_meta(data: dict) -> None:
    for k in ("outputs", "output_mappings", "description"):
        data.pop(k, None)


def transform_cmd(cmd: str) -> str:
    def sub_jq_r(m: re.Match) -> str:
        sid = m.group(1)
        q = m.group(2).replace("'", "'\\''")
        return f"$(printf '%s' \"${sid}.stdout\" | jq -r '{q}')"

    def sub_jq(m: re.Match) -> str:
        sid = m.group(1)
        q = m.group(2).replace("'", "'\\''")
        return f"$(printf '%s' \"${sid}.stdout\" | jq '{q}')"

    def sub_outs(m: re.Match) -> str:
        sid = m.group(1)
        field = m.group(2)
        return (
            f"$(printf '%s' \"${sid}.stdout\" | jq -c '.output[0].{field}' "
            f"2>/dev/null || printf '%s' \"${sid}.stdout\")"
        )

    def sub_out(m: re.Match) -> str:
        sid = m.group(1)
        return f"$(printf '%s' \"${sid}.stdout\")"

    cmd = re.sub(
        r"\$\{\{\s*steps\.([A-Za-z0-9_-]+)\.output\s*\|\s*jq\s+-r\s+'([^']*)'\s*\}\}",
        sub_jq_r,
        cmd,
    )
    cmd = re.sub(
        r"\$\{\{\s*steps\.([A-Za-z0-9_-]+)\.output\s*\|\s*jq\s+'([^']*)'\s*\}\}",
        sub_jq,
        cmd,
    )
    cmd = re.sub(
        r"\$\{\{\s*steps\.([A-Za-z0-9_-]+)\.outputs\.([A-Za-z0-9_-]+)\s*\}\}",
        sub_outs,
        cmd,
    )
    cmd = re.sub(
        r"\$\{\{\s*steps\.([A-Za-z0-9_-]+)\.output\s*\}\}",
        sub_out,
        cmd,
    )
    cmd = re.sub(r"\{\{inputs\.([A-Za-z0-9_-]+)\}\}", r"${\1}", cmd)
    return cmd


def arg_jq_line(key: str, value_template: str) -> str:
    tv = transform_cmd(value_template.strip())
    if tv.strip().startswith("$("):
        return f'  --arg {key} "{tv}" \\'
    return f"  --arg {key} \"$(printf '%s' {tv})\" \\"


def expand_nested(step: dict, wf: str, inputs_map: dict) -> None:
    keys = list(inputs_map.keys())
    lines = [f'WS="${{WORKSPACE:-{REPO_ROOT}}}"', "ARGS=$(jq -n \\"]
    for k in keys:
        lines.append(arg_jq_line(k, str(inputs_map[k])))
    inner = ", ".join(f"{k}: ${k}" for k in keys)
    lines.append(f"  '{{{inner}}}') &&")
    lines.append(f'lobster run --mode tool "$WS/intake/workflows/{wf}" --args-json "$ARGS"')
    step["command"] = "\n".join(lines)
    step.pop("workflow", None)
    step.pop("inputs", None)


def ensure_id(step: dict) -> None:
    if step.get("id"):
        return
    if not step.get("name"):
        raise ValueError("step missing id and name")
    step["id"] = step["name"]


def clean_step(step: dict) -> None:
    for k in ("depends_on", "name", "description", "timeout", "outputs", "when"):
        step.pop(k, None)


def process(data: dict) -> None:
    inputs_to_args(data)
    strip_workflow_meta(data)
    steps = data.get("steps")
    if not isinstance(steps, list):
        raise ValueError("steps must be a list")
    for step in steps:
        if not isinstance(step, dict):
            continue
        ensure_id(step)
        wf = step.get("workflow")
        ins = step.get("inputs")
        if wf and isinstance(ins, dict):
            expand_nested(step, str(wf), dict(ins))
            clean_step(step)
            continue
        appr = step.get("approval")
        cmd = step.get("command")
        if appr and not cmd:
            step["command"] = 'printf \'%s\\n\' \'{"requiresApproval":{"prompt":"Approve checkpoint"}}\''

            step["approval"] = True
            clean_step(step)
        elif cmd:
            step["command"] = transform_cmd(cmd)
            clean_step(step)
            if appr:
                step["approval"] = True
        else:
            raise ValueError(f"step {step.get('id')} missing command and workflow")


class _Dumper(yaml.SafeDumper):
    pass


def _repr_str(dumper: yaml.Dumper, data: str):
    if "\n" in data and len(data) > 80:
        return dumper.represent_scalar("tag:yaml.org,2002:str", data, style="|")
    return dumper.represent_scalar("tag:yaml.org,2002:str", data)


_Dumper.add_representer(str, _repr_str)


def main() -> None:
    paths = [Path(a) for a in sys.argv[1:] if not a.startswith("-")]
    if not paths:
        paths = sorted(WORKFLOW_DIR.glob("*.lobster.yaml"))
    for path in paths:
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict) or "steps" not in data:
            print(f"skip {path.name}", file=sys.stderr)
            continue
        process(data)
        path.write_text(
            yaml.dump(data, Dumper=_Dumper, sort_keys=False, allow_unicode=True, width=120),
            encoding="utf-8",
        )
        print("OK", path.name)


if __name__ == "__main__":
    main()
