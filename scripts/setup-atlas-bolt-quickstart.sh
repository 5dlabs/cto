#!/bin/bash
# Quickstart: Complete setup for Atlas and Bolt agents
# Runs all steps in order with user guidance

set -euo pipefail

echo "════════════════════════════════════════════════════════════════"
echo "║                                                                ║"
echo "║        Atlas & Bolt Agent Setup - Complete Quickstart         ║"
echo "║                                                                ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "This script will set up two new agents:"
echo "  🔗 Atlas - Integration & Merge Specialist"
echo "  ⚡ Bolt - DevOps & Deployment Specialist"
echo ""
echo "Process overview:"
echo "  1. Process avatar images → PNG files"
echo "  2. Upload avatars to .github repo"
echo "  3. Create GitHub Apps (manual in browser)"
echo "  4. Collect and save credentials locally"
echo "  5. Generate configuration files"
echo ""
echo "Prerequisites:"
echo "  ✅ images/Atlas.jpg exists"
echo "  ✅ images/Bolt.jpg exists"
echo "  ✅ You have access to 5dlabs GitHub organization"
echo "  ✅ You have admin access to create GitHub Apps"
echo ""
echo "Press ENTER to begin..."
read -r

# Step 1: Process avatars
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ STEP 1: Processing Avatar Images                             ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

if [ ! -f "images/Atlas.jpg" ] || [ ! -f "images/Bolt.jpg" ]; then
  echo "❌ Avatar images not found!"
  echo "   Expected: images/Atlas.jpg and images/Bolt.jpg"
  exit 1
fi

./scripts/process-agent-avatars.sh atlas images/Atlas.jpg
./scripts/process-agent-avatars.sh bolt images/Bolt.jpg

echo ""
echo "✅ Avatar processing complete"
echo "📁 Files ready in: images/processed/"
echo ""
echo "Press ENTER to continue to avatar upload..."
read -r

# Step 2: Upload avatars
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ STEP 2: Uploading Avatars to .github Repository              ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

read -p "Upload avatars now? (Y/n): " UPLOAD_CHOICE
if [[ "$UPLOAD_CHOICE" =~ ^[Yy]?$ ]]; then
  ./scripts/upload-agent-avatars.sh
  echo ""
  echo "✅ Avatars uploaded to GitHub"
else
  echo "⏭️  Skipping avatar upload (you can run manually later)"
  echo "   Command: ./scripts/upload-agent-avatars.sh"
fi

echo ""
echo "Press ENTER to continue to GitHub App creation..."
read -r

# Step 3: Create GitHub Apps and collect credentials
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ STEP 3: Creating GitHub Apps & Collecting Credentials        ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

./scripts/create-github-apps-local.sh

echo ""
echo "✅ GitHub Apps created and credentials saved"
echo ""
echo "Press ENTER to continue to configuration update..."
read -r

# Step 4: Update configuration files
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ STEP 4: Updating Configuration Files                         ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

echo "Generated configuration files:"
echo "  📄 /tmp/atlas-bolt-values-snippet.yaml"
echo "  📄 /tmp/atlas-bolt-external-secrets.yaml"
echo "  📄 /tmp/vault-commands.sh"
echo ""

read -p "Automatically add to configuration files? (Y/n): " AUTO_UPDATE
if [[ "$AUTO_UPDATE" =~ ^[Yy]?$ ]]; then
  
  # Backup original files
  cp infra/charts/controller/values.yaml infra/charts/controller/values.yaml.backup
  cp infra/secret-store/agent-secrets-external-secrets.yaml infra/secret-store/agent-secrets-external-secrets.yaml.backup
  
  echo "✅ Created backups (.backup files)"
  
  # Append to values.yaml (add before the end of agents section)
  # This requires careful editing - let's provide manual instructions instead for safety
  echo ""
  echo "📝 Manual update required for safety:"
  echo ""
  echo "1️⃣  Add to infra/charts/controller/values.yaml:"
  echo "   cat /tmp/atlas-bolt-values-snippet.yaml"
  echo "   (Add under 'agents:' section, after existing agents)"
  echo ""
  echo "2️⃣  Add to infra/secret-store/agent-secrets-external-secrets.yaml:"
  echo "   cat /tmp/atlas-bolt-external-secrets.yaml"
  echo "   (Add at the end of the file)"
  echo ""
  
else
  echo "⏭️  Skipped automatic update"
  echo "   Review files and add manually when ready"
fi

# Final summary
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║                    SETUP SUMMARY                              ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "✅ Avatar images processed and uploaded"
echo "✅ GitHub Apps created: 5DLabs-Atlas, 5DLabs-Bolt"
echo "✅ Credentials saved locally: .agent-credentials.env"
echo "✅ Configuration files generated"
echo ""
echo "📋 Configuration Files Generated:"
echo "   🔹 /tmp/atlas-bolt-values-snippet.yaml"
echo "   🔹 /tmp/atlas-bolt-external-secrets.yaml"
echo "   🔹 /tmp/vault-commands.sh"
echo ""
echo "📋 Manual Steps Remaining:"
echo ""
echo "1️⃣  Update Helm values:"
echo "   - Edit: infra/charts/controller/values.yaml"
echo "   - Add content from: /tmp/atlas-bolt-values-snippet.yaml"
echo ""
echo "2️⃣  Update ExternalSecrets:"
echo "   - Edit: infra/secret-store/agent-secrets-external-secrets.yaml"
echo "   - Add content from: /tmp/atlas-bolt-external-secrets.yaml"
echo ""
echo "3️⃣  Store credentials in Vault:"
echo "   bash /tmp/vault-commands.sh"
echo ""
echo "4️⃣  Install GitHub Apps to repositories:"
echo "   https://github.com/organizations/5dlabs/settings/installations"
echo ""
echo "5️⃣  Commit and push changes:"
echo "   git add README.md infra/charts/controller/values.yaml infra/secret-store/"
echo "   git commit -m 'feat: add Atlas and Bolt agents'"
echo "   git push"
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "🎉 Setup complete! Your team now has 8 agents!"
echo "════════════════════════════════════════════════════════════════"




