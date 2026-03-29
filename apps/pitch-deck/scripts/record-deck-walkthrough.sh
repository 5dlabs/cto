#!/usr/bin/env bash
# Open the live pitch deck and print a short recording checklist.
# Usage: ./scripts/record-deck-walkthrough.sh
#        PITCH_DECK_URL=http://localhost:3010 ./scripts/record-deck-walkthrough.sh

set -euo pipefail

URL="${PITCH_DECK_URL:-https://pitch.5dlabs.ai}"

echo "Opening pitch deck: $URL"
echo ""
echo "Recording checklist:"
echo "  [ ] 1080p (or 1440p) capture · quiet room · mic levels ~-12 to -6 dB"
echo "  [ ] Fullscreen or clean window · hide unrelated tabs"
echo "  [ ] Narration script: apps/pitch-deck/docs/video-walkthrough-script.md"
echo "  [ ] Navigate with arrow keys or deck chrome for smooth scroll"
echo ""

case "$(uname -s)" in
  Darwin) open "$URL" ;;
  Linux) xdg-open "$URL" 2>/dev/null || sensible-browser "$URL" 2>/dev/null || echo "Open manually: $URL" ;;
  *) echo "Open manually: $URL" ;;
esac
