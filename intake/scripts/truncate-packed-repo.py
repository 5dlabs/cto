#!/usr/bin/env python3
"""Truncate a repomix packed-repo JSON to fit within LLM context limits.

Usage: truncate-packed-repo.py <packed-repo.json> [max_bytes]
Default max_bytes: 400000 (~400KB)
"""
import json, sys

MAX = int(sys.argv[2]) if len(sys.argv) > 2 else 400000
PRIORITY = [
    'crates/', 'intake/', 'docs/', 'infra/', 'shared/', 'config/',
    'scripts/', 'workflows/', 'templates/', 'apps/',
    'Cargo.toml', 'package.json', 'justfile',
    'AGENTS.md', 'IDENTITY.md', 'SOUL.md', 'TOOLS.md', 'README.md',
]

path = sys.argv[1]
with open(path) as f:
    data = json.load(f)

ds = data.get('directoryStructure', '')
if len(ds) > 50000:
    ds = ds[:50000] + '\n... (truncated)'

files = data.get('files', {})
selected = {}
total = len(ds)

for p, c in (files.items() if isinstance(files, dict) else []):
    cs = str(c) if not isinstance(c, str) else c
    if any(p.startswith(x) or p == x for x in PRIORITY) and total + len(cs) < MAX:
        selected[p] = cs
        total += len(cs)

data['directoryStructure'] = ds
data['files'] = selected
data['_truncation'] = {
    'original_files': len(files),
    'selected_files': len(selected),
    'original_bytes': sum(len(str(v)) for v in files.values()),
    'selected_bytes': total,
}

with open(path, 'w') as f:
    json.dump(data, f)

print(f'Truncated {len(files)}->{len(selected)} files, {total:,} chars', file=sys.stderr)
