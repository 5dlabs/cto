/**
 * React hooks for Tauri commands with loading states and error handling
 */

import { useState, useEffect, useCallback } from 'react';
import * as tauri from '@/lib/tauri';

// ============================================================================
// Generic async hook
// ============================================================================

interface AsyncState<T> {
  data: T | null;
  loading: boolean;
  error: Error | null;
}

function useAsync<T>(
  asyncFn: () => Promise<T>,
  deps: unknown[] = []
): AsyncState<T> & { refetch: () => Promise<void> } {
  const [state, setState] = useState<AsyncState<T>>({
    data: null,
    loading: true,
    error: null,
  });

  const execute = useCallback(async () => {
    setState((s) => ({ ...s, loading: true, error: null }));
    try {
      const data = await asyncFn();
      setState({ data, loading: false, error: null });
    } catch (error) {
      setState({ data: null, loading: false, error: error as Error });
    }
  }, deps);

  useEffect(() => {
    execute();
  }, [execute]);

  return { ...state, refetch: execute };
}

// ============================================================================
// Setup Hooks
// ============================================================================

/** Hook to check Docker status */
export function useDockerStatus() {
  return useAsync(() => tauri.checkDocker(), []);
}

/** Hook to check Kind status */
export function useKindStatus() {
  return useAsync(() => tauri.checkKind(), []);
}

/** Hook for setup wizard state */
export function useSetupState() {
  const state = useAsync(() => tauri.getSetupState(), []);
  
  const updateState = useCallback(async (newState: tauri.SetupState) => {
    await tauri.saveSetupState(newState);
    state.refetch();
  }, [state.refetch]);

  const complete = useCallback(async () => {
    await tauri.completeSetup();
    state.refetch();
  }, [state.refetch]);

  return {
    ...state,
    updateState,
    complete,
  };
}

// ============================================================================
// API Key Hooks
// ============================================================================

/** Hook to manage API keys */
export function useApiKeys() {
  const [keys, setKeys] = useState<tauri.ApiKeysConfigured>({
    anthropic: false,
    openai: false,
    github: false,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const checkKeys = useCallback(async () => {
    setLoading(true);
    try {
      const [anthropic, openai, github] = await Promise.all([
        tauri.hasApiKey('anthropic'),
        tauri.hasApiKey('openai'),
        tauri.hasApiKey('github'),
      ]);
      setKeys({ anthropic, openai, github });
      setError(null);
    } catch (e) {
      setError(e as Error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    checkKeys();
  }, [checkKeys]);

  const storeKey = useCallback(async (keyType: tauri.ApiKeyType, value: string) => {
    await tauri.storeApiKey(keyType, value);
    await checkKeys();
  }, [checkKeys]);

  const deleteKey = useCallback(async (keyType: tauri.ApiKeyType) => {
    await tauri.deleteApiKey(keyType);
    await checkKeys();
  }, [checkKeys]);

  return {
    keys,
    loading,
    error,
    storeKey,
    deleteKey,
    refetch: checkKeys,
  };
}

// ============================================================================
// Cluster Hooks
// ============================================================================

/** Hook to manage Kind cluster */
export function useCluster() {
  const status = useAsync(() => tauri.getClusterStatus(), []);
  const [creating, setCreating] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const create = useCallback(async () => {
    setCreating(true);
    try {
      await tauri.createCluster();
      await status.refetch();
    } finally {
      setCreating(false);
    }
  }, [status.refetch]);

  const remove = useCallback(async () => {
    setDeleting(true);
    try {
      await tauri.deleteCluster();
      await status.refetch();
    } finally {
      setDeleting(false);
    }
  }, [status.refetch]);

  return {
    ...status,
    creating,
    deleting,
    create,
    remove,
  };
}

// ============================================================================
// Workflow Hooks
// ============================================================================

/** Hook to list workflows */
export function useWorkflows() {
  return useAsync(() => tauri.listWorkflows(), []);
}

/** Hook to get a single workflow's status */
export function useWorkflowStatus(workflowId: string | null) {
  return useAsync(
    async () => {
      if (!workflowId) return null;
      return tauri.getWorkflowStatus(workflowId);
    },
    [workflowId]
  );
}

/** Hook to get workflow logs */
export function useWorkflowLogs(workflowId: string | null, nodeName?: string) {
  return useAsync(
    async () => {
      if (!workflowId) return '';
      return tauri.getWorkflowLogs(workflowId, nodeName);
    },
    [workflowId, nodeName]
  );
}

/** Hook to trigger workflows */
export function useTriggerWorkflow() {
  const [triggering, setTriggering] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [lastWorkflowId, setLastWorkflowId] = useState<string | null>(null);

  const trigger = useCallback(async (
    repoUrl: string,
    prompt: string,
    branch?: string,
    stack?: 'grizz' | 'nova'
  ) => {
    setTriggering(true);
    setError(null);
    try {
      const workflowId = await tauri.triggerWorkflow(repoUrl, prompt, branch, stack);
      setLastWorkflowId(workflowId);
      return workflowId;
    } catch (e) {
      setError(e as Error);
      throw e;
    } finally {
      setTriggering(false);
    }
  }, []);

  return {
    trigger,
    triggering,
    error,
    lastWorkflowId,
  };
}

// ============================================================================
// Combined System Check Hook
// ============================================================================

/** Hook to check all system requirements at once */
export function useSystemCheck() {
  const docker = useDockerStatus();
  const kind = useKindStatus();

  const allReady = 
    docker.data?.installed && 
    docker.data?.running && 
    kind.data?.installed;

  const loading = docker.loading || kind.loading;
  const error = docker.error || kind.error;

  return {
    docker: docker.data,
    kind: kind.data,
    allReady,
    loading,
    error,
    refetch: () => {
      docker.refetch();
      kind.refetch();
    },
  };
}
