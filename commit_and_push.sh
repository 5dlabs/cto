#!/bin/bash
cd /workspace

echo "=== Git Status ==="
git status

echo "=== Adding changes ==="
git add .

echo "=== Committing changes ==="
git commit -m "Fix trader application validation issues

- Add allow-auto-prune label to trader.yaml to fix OPA policy validation
- Add trader namespace to platform project destinations
- Ensure all GitOps validation checks pass"

echo "=== Pushing to feature branch ==="
git push

echo "=== Done ==="