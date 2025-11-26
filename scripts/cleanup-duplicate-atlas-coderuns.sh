#!/bin/bash
# Cleanup duplicate Atlas CodeRuns
# Keeps the oldest CodeRun per PR, deletes the rest

set -euo pipefail

echo "=== Atlas CodeRun Cleanup ==="
echo "Finding duplicate CodeRuns per PR..."

# Get all Atlas CodeRuns with PR numbers
CODERUNS=$(kubectl get coderuns -n cto -l agent=atlas -o json)

# Group by PR number and keep only the oldest
echo "$CODERUNS" | jq -r '.items[] | "\(.metadata.labels["pr-number"] // "none")|\(.metadata.name)|\(.metadata.creationTimestamp)"' \
  | sort -t'|' -k1,1 -k3,3 \
  | awk -F'|' '
    {
      pr=$1
      name=$2
      ts=$3
      if (pr == "none") next
      if (pr != last_pr) {
        # New PR, this is the oldest (first in sorted list)
        print "KEEP", name, pr
        last_pr = pr
      } else {
        # Duplicate for same PR
        print "DELETE", name, pr
      }
    }
  ' > /tmp/atlas-cleanup.txt

KEEP_COUNT=$(grep -c "^KEEP" /tmp/atlas-cleanup.txt || echo 0)
DELETE_COUNT=$(grep -c "^DELETE" /tmp/atlas-cleanup.txt || echo 0)

echo ""
echo "Summary:"
echo "  CodeRuns to KEEP: $KEEP_COUNT"
echo "  CodeRuns to DELETE: $DELETE_COUNT"
echo ""

if [ "$DELETE_COUNT" -eq 0 ]; then
  echo "No duplicates found!"
  exit 0
fi

echo "Duplicates to delete:"
grep "^DELETE" /tmp/atlas-cleanup.txt | awk '{print "  - "$2" (PR #"$3")"}'
echo ""

read -p "Proceed with deletion? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
  echo "Aborted."
  exit 0
fi

echo ""
echo "Deleting duplicate CodeRuns..."
grep "^DELETE" /tmp/atlas-cleanup.txt | while read -r action name pr; do
  echo "  Deleting $name (PR #$pr)..."
  kubectl delete coderun "$name" -n cto --wait=false 2>&1 | grep -v "deleted" || true
done

echo ""
echo "✅ Cleanup complete!"
echo ""
echo "Remaining Atlas CodeRuns:"
kubectl get coderuns -n cto -l agent=atlas --no-headers | wc -l

rm -f /tmp/atlas-cleanup.txt
