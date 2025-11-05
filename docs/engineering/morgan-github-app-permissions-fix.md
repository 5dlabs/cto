# Morgan GitHub App Permissions Fix

## Issue

Morgan PM can create GitHub Projects and Issues successfully, but **issues are not being linked to the projects**. 

The logs show success:
```
✅ Added issue #307 to project (Item ID: PVTI_lADOC8B7k84BHR1DzggvgPs)
```

But when checking the actual issue:
```bash
gh issue view 307 --json projectItems --jq '.projectItems'
# Returns: []  ← Issue is NOT linked!
```

## Root Cause

The Morgan GitHub App is **missing the "Organization projects" permission** (Read and write access).

Without this permission, the GraphQL mutation `addProjectV2ItemById` fails silently, and issues cannot be added to Projects v2.

## Solution

### 1. Update Morgan's GitHub App Permissions

Go to: https://github.com/organizations/5dlabs/settings/apps/morgan-5dlabs

1. Click **"Permissions & events"**
2. Scroll to **"Organization permissions"**
3. Find **"Projects"** and change from "No access" to **"Read and write"**
4. Click **"Save changes"**
5. You'll see a message that the app needs to be reinstalled - click **"Install"** or have an admin approve the permission change

### 2. Reinstall on Repositories

After updating permissions, the app must be reinstalled:

1. Go to: https://github.com/organizations/5dlabs/settings/installations
2. Find **"morgan-5dlabs"**
3. Click **"Configure"**
4. Verify the repositories where Morgan should have access
5. Save (this completes the permission grant)

### 3. Verify Permissions Work

Test with a simple GraphQL query:

```bash
# Get installation token (Morgan does this automatically)
INSTALLATION_TOKEN="<token from Morgan's authentication>"

# Try to add an issue to a project
gh api graphql -f query='
  mutation {
    addProjectV2ItemById(input: {
      projectId: "PVT_kwDOC8B7k84BHR1D"
      contentId: "<issue-node-id>"
    }) {
      item {
        id
      }
    }
  }
'
```

If permissions are correct, this will succeed. If not, you'll get an error like:
```
Resource not accessible by integration
```

## What This Enables

Once the permission is granted, Morgan will be able to:

✅ Create GitHub Projects (org-level)
✅ **Add issues to projects** (currently failing)
✅ **Update project item field values** (Status, Agent, Priority)
✅ Link repository to project
✅ Full bidirectional GitHub ↔ Kubernetes sync

## Related Documentation

- GitHub Projects v2 API: https://docs.github.com/en/graphql/reference/mutations#addprojectv2itembyid
- GitHub App permissions: https://docs.github.com/en/apps/creating-github-apps/setting-up-a-github-app/choosing-permissions-for-a-github-app#organization-permissions
- Morgan PM Integration: [morgan-pm-github-projects-integration.md](./morgan-pm-github-projects-integration.md)

## Testing After Fix

After granting the permission, restart the Morgan PM pod or trigger a new workflow:

```bash
# Delete Morgan pod to force restart with new permissions
kubectl delete pod -n agent-platform -l agent=morgan

# Or trigger a new play workflow
task-master-play --task-id 1
```

Morgan should now successfully link issues to projects! 🎯

