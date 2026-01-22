import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { headers } from 'next/headers';

/**
 * ONB-005 & ONB-006: GitHub Organization API
 *
 * Handles GitHub organization validation and GitOps repository creation:
 * - Validates that the GitHub organization exists
 * - Checks if the 5D Labs GitHub App is installed on the org
 * - Creates the customer GitOps repository from template
 */

interface GitHubRequest {
  action: 'validate' | 'create_repo';
  githubOrg: string;
  provider?: string;
  region?: string;
  size?: string;
}

// GitHub App configuration
const GITHUB_APP_NAME = '5dlabs-cto';
const GITHUB_APP_ID = process.env.GITHUB_APP_ID || '';
const GITHUB_APP_PRIVATE_KEY = process.env.GITHUB_APP_PRIVATE_KEY || '';
const TEMPLATE_REPO = '5dlabs/cto-argocd-template';
const TARGET_REPO_NAME = 'cto-argocd';

/**
 * Generate JWT for GitHub App authentication
 */
async function generateAppJWT(): Promise<string> {
  if (!GITHUB_APP_ID || !GITHUB_APP_PRIVATE_KEY) {
    throw new Error('GitHub App credentials not configured');
  }

  // In production, use proper JWT library
  // For now, we'll use a mock or environment token
  const appToken = process.env.GITHUB_APP_JWT;
  if (appToken) {
    return appToken;
  }

  throw new Error('GitHub App JWT not available');
}

/**
 * Get installation access token for an organization
 */
async function getInstallationToken(org: string): Promise<string | null> {
  try {
    const appJWT = await generateAppJWT();

    // Get installation for the org
    const installationResponse = await fetch(`https://api.github.com/orgs/${org}/installation`, {
      headers: {
        Authorization: `Bearer ${appJWT}`,
        Accept: 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
      },
    });

    if (installationResponse.status === 404) {
      return null; // App not installed
    }

    if (!installationResponse.ok) {
      console.error('[ONB-005] Failed to get installation:', installationResponse.statusText);
      return null;
    }

    const installation = await installationResponse.json();

    // Get installation access token
    const tokenResponse = await fetch(
      `https://api.github.com/app/installations/${installation.id}/access_tokens`,
      {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${appJWT}`,
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
        },
      }
    );

    if (!tokenResponse.ok) {
      console.error('[ONB-005] Failed to get installation token:', tokenResponse.statusText);
      return null;
    }

    const tokenData = await tokenResponse.json();
    return tokenData.token;
  } catch (error) {
    console.error('[ONB-005] Error getting installation token:', error);
    return null;
  }
}

/**
 * ONB-005: Validate GitHub organization and app installation
 */
async function validateGitHubOrg(org: string): Promise<{
  valid: boolean;
  orgExists: boolean;
  appInstalled: boolean;
  installUrl?: string;
  error?: string;
}> {
  try {
    // First, check if the org exists (public API)
    const orgResponse = await fetch(`https://api.github.com/orgs/${org}`, {
      headers: {
        Accept: 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
      },
    });

    if (orgResponse.status === 404) {
      return {
        valid: false,
        orgExists: false,
        appInstalled: false,
        error: `Organization "${org}" not found on GitHub`,
      };
    }

    if (!orgResponse.ok) {
      return {
        valid: false,
        orgExists: false,
        appInstalled: false,
        error: `Failed to verify organization: ${orgResponse.statusText}`,
      };
    }

    // Check if GitHub App is installed
    const installationToken = await getInstallationToken(org);

    if (!installationToken) {
      // In development mode, simulate app being installed
      if (process.env.NODE_ENV === 'development') {
        console.log(`[ONB-005] Development mode: simulating app installed on ${org}`);
        return {
          valid: true,
          orgExists: true,
          appInstalled: true,
        };
      }

      return {
        valid: true,
        orgExists: true,
        appInstalled: false,
        installUrl: `https://github.com/apps/${GITHUB_APP_NAME}/installations/new/permissions?target_id=${org}`,
      };
    }

    return {
      valid: true,
      orgExists: true,
      appInstalled: true,
    };
  } catch (error) {
    console.error('[ONB-005] GitHub validation error:', error);
    return {
      valid: false,
      orgExists: false,
      appInstalled: false,
      error: 'Failed to validate GitHub organization',
    };
  }
}

/**
 * ONB-006: Create customer GitOps repository
 */
async function createGitOpsRepo(
  org: string,
  config: {
    provider?: string;
    region?: string;
    size?: string;
    tenantId: string;
  }
): Promise<{
  success: boolean;
  repoUrl?: string;
  error?: string;
}> {
  try {
    const installationToken = await getInstallationToken(org);

    // In development mode, simulate repo creation
    if (!installationToken && process.env.NODE_ENV === 'development') {
      console.log(`[ONB-006] Development mode: simulating repo creation for ${org}`);
      return {
        success: true,
        repoUrl: `https://github.com/${org}/${TARGET_REPO_NAME}`,
      };
    }

    if (!installationToken) {
      return {
        success: false,
        error: 'GitHub App not installed on organization',
      };
    }

    // Check if repo already exists
    const existingRepoResponse = await fetch(
      `https://api.github.com/repos/${org}/${TARGET_REPO_NAME}`,
      {
        headers: {
          Authorization: `Bearer ${installationToken}`,
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
        },
      }
    );

    if (existingRepoResponse.ok) {
      console.log(`[ONB-006] Repo ${org}/${TARGET_REPO_NAME} already exists`);
      return {
        success: true,
        repoUrl: `https://github.com/${org}/${TARGET_REPO_NAME}`,
      };
    }

    // Create repo from template
    const createResponse = await fetch(`https://api.github.com/repos/${TEMPLATE_REPO}/generate`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${installationToken}`,
        Accept: 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
      },
      body: JSON.stringify({
        owner: org,
        name: TARGET_REPO_NAME,
        description: 'ArgoCD GitOps configuration for CTO Managed Dedicated cluster',
        private: false, // Can be configured per customer preference
        include_all_branches: false,
      }),
    });

    if (!createResponse.ok) {
      const errorData = await createResponse.json();
      console.error('[ONB-006] Failed to create repo:', errorData);
      return {
        success: false,
        error: errorData.message || 'Failed to create repository',
      };
    }

    const newRepo = await createResponse.json();
    console.log(`[ONB-006] Created repo: ${newRepo.full_name}`);

    // Update tenant-specific values.yaml
    await updateTenantConfig(org, installationToken, config);

    return {
      success: true,
      repoUrl: newRepo.html_url,
    };
  } catch (error) {
    console.error('[ONB-006] Repo creation error:', error);
    return {
      success: false,
      error: 'Failed to create GitOps repository',
    };
  }
}

/**
 * Update tenant-specific configuration in the GitOps repo
 */
async function updateTenantConfig(
  org: string,
  token: string,
  config: {
    provider?: string;
    region?: string;
    size?: string;
    tenantId: string;
  }
): Promise<void> {
  try {
    // Generate tenant-specific values.yaml content
    const valuesContent = `# Tenant Configuration for ${config.tenantId}
# Generated by CTO Onboarding - DO NOT EDIT MANUALLY

tenant:
  id: ${config.tenantId}
  provider: ${config.provider || 'latitude'}
  region: ${config.region || 'DAL'}
  clusterSize: ${config.size || 'medium'}

cluster:
  name: ${config.tenantId.replace('tenant-', '')}-prod

argocd:
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true

# Runtime components to deploy
applications:
  - name: agent-controller
    enabled: true
  - name: mcp-tools
    enabled: true
  - name: cilium
    enabled: true
  - name: local-path-provisioner
    enabled: true
`;

    // Create or update values.yaml
    // First, check if file exists to get SHA
    const getFileResponse = await fetch(
      `https://api.github.com/repos/${org}/${TARGET_REPO_NAME}/contents/values.yaml`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
        },
      }
    );

    let sha: string | undefined;
    if (getFileResponse.ok) {
      const existingFile = await getFileResponse.json();
      sha = existingFile.sha;
    }

    // Create/update the file
    const updateResponse = await fetch(
      `https://api.github.com/repos/${org}/${TARGET_REPO_NAME}/contents/values.yaml`,
      {
        method: 'PUT',
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
        },
        body: JSON.stringify({
          message: `chore: configure tenant ${config.tenantId}`,
          content: Buffer.from(valuesContent).toString('base64'),
          ...(sha ? { sha } : {}),
        }),
      }
    );

    if (!updateResponse.ok) {
      console.error('[ONB-006] Failed to update values.yaml:', updateResponse.statusText);
    } else {
      console.log(`[ONB-006] Updated values.yaml for ${config.tenantId}`);
    }
  } catch (error) {
    console.error('[ONB-006] Failed to update tenant config:', error);
  }
}

export async function POST(request: NextRequest) {
  const session = await auth.api.getSession({
    headers: await headers(),
  });

  if (!session) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  let body: GitHubRequest;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid request body' }, { status: 400 });
  }

  const { action, githubOrg, provider, region, size } = body;

  if (!githubOrg) {
    return NextResponse.json({ error: 'Missing required field: githubOrg' }, { status: 400 });
  }

  // Derive tenant ID from user
  const tenantId = `tenant-${session.user.id.slice(0, 8)}`;

  console.log(`[ONB-005/006] ${action} request for org: ${githubOrg}, tenant: ${tenantId}`);

  if (action === 'validate') {
    const result = await validateGitHubOrg(githubOrg);

    if (!result.orgExists) {
      return NextResponse.json({ error: result.error }, { status: 400 });
    }

    return NextResponse.json({
      valid: result.valid,
      orgExists: result.orgExists,
      appInstalled: result.appInstalled,
      installUrl: result.installUrl,
    });
  }

  if (action === 'create_repo') {
    const result = await createGitOpsRepo(githubOrg, {
      provider,
      region,
      size,
      tenantId,
    });

    if (!result.success) {
      return NextResponse.json({ error: result.error }, { status: 500 });
    }

    return NextResponse.json({
      success: true,
      repoUrl: result.repoUrl,
      repoName: `${githubOrg}/${TARGET_REPO_NAME}`,
    });
  }

  return NextResponse.json({ error: `Unknown action: ${action}` }, { status: 400 });
}
