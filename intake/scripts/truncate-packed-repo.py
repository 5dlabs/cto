#!/usr/bin/env python3
"""Truncate a repomix packed-repo JSON to fit within LLM context limits.

Prioritises architecture docs and manifests over raw source code so that
LLM deliberation sees *what the platform is*, not implementation details.

Usage: truncate-packed-repo.py <packed-repo.json> [max_bytes]
Default max_bytes: 400000 (~400KB)
"""
import json, sys, os, re

MAX = int(sys.argv[2]) if len(sys.argv) > 2 else 400000

# --- Tier 1: architecture docs & manifests (always include first) ----------
TIER1_EXACT = {
    'AGENTS.md', 'IDENTITY.md', 'SOUL.md', 'TOOLS.md', 'README.md',
    'HANDOFF.md', 'HEARTBEAT.md', 'USER.md', 'CHANGELOG.md',
    'Cargo.toml', 'package.json', 'justfile', 'Tiltfile',
    'cto-config.json', 'process-compose.yaml',
}
TIER1_PREFIX = ['docs/']

# --- Tier 2: config, schemas, infra overviews ------------------------------
TIER2_PREFIX = [
    'config/', 'infra/charts/', 'shared/', 'templates/',
    'intake/prompts/', 'intake/schemas/',
]
TIER2_SUFFIX = ['.toml', '.yaml', '.yml', '.json', '.md']

# --- Tier 3: source code (only if budget remains) -------------------------
TIER3_PREFIX = ['crates/', 'apps/', 'intake/', 'scripts/', 'workflows/']

# --- Deny list: never include (tests, locks, binary stubs, generated) ------
DENY_RE = re.compile(
    r'(\.lock$|\.test\.|_test\.|tests/|__pycache__|node_modules/|'
    r'\.gitkeep$|\.dockerignore$|\.npmrc$|gen/schemas/|'
    r'deno\.lock|package-lock|\.intake/|target/)',
    re.IGNORECASE,
)

def tier_of(path: str) -> int:
    base = os.path.basename(path)
    if base in TIER1_EXACT or any(path.startswith(p) for p in TIER1_PREFIX):
        return 1
    if any(path.startswith(p) for p in TIER2_PREFIX):
        return 2
    if any(path.endswith(s) for s in TIER2_SUFFIX):
        return 2
    if any(path.startswith(p) for p in TIER3_PREFIX):
        return 3
    return 4

path = sys.argv[1]
with open(path) as f:
    data = json.load(f)

# Cap directory structure at 20K (enough for shape, not noise)
ds = data.get('directoryStructure', '')
if len(ds) > 20000:
    ds = ds[:20000] + '\n... (truncated)'

files = data.get('files', {})
if not isinstance(files, dict):
    files = {}

# Classify and sort: tier asc, then smaller files first within tier
ranked = []
for p, c in files.items():
    cs = str(c) if not isinstance(c, str) else c
    if DENY_RE.search(p):
        continue
    ranked.append((tier_of(p), len(cs), p, cs))
ranked.sort(key=lambda x: (x[0], x[1]))

selected = {}
total = len(ds)
tier_counts = {1: 0, 2: 0, 3: 0, 4: 0}

for tier, sz, p, cs in ranked:
    if total + sz >= MAX:
        continue
    selected[p] = cs
    total += sz
    tier_counts[tier] += 1

data['directoryStructure'] = ds
data['files'] = selected
data['_truncation'] = {
    'original_files': len(files),
    'selected_files': len(selected),
    'original_bytes': sum(len(str(v)) for v in files.values()),
    'selected_bytes': total,
    'tier_counts': tier_counts,
}

with open(path, 'w') as f:
    json.dump(data, f)

print(f'Truncated {len(files)}->{len(selected)} files '
      f'(T1:{tier_counts[1]} T2:{tier_counts[2]} T3:{tier_counts[3]}), '
      f'{total:,} chars', file=sys.stderr)
