#!/bin/bash
for f in infra/gitops/applications/*.yaml; do
  [ -f "$f" ] || continue
  grep -q "app.kubernetes.io/name:" "$f" || (echo "Missing required label in $f" && exit 1)
done