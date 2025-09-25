#!/bin/bash
cd /workspace

# Check git status
echo "=== Git Status ==="
git status

# Add all changes
echo "=== Adding changes ==="
git add .

# Commit the changes
echo "=== Committing changes ==="
git commit -m "Fix trader application validation issues

- Add allow-auto-prune label to trader.yaml to fix OPA policy validation
- Add trader namespace to platform project destinations
- Ensure all GitOps validation checks pass"

# Push the changes
echo "=== Pushing changes ==="
git push

echo "=== Done ==="