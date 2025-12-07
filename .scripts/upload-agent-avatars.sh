#!/bin/bash
# Upload processed agent avatars to 5dlabs/.github repository
# This makes them available for GitHub Apps and README

set -euo pipefail

GITHUB_REPO_DIR="${1:-/tmp/5dlabs-github}"

echo "════════════════════════════════════════════════════════════════"
echo "║         Uploading Agent Avatars to .github Repo              ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check if processed avatars exist
if [ ! -d "images/processed" ]; then
  echo "❌ No processed avatars found"
  echo "Run: ./scripts/process-agent-avatars.sh atlas images/Atlas.jpg"
  echo "Run: ./scripts/process-agent-avatars.sh bolt images/Bolt.jpg"
  exit 1
fi

# Clone or update .github repo
if [ -d "$GITHUB_REPO_DIR" ]; then
  echo "📁 Using existing .github repo at: $GITHUB_REPO_DIR"
  cd "$GITHUB_REPO_DIR"
  git pull origin main
else
  echo "📥 Cloning .github repository..."
  git clone git@github.com:5dlabs/.github.git "$GITHUB_REPO_DIR"
  cd "$GITHUB_REPO_DIR"
fi

# Create assets directory if it doesn't exist
mkdir -p profile/assets

# Copy processed avatars
echo "📤 Uploading avatars..."

for avatar in atlas bolt; do
  if [ -f "$(dirs +1)/images/processed/${avatar}-avatar.png" ]; then
    cp "$(dirs +1)/images/processed/${avatar}-avatar.png" "profile/assets/${avatar}-avatar.png"
    echo "✅ Copied ${avatar}-avatar.png"
  else
    echo "⚠️  ${avatar}-avatar.png not found"
  fi
done

# Check if there are changes to commit
if git diff --quiet && git diff --cached --quiet; then
  echo "ℹ️  No changes to commit (avatars already uploaded)"
  exit 0
fi

# Stage, commit, and push
git add profile/assets/atlas-avatar.png profile/assets/bolt-avatar.png

echo ""
echo "📝 Committing changes..."
git commit -m "feat: add Atlas and Bolt agent avatars

- Atlas: Integration & Merge Specialist (gorilla)
- Bolt: DevOps & Deployment Specialist (cheetah)

Avatars for new agents in the CTO multi-agent orchestration platform."

echo "📤 Pushing to GitHub..."
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✅ Avatars uploaded successfully!"
echo ""
echo "📋 Avatar URLs:"
echo "   Atlas: https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/atlas-avatar.png"
echo "   Bolt: https://raw.githubusercontent.com/5dlabs/.github/main/profile/assets/bolt-avatar.png"
echo ""
echo "✅ These URLs are already configured in README.md"
echo "════════════════════════════════════════════════════════════════"

