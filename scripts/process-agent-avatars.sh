#!/bin/bash
# Process agent avatar images for GitHub and documentation
# Creates multiple sizes optimized for different use cases

set -euo pipefail

AGENT_NAME="${1:-}"
INPUT_IMAGE="${2:-}"

if [ -z "$AGENT_NAME" ] || [ -z "$INPUT_IMAGE" ]; then
  echo "Usage: $0 <agent-name> <input-image>"
  echo "Example: $0 atlas images/Atlas.jpg"
  exit 1
fi

if [ ! -f "$INPUT_IMAGE" ]; then
  echo "❌ Input image not found: $INPUT_IMAGE"
  exit 1
fi

AGENT_LOWER=$(echo "$AGENT_NAME" | tr '[:upper:]' '[:lower:]')
OUTPUT_DIR="images/processed"
mkdir -p "$OUTPUT_DIR"

echo "════════════════════════════════════════════════════════════════"
echo "Processing avatar for: $AGENT_NAME"
echo "Input: $INPUT_IMAGE"
echo "Output: $OUTPUT_DIR/"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check if ImageMagick is available
if ! command -v convert >/dev/null 2>&1; then
  echo "⚠️  ImageMagick not found. Attempting to install..."
  
  if command -v brew >/dev/null 2>&1; then
    brew install imagemagick
  elif command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update && sudo apt-get install -y imagemagick
  else
    echo "❌ Please install ImageMagick manually:"
    echo "   macOS: brew install imagemagick"
    echo "   Linux: apt-get install imagemagick"
    exit 1
  fi
fi

# Generate different sizes for different use cases
echo "📐 Generating avatar sizes..."

# GitHub profile avatar (ideal: 512x512 PNG)
convert "$INPUT_IMAGE" \
  -resize 512x512 \
  -background none \
  -gravity center \
  -extent 512x512 \
  "$OUTPUT_DIR/${AGENT_LOWER}-avatar-512.png"
echo "✅ Created: ${AGENT_LOWER}-avatar-512.png (GitHub profile)"

# README avatar (180x180 for consistency with other agents)
convert "$INPUT_IMAGE" \
  -resize 180x180 \
  -background none \
  -gravity center \
  -extent 180x180 \
  "$OUTPUT_DIR/${AGENT_LOWER}-avatar-180.png"
echo "✅ Created: ${AGENT_LOWER}-avatar-180.png (README)"

# Favicon size (64x64)
convert "$INPUT_IMAGE" \
  -resize 64x64 \
  -background none \
  -gravity center \
  -extent 64x64 \
  "$OUTPUT_DIR/${AGENT_LOWER}-avatar-64.png"
echo "✅ Created: ${AGENT_LOWER}-avatar-64.png (Favicon/Small icons)"

# High-res version (1024x1024 PNG)
convert "$INPUT_IMAGE" \
  -resize 1024x1024 \
  -background none \
  -gravity center \
  -extent 1024x1024 \
  "$OUTPUT_DIR/${AGENT_LOWER}-avatar-1024.png"
echo "✅ Created: ${AGENT_LOWER}-avatar-1024.png (High-res)"

# Create optimized version for web (compressed)
convert "$OUTPUT_DIR/${AGENT_LOWER}-avatar-512.png" \
  -strip \
  -quality 85 \
  "$OUTPUT_DIR/${AGENT_LOWER}-avatar.png"
echo "✅ Created: ${AGENT_LOWER}-avatar.png (Optimized for GitHub)"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✅ Avatar processing complete!"
echo ""
echo "📁 Output files in: $OUTPUT_DIR/"
ls -lh "$OUTPUT_DIR/${AGENT_LOWER}-"*
echo ""
echo "📋 Next steps:"
echo "   1. Upload to .github repo: $OUTPUT_DIR/${AGENT_LOWER}-avatar.png"
echo "   2. GitHub will use this as the app avatar"
echo "════════════════════════════════════════════════════════════════"




