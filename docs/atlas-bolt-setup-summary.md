# Atlas & Bolt - Secrets Configuration Complete

**Date:** November 3, 2025  
**Status:** ✅ Phase 1 Complete - Credentials Configured

---

## ✅ Completed Actions

### 1. **Updated Helm Values (values.yaml)**
- **File:** `infra/charts/controller/values.yaml`
- **Changes:**
  - Atlas App ID: `2225842`
  - Atlas Client ID: `Iv23liTupEPix4hvGi0w`
  - Bolt App ID: `2225782`
  - Bolt Client ID: `Iv23liYmdPdctJx4YCx2`

### 2. **Added ExternalSecrets Configuration**
- **File:** `infra/secret-store/agent-secrets-external-secrets.yaml`
- **Added 6 ExternalSecret Definitions:**

#### Atlas Secrets:
1. `github-app-5dlabs-atlas` (secret-store namespace)
2. `github-app-5dlabs-atlas-agent-platform` (agent-platform namespace)
3. `github-app-atlas` (agent-platform namespace - short name alias)

#### Bolt Secrets:
4. `github-app-5dlabs-bolt` (secret-store namespace)
5. `github-app-5dlabs-bolt-agent-platform` (agent-platform namespace)
6. `github-app-bolt` (agent-platform namespace - short name alias)

### 3. **Updated MCP Configuration**
- **File:** `cto-config.json`
- **Added Agent Configs:**
  - `atlas`: Claude Sonnet 4, full tool access
  - `bolt`: Claude Sonnet 4, full tool access
- **Total Agents:** 8 (morgan, rex, cleo, tess, blaze, cipher, atlas, bolt)

### 4. **Created Vault Storage Script**
- **File:** `scripts/store-atlas-bolt-vault-credentials.sh`
- **Purpose:** Automates storing credentials in Vault
- **Features:**
  - Validates private key files exist
  - Stores app_id, client_id, private_key for both agents
  - Provides verification commands
  - Includes next steps guidance

---

## 📋 Next Step: Store Credentials in Vault

Run the generated script to store the private keys in Vault:

```bash
cd /Users/jonathonfritz/code/work-projects/5dlabs/cto
./scripts/store-atlas-bolt-vault-credentials.sh
```

**What this script does:**
1. Validates that private key files exist:
   - `keys/atlas-5dlabs.2025-11-03.private-key.pem`
   - `keys/bolt-5dlabs.2025-11-02.private-key.pem`
2. Stores credentials in Vault:
   - `secret/github-app-atlas` (app_id, client_id, private_key)
   - `secret/github-app-bolt` (app_id, client_id, private_key)
3. Provides verification commands

---

## 🔍 Verification Steps

### After Running the Vault Script:

#### 1. Check ExternalSecrets are syncing:
```bash
kubectl get externalsecrets -n secret-store | grep -E 'atlas|bolt'
kubectl get externalsecrets -n agent-platform | grep -E 'atlas|bolt'
```

Expected output: All ExternalSecrets should show `SecretSynced` status

#### 2. Verify secrets were created:
```bash
kubectl get secrets -n agent-platform | grep -E 'atlas|bolt'
```

Expected secrets:
- `github-app-5dlabs-atlas`
- `github-app-5dlabs-bolt`
- `github-app-atlas`
- `github-app-bolt`

#### 3. Inspect a secret to verify contents:
```bash
kubectl get secret github-app-atlas -n agent-platform -o yaml
```

Should contain keys: `app-id`, `client-id`, `private-key`

---

## 📦 Commit Changes

After verifying everything works, commit the configuration:

```bash
git add infra/charts/controller/values.yaml
git add infra/secret-store/agent-secrets-external-secrets.yaml
git add cto-config.json
git add scripts/store-atlas-bolt-vault-credentials.sh
git commit -m 'feat: add Atlas and Bolt agent credentials configuration

- Add GitHub App IDs and Client IDs to values.yaml
- Configure ExternalSecrets for Atlas and Bolt
- Add agents to MCP configuration (cto-config.json)
- Create Vault storage script for credentials'
```

---

## 🚀 What's Next (Remaining Work)

### Phase 2: GitHub App Installation
- [ ] Install Atlas GitHub App to repositories
- [ ] Install Bolt GitHub App to repositories
- [ ] Configure repository permissions

### Phase 3: Workflow Integration Design
- [ ] Define when/how Atlas activates (merge conflicts, PR ready to merge)
- [ ] Define when/how Bolt activates (post-merge, deployment monitoring)
- [ ] Design integration with existing Rex → Cleo → Tess flow
- [ ] Specify event triggers and sensors

### Phase 4: Implementation
- [ ] Add workflow parameters for integration-agent and deployment-agent
- [ ] Create workflow templates for Atlas and Bolt
- [ ] Implement Argo Events sensors
- [ ] Test end-to-end integration

---

## 📊 Progress Summary

| Task | Status |
|------|--------|
| GitHub Apps Created | ✅ Complete |
| App IDs in values.yaml | ✅ Complete |
| ExternalSecrets Configured | ✅ Complete |
| MCP Config Updated | ✅ Complete |
| Vault Script Created | ✅ Complete |
| Store Credentials in Vault | ⏳ **Next Step** |
| Install GitHub Apps | ⏳ Pending |
| Workflow Integration | ⏳ Pending |
| Event Triggers | ⏳ Pending |
| Testing | ⏳ Pending |

**Overall Phase 1 Progress:** 5/5 tasks complete (100%)  
**Overall Project Progress:** 5/12 tasks complete (42%)

---

## 🔑 Credential Reference

### Atlas (Integration Specialist)
- **GitHub App Name:** 5DLabs-Atlas
- **App ID:** 2225842
- **Client ID:** Iv23liTupEPix4hvGi0w
- **Private Key:** `keys/atlas-5dlabs.2025-11-03.private-key.pem`
- **Vault Path:** `secret/github-app-atlas`

### Bolt (DevOps Specialist)
- **GitHub App Name:** 5DLabs-Bolt
- **App ID:** 2225782
- **Client ID:** Iv23liYmdPdctJx4YCx2
- **Private Key:** `keys/bolt-5dlabs.2025-11-02.private-key.pem`
- **Vault Path:** `secret/github-app-bolt`

---

## ⚠️ Security Notes

- Private key files are stored in `keys/` directory (should be in .gitignore)
- Never commit private key files to git
- Vault is the source of truth for credentials
- ExternalSecrets automatically sync from Vault to Kubernetes
- Credentials refresh every 1 hour via ExternalSecrets

---

## 🎯 Success Criteria for Phase 1

✅ All completed:
- [x] GitHub Apps exist with proper permissions
- [x] App IDs and Client IDs configured in values.yaml
- [x] ExternalSecrets configured for both agents
- [x] MCP server knows about both agents
- [x] Vault storage script ready to run

**Next Action:** Run `./scripts/store-atlas-bolt-vault-credentials.sh`



