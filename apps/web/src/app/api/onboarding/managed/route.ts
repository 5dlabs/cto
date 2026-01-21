import { NextRequest, NextResponse } from "next/server";
import { auth } from "@/lib/auth";
import { headers } from "next/headers";

/**
 * Managed Dedicated Onboarding API
 *
 * Handles the provisioning workflow for Tier 2 Managed Dedicated clusters.
 * Creates a BoltRun CRD in the cto-admin namespace to trigger the installer.
 */

interface ProvisionRequest {
  action: "provision" | "status";
  provider?: string;
  region?: string;
  size?: string;
  githubOrg?: string;
}

interface BoltRunSpec {
  tenantRef: string;
  taskType: "provision" | "debug" | "upgrade" | "destroy";
  provision: {
    provider: string;
    region: string;
    clusterSize: string;
    credentialRef: string;
    gitopsRepo?: string;
  };
  execution: {
    timeout: string;
    retryLimit: number;
    model: string;
  };
}

// Cluster size configurations
const CLUSTER_CONFIGS: Record<string, { nodes: number; cpNodes: number; workerNodes: number; plan: string }> = {
  small: { nodes: 2, cpNodes: 1, workerNodes: 1, plan: "c2-small-x86" },
  medium: { nodes: 4, cpNodes: 1, workerNodes: 3, plan: "c2-medium-x86" },
  large: { nodes: 8, cpNodes: 3, workerNodes: 5, plan: "c2-large-x86" },
};

export async function POST(request: NextRequest) {
  const session = await auth.api.getSession({
    headers: await headers(),
  });

  if (!session) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  let body: ProvisionRequest;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: "Invalid request body" }, { status: 400 });
  }

  const { action, provider, region, size, githubOrg } = body;

  if (action === "provision") {
    if (!provider || !region || !size) {
      return NextResponse.json(
        { error: "Missing required fields: provider, region, size" },
        { status: 400 }
      );
    }

    // ONB-005/006: GitHub org is now required for Tier 2
    if (!githubOrg) {
      return NextResponse.json(
        { error: "Missing required field: githubOrg" },
        { status: 400 }
      );
    }

    const clusterConfig = CLUSTER_CONFIGS[size];
    if (!clusterConfig) {
      return NextResponse.json(
        { error: `Invalid cluster size: ${size}` },
        { status: 400 }
      );
    }

    // Derive tenant ID from user
    const tenantId = `tenant-${session.user.id.slice(0, 8)}`;
    const clusterName = `${tenantId.replace("tenant-", "")}-prod`;

    // Create BoltRun spec with GitOps repo from ONB-006
    const boltRunSpec: BoltRunSpec = {
      tenantRef: tenantId,
      taskType: "provision",
      provision: {
        provider,
        region,
        clusterSize: size,
        credentialRef: `tenants/${tenantId}/provider-creds`,
        gitopsRepo: `https://github.com/${githubOrg}/cto-argocd`,
      },
      execution: {
        timeout: "30m",
        retryLimit: 3,
        model: "claude-sonnet-4-20250514",
      },
    };

    // In production, this would create the actual BoltRun CRD via Kubernetes API
    // For now, we'll log and return success
    console.log(`[Managed Onboarding] Creating BoltRun for ${tenantId}:`, JSON.stringify(boltRunSpec, null, 2));

    // Check if we have K8s API access
    const k8sApiServer = process.env.KUBERNETES_SERVICE_HOST;
    const k8sToken = process.env.KUBERNETES_SERVICE_ACCOUNT_TOKEN;

    if (k8sApiServer && k8sToken) {
      try {
        // Create BoltRun CRD
        const boltRunManifest = {
          apiVersion: "cto.5dlabs.ai/v1alpha1",
          kind: "BoltRun",
          metadata: {
            name: `bolt-${clusterName}-provision`,
            namespace: "cto-admin",
            labels: {
              "cto.5dlabs.ai/tenant": tenantId,
              "cto.5dlabs.ai/task-type": "provision",
            },
          },
          spec: boltRunSpec,
        };

        const response = await fetch(
          `https://${k8sApiServer}/apis/cto.5dlabs.ai/v1alpha1/namespaces/cto-admin/boltruns`,
          {
            method: "POST",
            headers: {
              "Authorization": `Bearer ${k8sToken}`,
              "Content-Type": "application/json",
            },
            body: JSON.stringify(boltRunManifest),
          }
        );

        if (!response.ok) {
          const error = await response.text();
          console.error("[Managed Onboarding] Failed to create BoltRun:", error);
          return NextResponse.json(
            { error: "Failed to create provisioning task" },
            { status: 500 }
          );
        }

        const createdBoltRun = await response.json();
        console.log(`[Managed Onboarding] BoltRun created: ${createdBoltRun.metadata.name}`);

        return NextResponse.json({
          success: true,
          boltRunName: createdBoltRun.metadata.name,
          tenantId,
          clusterName,
          config: {
            provider,
            region,
            size,
            nodes: clusterConfig.nodes,
            plan: clusterConfig.plan,
            githubOrg,
            gitopsRepo: `https://github.com/${githubOrg}/cto-argocd`,
          },
        });

      } catch (error) {
        console.error("[Managed Onboarding] K8s API error:", error);
        return NextResponse.json(
          { error: "Failed to create provisioning task" },
          { status: 500 }
        );
      }
    }

    // Development mode - return mock success
    console.log("[Managed Onboarding] Development mode - BoltRun would be created");
    return NextResponse.json({
      success: true,
      boltRunName: `bolt-${clusterName}-provision`,
      tenantId,
      clusterName,
      config: {
        provider,
        region,
        size,
        nodes: clusterConfig.nodes,
        plan: clusterConfig.plan,
        githubOrg,
        gitopsRepo: `https://github.com/${githubOrg}/cto-argocd`,
      },
      development: true,
    });
  }

  if (action === "status") {
    // TODO: Query BoltRun status from K8s API
    return NextResponse.json({
      status: "pending",
      message: "Status check not yet implemented",
    });
  }

  return NextResponse.json(
    { error: `Unknown action: ${action}` },
    { status: 400 }
  );
}

export async function GET() {
  const session = await auth.api.getSession({
    headers: await headers(),
  });

  if (!session) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  // Return list of available providers and their configurations
  return NextResponse.json({
    providers: [
      {
        id: "latitude",
        name: "Latitude.sh",
        status: "supported",
        regions: [
          { id: "DAL", name: "Dallas" },
          { id: "NYC", name: "New York" },
          { id: "LAX", name: "Los Angeles" },
          { id: "MIA", name: "Miami" },
          { id: "CHI", name: "Chicago" },
          { id: "ASH", name: "Ashburn" },
          { id: "FRA", name: "Frankfurt" },
          { id: "LON", name: "London" },
          { id: "SYD", name: "Sydney" },
          { id: "TYO", name: "Tokyo" },
          { id: "SAO", name: "São Paulo" },
        ],
      },
      {
        id: "hetzner",
        name: "Hetzner",
        status: "coming_soon",
        regions: [],
      },
      {
        id: "vultr",
        name: "Vultr",
        status: "coming_soon",
        regions: [],
      },
    ],
    sizes: [
      { id: "small", name: "Small", nodes: 2, estimate: "$200/mo" },
      { id: "medium", name: "Medium", nodes: 4, estimate: "$600/mo" },
      { id: "large", name: "Large (HA)", nodes: 8, estimate: "$1,500/mo" },
    ],
  });
}
