#!/usr/bin/env python3
"""
compute-gaps.py — Deterministic gap computation for agent package provisioning.

Compares required capabilities (from LLM analysis) against the effective
tool/skill inventory and the curated capability registry. Produces a JSON
gap report: what's already available, what's missing, and what can be
resolved from known sources vs needing web search.

Usage:
  python3 compute-gaps.py \
    --capabilities <capabilities.json> \
    --inventory-tools <tools-inventory.json> \
    --inventory-skills <skills-inventory.json> \
    --registry <capability-registry.yaml> \
    [--agent <agent-name>]

Output (stdout): JSON matching the gap computation result schema.
"""

import argparse
import json
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    # Fallback: inline YAML parser for simple structure
    yaml = None


def parse_yaml_simple(text: str) -> dict:
    """Minimal YAML parser for capability-registry.yaml structure."""
    if yaml:
        return yaml.safe_load(text)
    # Very basic fallback — only handles our specific YAML structure
    import re
    result = {}
    current_section = None
    current_key = None
    current_obj = {}

    for line in text.split("\n"):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        # Top-level key (capabilities: or npx_servers:)
        if re.match(r"^\w[\w_-]*:\s*$", line) and not line.startswith(" "):
            if current_section and current_key:
                result.setdefault(current_section, {})[current_key] = current_obj
            current_section = stripped.rstrip(":")
            result[current_section] = {}
            current_key = None
            current_obj = {}
        # Second-level key (capability name)
        elif re.match(r"^  \w[\w_-]*:\s*$", stripped) or (
            line.startswith("  ") and not line.startswith("    ") and ":" in stripped
        ):
            if current_key and current_section:
                result[current_section][current_key] = current_obj
            current_key = stripped.split(":")[0].strip()
            current_obj = {}
        # Array item
        elif stripped.startswith("- ") and current_key:
            field = None
            for f in ("tools", "skills", "args", "env_vars"):
                if f in current_obj and isinstance(current_obj[f], list):
                    field = f
                    break
            if field:
                current_obj[field].append(stripped[2:].strip())
        # Key-value in object
        elif ":" in stripped and current_key:
            k, v = stripped.split(":", 1)
            k = k.strip()
            v = v.strip().strip('"').strip("'")
            if v.startswith("[") and v.endswith("]"):
                items = [x.strip().strip('"').strip("'") for x in v[1:-1].split(",") if x.strip()]
                current_obj[k] = items
            elif v.lower() == "true":
                current_obj[k] = True
            elif v.lower() == "false":
                current_obj[k] = False
            elif v == "":
                current_obj[k] = []  # likely an array that follows
            else:
                current_obj[k] = v

    if current_key and current_section:
        result[current_section][current_key] = current_obj

    return result


def load_json(path: str) -> dict:
    with open(path) as f:
        return json.load(f)


def load_registry(path: str) -> dict:
    text = Path(path).read_text()
    return parse_yaml_simple(text)


def compute_gaps(
    capabilities: list,
    tools_inventory: list,
    skills_inventory: list,
    registry: dict,
) -> dict:
    """
    Compute gaps between required capabilities and available tools/skills.

    Returns:
        {
            "already_available": [{"capability", "satisfied_by_tools", "satisfied_by_skills"}],
            "resolvable_from_registry": [{"capability", "tool_servers", "skills", "local_fallback"}],
            "needs_search": [{"capability", "reason"}],
            "summary": {"total", "available", "resolvable", "needs_search"}
        }
    """
    cap_registry = registry.get("capabilities", {})
    npx_servers = registry.get("npx_servers", {})

    # Normalize inventory to sets of prefixes/slugs
    tool_prefixes = set()
    for t in tools_inventory:
        if isinstance(t, str):
            # Could be "github_search_code" → extract prefix "github"
            prefix = t.split("_")[0] if "_" in t else t
            tool_prefixes.add(prefix)
        elif isinstance(t, dict):
            tool_prefixes.add(t.get("name", "").split("_")[0])

    skill_slugs = set()
    for s in skills_inventory:
        if isinstance(s, str):
            skill_slugs.add(s)
        elif isinstance(s, dict):
            skill_slugs.add(s.get("slug", s.get("name", "")))

    already_available = []
    resolvable = []
    needs_search = []

    for cap_item in capabilities:
        cap_name = cap_item.get("capability", "")
        priority = cap_item.get("priority", "required")

        # Look up in registry
        reg_entry = cap_registry.get(cap_name, {})

        if reg_entry.get("builtin"):
            already_available.append({
                "capability": cap_name,
                "priority": priority,
                "satisfied_by": "builtin",
                "satisfied_by_tools": [],
                "satisfied_by_skills": [],
            })
            continue

        reg_tools = reg_entry.get("tools", [])
        reg_skills = reg_entry.get("skills", [])

        # Check if any registered tool is in our inventory
        matched_tools = [t for t in reg_tools if t in tool_prefixes]
        matched_skills = [s for s in reg_skills if s in skill_slugs]

        if matched_tools or matched_skills:
            already_available.append({
                "capability": cap_name,
                "priority": priority,
                "satisfied_by": "inventory",
                "satisfied_by_tools": matched_tools,
                "satisfied_by_skills": matched_skills,
            })
        elif reg_tools or reg_skills or reg_entry.get("local_fallback"):
            # Registry knows about this capability but agent doesn't have it
            entry = {
                "capability": cap_name,
                "priority": priority,
                "tool_servers": reg_tools,
                "skills": reg_skills,
            }
            # Check for local fallback (npx server)
            local_fb = reg_entry.get("local_fallback")
            if local_fb:
                entry["local_fallback"] = local_fb
            resolvable.append(entry)
        elif cap_name in cap_registry:
            # Known capability but no tools/skills registered for it
            needs_search.append({
                "capability": cap_name,
                "priority": priority,
                "reason": f"Capability '{cap_name}' is known but has no registered tools or skills",
            })
        else:
            # Unknown capability — definitely needs search
            needs_search.append({
                "capability": cap_name,
                "priority": priority,
                "reason": f"Capability '{cap_name}' not in curated registry — web search required",
            })

    return {
        "already_available": already_available,
        "resolvable_from_registry": resolvable,
        "needs_search": needs_search,
        "summary": {
            "total": len(capabilities),
            "available": len(already_available),
            "resolvable": len(resolvable),
            "needs_search": len(needs_search),
        },
    }


def main():
    parser = argparse.ArgumentParser(description="Compute tool/skill gaps for agent provisioning")
    parser.add_argument("--capabilities", required=True, help="Path to capability-analysis.json (LLM output)")
    parser.add_argument("--inventory-tools", required=True, help="Path to tools inventory JSON")
    parser.add_argument("--inventory-skills", required=True, help="Path to skills inventory JSON")
    parser.add_argument("--registry", required=True, help="Path to capability-registry.yaml")
    parser.add_argument("--agent", default="", help="Agent name (for logging)")
    args = parser.parse_args()

    caps_data = load_json(args.capabilities)
    tools_inv = load_json(args.inventory_tools)
    skills_inv = load_json(args.inventory_skills)
    registry = load_registry(args.registry)

    if isinstance(caps_data, list):
        capabilities = caps_data
    else:
        capabilities = caps_data.get("required_capabilities", [])

    result = compute_gaps(capabilities, tools_inv, skills_inv, registry)

    if args.agent:
        result["agent"] = args.agent

    json.dump(result, sys.stdout, indent=2)
    print()  # trailing newline


if __name__ == "__main__":
    main()
