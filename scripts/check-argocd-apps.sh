#!/bin/bash
for f in infra/gitops/applications/*.yaml; do
  [ -f "$f" ] || continue
  grep -q "kind: Application" "$f" || exit 1
done