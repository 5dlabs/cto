#!/bin/bash
# Store Atlas and Bolt GitHub App credentials in Vault
# Run this script after creating the GitHub Apps to store credentials securely

set -euo pipefail

echo "════════════════════════════════════════════════════════════════"
echo "║   Storing Atlas & Bolt Credentials in Vault                  ║"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check if private key files exist
if [ ! -f "keys/atlas-5dlabs.2025-11-03.private-key.pem" ]; then
  echo "❌ Atlas private key not found: keys/atlas-5dlabs.2025-11-03.private-key.pem"
  exit 1
fi

if [ ! -f "keys/bolt-5dlabs.2025-11-02.private-key.pem" ]; then
  echo "❌ Bolt private key not found: keys/bolt-5dlabs.2025-11-02.private-key.pem"
  exit 1
fi

echo "✅ Found private key files"
echo ""

# Store Atlas credentials in Vault
echo "📝 Storing Atlas credentials..."
vault kv put secret/github-app-atlas \
  app_id="2225842" \
  client_id="Iv23liTupEPix4hvGi0w" \
  private_key=@keys/atlas-5dlabs.2025-11-03.private-key.pem

if [ $? -eq 0 ]; then
  echo "✅ Atlas credentials stored successfully"
else
  echo "❌ Failed to store Atlas credentials"
  exit 1
fi

echo ""

# Store Bolt credentials in Vault
echo "📝 Storing Bolt credentials..."
vault kv put secret/github-app-bolt \
  app_id="2225782" \
  client_id="Iv23liYmdPdctJx4YCx2" \
  private_key=@keys/bolt-5dlabs.2025-11-02.private-key.pem

if [ $? -eq 0 ]; then
  echo "✅ Bolt credentials stored successfully"
else
  echo "❌ Failed to store Bolt credentials"
  exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║                   SUCCESS                                     ║"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "✅ All credentials stored in Vault"
echo ""
echo "📋 Vault Keys Created:"
echo "   - secret/github-app-atlas (app_id, client_id, private_key)"
echo "   - secret/github-app-bolt (app_id, client_id, private_key)"
echo ""
echo "📋 Next Steps:"
echo "   1. Verify VaultStaticSecrets are syncing:"
echo "      kubectl get vaultstaticsecrets -n agent-platform | grep -E 'atlas|bolt'"
echo ""
echo "   2. Check that secrets are created:"
echo "      kubectl get secrets -n agent-platform | grep -E 'atlas|bolt'"
echo ""
echo "   3. Commit configuration changes:"
echo "      git add infra/charts/controller/values.yaml"
echo "      git add cto-config.json"
echo "      git commit -m 'feat: add Atlas and Bolt agent credentials'"
echo ""
echo "════════════════════════════════════════════════════════════════"



