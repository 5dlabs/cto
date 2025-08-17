#!/bin/bash
for f in infra/gitops/databases/*.yaml; do
  [ -f "$f" ] || continue
  [[ "$f" == *"examples"* ]] && continue
  grep -q "resources:" "$f" || echo "Warning: No resource limits in $f"
done