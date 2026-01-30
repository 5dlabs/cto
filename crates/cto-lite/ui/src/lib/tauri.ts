/**
 * Tauri API bindings for CTO Lite
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

/** Workflow status */
export interface WorkflowStatus {
  id: string;
  name: string;
  status: string;
  startedAt: string | null;
  finishedAt: string | null;
  message: string | null;
}

/** Workflow trigger request */
export interface WorkflowTriggerRequest {
  repoUrl: string;
  branch?: string;
  prompt: string;
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

/** Create the CTO Lite Kind cluster */
export async function createCluster(): Promise<void> {
  return invoke('create_cluster');
}

/** Delete the CTO Lite Kind cluster */
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
export async function triggerWorkflow(request: WorkflowTriggerRequest): Promise<string> {
  return invoke<string>('trigger_workflow', { request });
}

/** Get workflow status */
export async function getWorkflowStatus(workflowId: string): Promise<WorkflowStatus> {
  return invoke<WorkflowStatus>('get_workflow_status', { workflowId });
}

/** List all workflows */
export async function listWorkflows(): Promise<WorkflowStatus[]> {
  return invoke<WorkflowStatus[]>('list_workflows');
}

/** Get workflow logs */
export async function getWorkflowLogs(workflowId: string, nodeName?: string): Promise<string> {
  return invoke<string>('get_workflow_logs', { workflowId, nodeName });
}
