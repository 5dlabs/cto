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

/** Full runtime environment scan result */
export interface RuntimeEnvironment {
  runtimes: RuntimeStatus[];
  docker_available: boolean;
  kubernetes_available: boolean;
  recommended: string | null;
  macos_version: string | null;
  can_use_apple_virtualization: boolean;
}

/** Runtime status from environment scan */
export interface RuntimeStatus {
  runtime: string;
  installed: boolean;
  running: boolean;
  version: string | null;
  path: string | null;
  docker_compatible: boolean;
  kubernetes_included: boolean;
  kind_compatible: boolean;
  kind_provider: string | null;
}

// ============================================================================
// Setup Commands
// ============================================================================

/** Check Docker installation and daemon status */
export async function checkDocker(): Promise<DockerInfo> {
  return invoke<DockerInfo>('check_docker');
}

/** Auto-detect and start container runtime */
export async function autoStartRuntime(): Promise<string | null> {
  return invoke<string | null>('auto_start_runtime');
}

/** Fully automated runtime detection and startup - zero touch */
export async function autoDetectAndStartRuntime(): Promise<RuntimeEnvironment> {
  return invoke<RuntimeEnvironment>('auto_detect_and_start_runtime');
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
// Log Streaming Commands
// ============================================================================

/** Log entry from pod */
export interface LogEntry {
  timestamp: string;
  pod: string;
  container: string;
  namespace: string;
  message: string;
}

/** Pod information */
export interface PodInfo {
  name: string;
  phase: string;
  containers: string[];
}

/** Filter for log streaming */
export interface LogStreamFilter {
  namespace?: string;
  pod_pattern?: string;
  container?: string;
  since_seconds?: number;
  tail_lines?: number;
}

/** Get list of namespaces */
export async function listNamespaces(): Promise<string[]> {
  return invoke<string[]>('list_namespaces');
}

/** Get list of pods in a namespace */
export async function listPods(namespace?: string): Promise<string[]> {
  return invoke<string[]>('list_pods', { namespace });
}

/** Get pods with status information */
export async function listPodsWithStatus(namespace?: string): Promise<PodInfo[]> {
  return invoke<PodInfo[]>('list_pods_with_status', { namespace });
}

/** Stream logs from a specific pod */
export async function streamPodLogs(
  podName: string,
  namespace?: string,
  container?: string
): Promise<LogEntry[]> {
  return invoke<LogEntry[]>('stream_pod_logs', { podName, namespace, container });
}

/** Start a log stream */
export async function startLogStream(
  namespace?: string,
  podPattern?: string,
  container?: string
): Promise<string> {
  return invoke<string>('start_log_stream', { namespace, podPattern, container });
}

/** Stop a log stream */
export async function stopLogStream(streamId: string): Promise<void> {
  return invoke('stop_log_stream', { streamId });
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

// ============================================================================
// Smart Initialization Commands
// ============================================================================

/** Smart initialization - fully automated zero-touch setup */
export interface SmartInitResult {
  docker_started: boolean;
  cluster_ready: boolean;
  context: string;
  actions: string[];
  errors: string[];
  needs_user_action: boolean;
  user_message: string | null;
}

export async function smartInit(): Promise<SmartInitResult> {
  return invoke<SmartInitResult>('smart_init');
}

/** Quick health check */
export async function quickHealthCheck(): Promise<{
  docker: { available: boolean; ready: boolean };
  kind: { installed: boolean };
  cluster: { exists: boolean; ready: boolean; name: string };
  overall: boolean;
  message: string;
}> {
  return invoke('quick_health_check');
}
