#!/bin/bash
# Vault to OpenBao Secret Migration Script
# Exports all secrets from HashiCorp Vault and imports them into OpenBao

set -e

VAULT_NS="vault"
OPENBAO_NS="openbao"

echo "=========================================="
echo "Migrating secrets from Vault to OpenBao"
echo "=========================================="

# Get list of all secret paths from Vault
SECRETS=$(kubectl exec -n $VAULT_NS vault-0 -- vault kv list -format=json secret/ 2>/dev/null | python3 -c "import sys,json; print('\n'.join(json.load(sys.stdin)))")

# Also get nested paths
NESTED_CTO=$(kubectl exec -n $VAULT_NS vault-0 -- vault kv list -format=json secret/cto/ 2>/dev/null | python3 -c "import sys,json; print('\n'.join(['cto/'+x for x in json.load(sys.stdin)]))" 2>/dev/null || echo "")
NESTED_PROVIDERS=$(kubectl exec -n $VAULT_NS vault-0 -- vault kv list -format=json secret/providers/ 2>/dev/null | python3 -c "import sys,json; print('\n'.join(['providers/'+x for x in json.load(sys.stdin)]))" 2>/dev/null || echo "")

# Combine all paths, excluding directories (ending with /)
ALL_PATHS=$(echo -e "$SECRETS\n$NESTED_CTO\n$NESTED_PROVIDERS" | grep -v '/$' | sort -u)

echo "Found secrets to migrate:"
echo "$ALL_PATHS"
echo ""

# Migrate each secret
while IFS= read -r path; do
    [ -z "$path" ] && continue
    
    echo -n "Migrating: $path ... "
    
    # Get secret from Vault as JSON
    SECRET_JSON=$(kubectl exec -n $VAULT_NS vault-0 -- vault kv get -format=json "secret/$path" 2>/dev/null)
    
    if [ -z "$SECRET_JSON" ]; then
        echo "SKIP (not found)"
        continue
    fi
    
    # Extract just the data portion
    DATA_JSON=$(echo "$SECRET_JSON" | python3 -c "
import sys, json
data = json.load(sys.stdin)
kv = data.get('data', {}).get('data', {})
if kv:
    print(json.dumps(kv))
else:
    sys.exit(1)
" 2>/dev/null)
    
    if [ -z "$DATA_JSON" ]; then
        echo "SKIP (empty)"
        continue
    fi
    
    # Write to OpenBao using JSON input (stdin with -)
    if echo "$DATA_JSON" | kubectl exec -i -n $OPENBAO_NS openbao-0 -- bao kv put "secret/$path" - >/dev/null 2>&1; then
        echo "OK"
    else
        echo "FAILED"
    fi
done <<< "$ALL_PATHS"

echo ""
echo "=========================================="
echo "Migration complete!"
echo "=========================================="

# Verify by listing secrets in OpenBao
echo ""
echo "Verifying OpenBao secrets:"
kubectl exec -n $OPENBAO_NS openbao-0 -- bao kv list secret/

