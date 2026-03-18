/**
 * Tauri API bindings for CTO
 * 
 * These functions wrap Tauri's invoke() to call Rust backend commands
 * with full TypeScript type safety.
 */

import { invoke } from '@tauri-apps/api/core';

type TauriCommandError = {
  code?: string;
  message?: string;
  error?: TauriCommandError | string;
  cause?: TauriCommandError | string;
};

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

export type InstallStep =
  | 'CheckingPrerequisites'
  | 'InstallingBinaries'
  | 'CreatingCluster'
  | 'PullingImages'
  | 'DeployingServices'
  | 'ConfiguringIngress'
  | 'Complete'
  | 'Failed';

export interface InstallStatus {
  step: InstallStep;
  message: string;
  progress: number;
  error: string | null;
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

/** Scan the local runtime environment without starting anything */
export async function scanRuntimeEnvironment(): Promise<RuntimeEnvironment> {
  return invoke<RuntimeEnvironment>('scan_runtime_environment');
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

export async function runInstallation(): Promise<void> {
  return invoke('run_installation');
}

export async function getInstallStatus(): Promise<boolean> {
  return invoke<boolean>('get_install_status');
}

export async function resetInstallation(): Promise<void> {
  return invoke('reset_installation');
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

/** Create the CTO Kind cluster */
export async function createCluster(): Promise<void> {
  return invoke('create_cluster');
}

/** Delete the CTO Kind cluster */
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

/** Deploy the CTO Helm chart */
export async function deployChart(values: HelmValues): Promise<void> {
  return invoke('deploy_chart', { values });
}

/** Get the status of the Helm release */
export async function getReleaseStatus(): Promise<HelmRelease | null> {
  return invoke<HelmRelease | null>('get_release_status');
}

/** Uninstall the CTO Helm chart */
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

// ============================================================================
// OpenClaw Gateway Commands
// ============================================================================

/** Response from the OpenClaw agent */
export interface OpenClawResponse {
  content: string;
  latencyMs?: number;
  gatewayUrl?: string;
  gatewaySessionKey?: string;
  acpSessionId?: string;
  stopReason?: string;
  action?: {
    type: 'oauth' | 'approve' | 'link' | 'confirm';
    label: string;
    description?: string;
    url?: string;
    workflowId?: string;
    completed?: boolean;
  };
}

/** Workflow start result */
export interface WorkflowStartResult {
  workflowId: string;
  status: string;
}

/** OpenClaw gateway status */
export interface OpenClawStatus {
  connected: boolean;
  version: string | null;
  agents: string[];
}

export interface OpenClawMessage {
  role: string;
  content: string;
}

/** Local bridge status for connecting CTO to the Morgan OpenClaw service */
export interface OpenClawBridgeStatus {
  running: boolean;
  connected: boolean;
  pid: number | null;
  namespace: string | null;
  service: string | null;
  localUrl: string;
}

export interface LocalMorganHealth {
  expectedContext: string;
  activeContext: string | null;
  dockerAvailable: boolean;
  kindContextConfigured: boolean;
  kindClusterExists: boolean;
  kindContextReachable: boolean;
  ingressControllerReady: boolean;
  morganDeploymentReady: boolean;
  morganServicePresent: boolean;
  morganIngressHost: string | null;
  ctoToolsReady: boolean;
  ctoOpenmemoryReady: boolean;
  natsReady: boolean;
  gatewayReachable: boolean;
  problems: string[];
}

export interface MorganDiagnostics {
  healthy: boolean;
  modelPrimary: string | null;
  modelFallbacks: string[];
  catalogSource: string | null;
  catalogGeneratedAt: string | null;
  catalogProviderCount: number;
  catalogModelCount: number;
  recentErrors: string[];
}

export interface ProjectRecord {
  id: string;
  name: string;
  summary: string;
  repository: string | null;
  prdTitle: string;
  prdContent: string;
  workflowSummary: string;
  workflowNotes: string;
  configNotes: string;
}

export interface AgentUiConfig {
  id: string;
  displayName: string;
  role: string;
  summary: string;
  avatarLabel: string;
  enabled: boolean;
  skills: string[];
  capabilities: string[];
  tools: string[];
  systemPrompt: string;
  heartbeatEvery: string;
  model: string;
}

export interface StudioState {
  selectedProjectId: string;
  projects: ProjectRecord[];
  agents: AgentUiConfig[];
}

export interface RenderedAgentConfig {
  agentId: string;
  projectId: string | null;
  target: string;
  renderedAt: string;
  content: string;
}

export interface ApplyAgentConfigResult {
  applied: boolean;
  agentId: string;
  projectId: string | null;
  target: string;
  renderedAt: string;
  message: string;
}

export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  if (error && typeof error === 'object') {
    const candidate = error as TauriCommandError;

    if (typeof candidate.message === 'string' && candidate.message.trim()) {
      return candidate.message;
    }

    if (candidate.error) {
      return getErrorMessage(candidate.error);
    }

    if (candidate.cause) {
      return getErrorMessage(candidate.cause);
    }
  }

  return 'Unknown error';
}

/** Send a message to the OpenClaw PM agent (Morgan) */
export async function openclawSendMessage(
  sessionId: string,
  message: string,
  agentId?: string
): Promise<OpenClawResponse> {
  return invoke<OpenClawResponse>('openclaw_send_message', {
    sessionId,
    message,
    agentId: agentId ?? null,
  });
}

/** Send pasted supporting context into the active Morgan avatar room session */
export async function openclawSendAvatarContext(
  roomName: string,
  content: string,
  agentId?: string
): Promise<OpenClawResponse> {
  return invoke<OpenClawResponse>('openclaw_send_avatar_context', {
    roomName,
    content,
    agentId: agentId ?? null,
  });
}

/** Get message history for a session */
export async function openclawGetMessages(
  sessionId: string
): Promise<OpenClawMessage[]> {
  return invoke<OpenClawMessage[]>('openclaw_get_messages', { sessionId });
}

/** Start a Lobster workflow via OpenClaw */
export async function openclawStartWorkflow(
  workflowType: string,
  params: Record<string, string>
): Promise<WorkflowStartResult> {
  return invoke<WorkflowStartResult>('openclaw_start_workflow', {
    workflowType,
    params,
  });
}

/** Get workflow status */
export async function openclawGetWorkflowStatus(
  workflowId: string
): Promise<WorkflowStartResult> {
  return invoke<WorkflowStartResult>('openclaw_get_workflow_status', {
    workflowId,
  });
}

/** Approve a pending workflow step */
export async function openclawApprove(workflowId: string): Promise<void> {
  return invoke('openclaw_approve', { workflowId });
}

/** Reject a pending workflow step */
export async function openclawReject(
  workflowId: string,
  reason: string
): Promise<void> {
  return invoke('openclaw_reject', { workflowId, reason });
}

/** Get OpenClaw gateway connection status */
export async function openclawGetStatus(): Promise<OpenClawStatus> {
  return invoke<OpenClawStatus>('openclaw_get_status');
}

/** Start the local Morgan bridge */
export async function openclawStartLocalBridge(agentId?: string): Promise<OpenClawBridgeStatus> {
  return invoke<OpenClawBridgeStatus>('openclaw_start_local_bridge', { agentId: agentId ?? null });
}

/** Stop the local Morgan bridge */
export async function openclawStopLocalBridge(agentId?: string): Promise<OpenClawBridgeStatus> {
  return invoke<OpenClawBridgeStatus>('openclaw_stop_local_bridge', { agentId: agentId ?? null });
}

/** Get the local Morgan bridge status */
export async function openclawGetLocalBridgeStatus(agentId?: string): Promise<OpenClawBridgeStatus> {
  return invoke<OpenClawBridgeStatus>('openclaw_get_local_bridge_status', { agentId: agentId ?? null });
}

export async function openclawGetLocalHealth(): Promise<LocalMorganHealth> {
  return invoke<LocalMorganHealth>('openclaw_get_local_health');
}

export async function openclawGetMorganDiagnostics(agentId?: string): Promise<MorganDiagnostics> {
  return invoke<MorganDiagnostics>('openclaw_get_morgan_diagnostics', { agentId: agentId ?? null });
}

/** Execute a CLI command through the OpenClaw proxy */
export async function openclawExecCli(
  cli: string,
  args: string[]
): Promise<string> {
  return invoke<string>('openclaw_exec_cli', { cli, args });
}

// ============================================================================
// Studio Commands
// ============================================================================

export async function studioGetState(): Promise<StudioState> {
  return invoke<StudioState>('studio_get_state');
}

export async function studioSaveState(state: StudioState): Promise<StudioState> {
  return invoke<StudioState>('studio_save_state', { state });
}

export async function studioRenderAgentConfig(
  agentId: string,
  projectId?: string | null
): Promise<RenderedAgentConfig> {
  return invoke<RenderedAgentConfig>('studio_render_agent_config', {
    agentId,
    projectId: projectId ?? null,
  });
}

export async function studioExportAgentConfig(
  agentId: string,
  projectId?: string | null
): Promise<RenderedAgentConfig> {
  return invoke<RenderedAgentConfig>('studio_export_agent_config', {
    agentId,
    projectId: projectId ?? null,
  });
}

export async function studioApplyAgentConfig(
  agentId: string,
  projectId?: string | null
): Promise<ApplyAgentConfigResult> {
  return invoke<ApplyAgentConfigResult>('studio_apply_agent_config', {
    agentId,
    projectId: projectId ?? null,
  });
}
