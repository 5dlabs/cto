/**
 * Tauri API bindings for CTO App
 * 
 * These functions wrap Tauri's invoke() to call Rust backend commands
 * with full TypeScript type safety.
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

/** Docker runtime information */
export interface DockerInfo {
  installed: boolean;
  running: boolean;
  version: string | null;
  runtime: 'docker' | 'podman' | 'orbstack' | 'colima' | 'rancherDesktop' | 'unknown' | 'notFound';
}

/** Kind installation info */
export interface KindInfo {
  installed: boolean;
  version: string | null;
}

/** Kubernetes node info */
export interface NodeInfo {
  name: string;
  status: string;
  role: string;
}

/** Kind cluster info */
export interface ClusterInfo {
  exists: boolean;
  name: string;
  running: boolean;
  nodes: NodeInfo[];
}

/** Stack selection for backend preference */
export type StackSelection = 'nova' | 'grizz';

/** API keys configuration status */
export interface ApiKeysConfigured {
  anthropic: boolean;
  openai: boolean;
  github: boolean;
}

/** Setup wizard state */
export interface SetupState {
  currentStep: number;
  completed: boolean;
  stackSelection: StackSelection | null;
  apiKeysConfigured: ApiKeysConfigured;
  dockerVerified: boolean;
  clusterCreated: boolean;
}

/** Workflow status from Argo */
export interface WorkflowStatus {
  name: string;
  namespace: string;
  phase: string;
  startedAt: string | null;
  finishedAt: string | null;
  progress: string | null;
  message: string | null;
}

/** Workflow node (step) status */
export interface WorkflowNode {
  id: string;
  name: string;
  displayName: string;
  nodeType: string;
  phase: string;
  startedAt: string | null;
  finishedAt: string | null;
  message: string | null;
}

/** Workflow detail with nodes */
export interface WorkflowDetail {
  status: WorkflowStatus;
  nodes: WorkflowNode[];
}

// ============================================================================
// Setup Commands
// ============================================================================

/** Check Docker installation and daemon status */
export async function checkDocker(): Promise<DockerInfo> {
  return invoke<DockerInfo>('check_docker');
}

/** Check Kind installation */
export async function checkKind(): Promise<KindInfo> {
  return invoke<KindInfo>('check_kind');
}

/** Get current setup wizard state */
export async function getSetupState(): Promise<SetupState> {
  return invoke<SetupState>('get_setup_state');
}

/** Save setup wizard state */
export async function saveSetupState(state: SetupState): Promise<void> {
  return invoke('save_setup_state', { setupState: state });
}

/** Mark setup as complete */
export async function completeSetup(): Promise<void> {
  return invoke('complete_setup');
}

// ============================================================================
// Keychain Commands
// ============================================================================

export type ApiKeyType = 'anthropic' | 'openai' | 'github';

/** Store an API key in the system keychain */
export async function storeApiKey(keyType: ApiKeyType, value: string): Promise<void> {
  return invoke('store_api_key', { keyType, value });
}

/** Get an API key from the system keychain */
export async function getApiKey(keyType: ApiKeyType): Promise<string | null> {
  return invoke<string | null>('get_api_key', { keyType });
}

/** Delete an API key from the system keychain */
export async function deleteApiKey(keyType: ApiKeyType): Promise<void> {
  return invoke('delete_api_key', { keyType });
}

/** Check if an API key exists in the keychain */
export async function hasApiKey(keyType: ApiKeyType): Promise<boolean> {
  return invoke<boolean>('has_api_key', { keyType });
}

// ============================================================================
// Cluster Commands
// ============================================================================

/** Create the CTO App Kind cluster */
export async function createCluster(): Promise<void> {
  return invoke('create_cluster');
}

/** Delete the CTO App Kind cluster */
export async function deleteCluster(): Promise<void> {
  return invoke('delete_cluster');
}

/** Get cluster status */
export async function getClusterStatus(): Promise<ClusterInfo> {
  return invoke<ClusterInfo>('get_cluster_status');
}

/** List all Kind clusters */
export async function listClusters(): Promise<string[]> {
  return invoke<string[]>('list_clusters');
}

// ============================================================================
// Workflow Commands
// ============================================================================

/** Trigger a new workflow */
export async function triggerWorkflow(
  repoUrl: string,
  prompt: string,
  branch?: string,
  stack?: 'grizz' | 'nova'
): Promise<string> {
  return invoke<string>('trigger_workflow', { repoUrl, prompt, branch, stack });
}

/** Get workflow status */
export async function getWorkflowStatus(workflowName: string): Promise<WorkflowDetail> {
  return invoke<WorkflowDetail>('get_workflow_status', { workflowName });
}

/** List all workflows */
export async function listWorkflows(): Promise<WorkflowStatus[]> {
  return invoke<WorkflowStatus[]>('list_workflows');
}

/** Get workflow logs */
export async function getWorkflowLogs(workflowName: string, nodeName?: string): Promise<string> {
  return invoke<string>('get_workflow_logs', { workflowName, nodeName });
}

/** Delete a workflow */
export async function deleteWorkflow(workflowName: string): Promise<void> {
  return invoke('delete_workflow', { workflowName });
}

/** Stop a running workflow */
export async function stopWorkflow(workflowName: string): Promise<void> {
  return invoke('stop_workflow', { workflowName });
}

/** Check if Argo Workflows is available */
export async function checkArgo(): Promise<boolean> {
  return invoke<boolean>('check_argo');
}

// ============================================================================
// Helm Commands
// ============================================================================

/** Helm release information */
export interface HelmRelease {
  name: string;
  namespace: string;
  revision: number;
  status: string;
  chart: string;
  appVersion: string;
}

/** Helm values for deployment */
export interface HelmValues {
  anthropicApiKey?: string;
  openaiApiKey?: string;
  githubToken?: string;
  cloudflareTunnelToken?: string;
  stack?: 'grizz' | 'nova';
}

/** Check if Helm is installed */
export async function checkHelm(): Promise<string | null> {
  return invoke<string | null>('check_helm');
}

/** Deploy the CTO App Helm chart */
export async function deployChart(values: HelmValues): Promise<void> {
  return invoke('deploy_chart', { values });
}

/** Get the status of the Helm release */
export async function getReleaseStatus(): Promise<HelmRelease | null> {
  return invoke<HelmRelease | null>('get_release_status');
}

/** Uninstall the CTO App Helm chart */
export async function uninstallChart(): Promise<void> {
  return invoke('uninstall_chart');
}

/** Update Helm dependencies */
export async function updateHelmDependencies(): Promise<void> {
  return invoke('update_helm_dependencies');
}
