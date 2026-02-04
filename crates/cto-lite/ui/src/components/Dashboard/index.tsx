import React, { useState, useEffect, useCallback, useRef } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Play,
  Square,
  Trash2,
  RefreshCw,
  Loader2,
  CheckCircle,
  XCircle,
  Clock,
  GitBranch,
  Terminal,
  Eye,
  EyeOff
} from "lucide-react";
import * as tauri from "@/lib/tauri";

interface WorkflowListItem extends tauri.WorkflowStatus {}

interface LogLine {
  timestamp: string;
  pod: string;
  container: string;
  message: string;
}

export function Dashboard() {
  const [workflows, setWorkflows] = useState<WorkflowListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedWorkflow, setSelectedWorkflow] = useState<string | null>(null);
  const [workflowDetail, setWorkflowDetail] = useState<tauri.WorkflowDetail | null>(null);
  const [showNewWorkflow, setShowNewWorkflow] = useState(false);

  // Log streaming state
  const [isStreaming, setIsStreaming] = useState(false);
  const [followEnabled, setFollowEnabled] = useState(true);
  const [selectedPod, setSelectedPod] = useState<string>("");
  const [selectedNamespace, setSelectedNamespace] = useState<string>("default");
  const [availablePods, setAvailablePods] = useState<tauri.PodInfo[]>([]);
  const [namespaces, setNamespaces] = useState<string[]>([]);
  const [logLines, setLogLines] = useState<LogLine[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);
  const logIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // New workflow form
  const [repoUrl, setRepoUrl] = useState("");
  const [branch, setBranch] = useState("main");
  const [prompt, setPrompt] = useState("");
  const [submitting, setSubmitting] = useState(false);

  // Load namespaces on mount
  useEffect(() => {
    loadNamespaces();
  }, []);

  // Load pods when namespace changes
  useEffect(() => {
    if (selectedNamespace) {
      loadPods(selectedNamespace);
    }
  }, [selectedNamespace]);

  // Auto-scroll when follow is enabled
  useEffect(() => {
    if (followEnabled && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logLines, followEnabled]);

  const loadNamespaces = async () => {
    try {
      const ns = await tauri.listNamespaces();
      setNamespaces(ns.length > 0 ? ns : ["default", "cto", "argo"]);
    } catch (e) {
      console.error("Failed to load namespaces:", e);
      setNamespaces(["default", "cto", "argo"]);
    }
  };

  const loadPods = async (namespace: string) => {
    try {
      const pods = await tauri.listPodsWithStatus(namespace);
      setAvailablePods(pods);
      if (pods.length > 0 && !selectedPod) {
        setSelectedPod(pods[0].name);
      }
    } catch (e) {
      console.error("Failed to load pods:", e);
      setAvailablePods([]);
    }
  };

  const startLogStreaming = useCallback(async () => {
    if (!selectedPod) return;

    setIsStreaming(true);
    setLogLines([]);

    // Poll for logs every 2 seconds
    logIntervalRef.current = setInterval(async () => {
      try {
        const logEntries = await tauri.streamPodLogs(selectedPod, selectedNamespace);
        if (logEntries.length > 0) {
          const newLines: LogLine[] = logEntries.map(entry => ({
            timestamp: entry.timestamp,
            pod: entry.pod,
            container: entry.container,
            message: entry.message
          }));
          setLogLines(prev => {
            // Keep last 1000 lines to prevent memory issues
            const combined = [...prev, ...newLines];
            if (combined.length > 1000) {
              return combined.slice(-1000);
            }
            return combined;
          });
        }
      } catch (e) {
        console.error("Failed to fetch logs:", e);
      }
    }, 2000);
  }, [selectedPod, selectedNamespace]);

  const stopLogStreaming = useCallback(() => {
    if (logIntervalRef.current) {
      clearInterval(logIntervalRef.current);
      logIntervalRef.current = null;
    }
    setIsStreaming(false);
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stopLogStreaming();
    };
  }, [stopLogStreaming]);

  const refreshWorkflows = useCallback(async () => {
    setLoading(true);
    try {
      const list = await tauri.listWorkflows();
      setWorkflows(list);
    } catch (e) {
      console.error("Failed to list workflows:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadWorkflowDetail = useCallback(async (name: string) => {
    try {
      const detail = await tauri.getWorkflowStatus(name);
      setWorkflowDetail(detail);
    } catch (e) {
      console.error("Failed to load workflow detail:", e);
    }
  }, []);

  useEffect(() => {
    refreshWorkflows();
    // Poll for updates every 10 seconds
    const interval = setInterval(refreshWorkflows, 10000);
    return () => clearInterval(interval);
  }, [refreshWorkflows]);

  useEffect(() => {
    if (selectedWorkflow) {
      loadWorkflowDetail(selectedWorkflow);
      // Poll selected workflow every 5 seconds
      const interval = setInterval(() => {
        loadWorkflowDetail(selectedWorkflow);
      }, 5000);
      return () => clearInterval(interval);
    }
  }, [selectedWorkflow, loadWorkflowDetail]);

  const handleTrigger = async () => {
    if (!repoUrl || !prompt) return;

    setSubmitting(true);
    try {
      const name = await tauri.triggerWorkflow(repoUrl, prompt, branch || undefined);
      setShowNewWorkflow(false);
      setRepoUrl("");
      setBranch("main");
      setPrompt("");
      setSelectedWorkflow(name);
      await refreshWorkflows();
    } catch (e) {
      console.error("Failed to trigger workflow:", e);
    } finally {
      setSubmitting(false);
    }
  };

  const handleStop = async (name: string) => {
    try {
      await tauri.stopWorkflow(name);
      await refreshWorkflows();
    } catch (e) {
      console.error("Failed to stop workflow:", e);
    }
  };

  const handleDelete = async (name: string) => {
    try {
      await tauri.deleteWorkflow(name);
      if (selectedWorkflow === name) {
        setSelectedWorkflow(null);
        setWorkflowDetail(null);
        // Stop log streaming if active
        if (isStreaming) {
          stopLogStreaming();
        }
        setLogLines([]);
      }
      await refreshWorkflows();
    } catch (e) {
      console.error("Failed to delete workflow:", e);
    }
  };

  const getPhaseIcon = (phase: string) => {
    switch (phase.toLowerCase()) {
      case "succeeded":
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case "failed":
      case "error":
        return <XCircle className="h-4 w-4 text-red-500" />;
      case "running":
        return <Loader2 className="h-4 w-4 animate-spin text-blue-500" />;
      case "pending":
        return <Clock className="h-4 w-4 text-yellow-500" />;
      default:
        return <Clock className="h-4 w-4 text-muted-foreground" />;
    }
  };

  const getPhaseBadge = (phase: string) => {
    const variant = {
      succeeded: "default" as const,
      failed: "destructive" as const,
      error: "destructive" as const,
      running: "secondary" as const,
      pending: "outline" as const,
    }[phase.toLowerCase()] || "outline" as const;

    return <Badge variant={variant}>{phase}</Badge>;
  };

  const formatTimestamp = (ts: string) => {
    if (!ts) return "";
    try {
      // Parse ISO timestamp
      const date = new Date(ts);
      return date.toLocaleTimeString();
    } catch {
      return ts.split("T")[1]?.split(".")[0] || ts;
    }
  };

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar - Workflow List */}
      <div className="w-80 border-r flex flex-col">
        <div className="p-4 border-b">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold">Workflows</h2>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="icon"
                onClick={refreshWorkflows}
                disabled={loading}
              >
                <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
              </Button>
              <Button
                size="icon"
                onClick={() => setShowNewWorkflow(true)}
              >
                <Play className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>

        <ScrollArea className="flex-1">
          {workflows.length === 0 && !loading && (
            <div className="p-4 text-center text-muted-foreground">
              No workflows yet. Click + to create one.
            </div>
          )}
          {workflows.map((workflow) => (
            <div
              key={workflow.name}
              className={`p-3 border-b cursor-pointer hover:bg-muted/50 transition-colors ${
                selectedWorkflow === workflow.name ? 'bg-muted' : ''
              }`}
              onClick={() => setSelectedWorkflow(workflow.name)}
            >
              <div className="flex items-center justify-between mb-1">
                <div className="flex items-center gap-2">
                  {getPhaseIcon(workflow.phase)}
                  <span className="font-medium text-sm truncate max-w-[150px]">
                    {workflow.name}
                  </span>
                </div>
                {getPhaseBadge(workflow.phase)}
              </div>
              {workflow.startedAt && (
                <div className="text-xs text-muted-foreground">
                  {new Date(workflow.startedAt).toLocaleString()}
                </div>
              )}
              {workflow.progress && (
                <div className="text-xs text-muted-foreground mt-1">
                  Progress: {workflow.progress}
                </div>
              )}
            </div>
          ))}
        </ScrollArea>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {showNewWorkflow ? (
          <div className="p-6">
            <Card>
              <CardHeader>
                <CardTitle>New Workflow</CardTitle>
                <CardDescription>
                  Trigger a new AI development workflow
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="repo">Repository URL</Label>
                  <Input
                    id="repo"
                    placeholder="https://github.com/owner/repo"
                    value={repoUrl}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setRepoUrl(e.target.value)}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="branch">Branch</Label>
                  <Input
                    id="branch"
                    placeholder="main"
                    value={branch}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setBranch(e.target.value)}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="prompt">Development Request</Label>
                  <Textarea
                    id="prompt"
                    placeholder="Describe what you want to build..."
                    rows={4}
                    value={prompt}
                    onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setPrompt(e.target.value)}
                  />
                </div>
                <div className="flex gap-2">
                  <Button
                    onClick={handleTrigger}
                    disabled={!repoUrl || !prompt || submitting}
                  >
                    {submitting ? (
                      <>
                        <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                        Starting...
                      </>
                    ) : (
                      <>
                        <Play className="h-4 w-4 mr-2" />
                        Start Workflow
                      </>
                    )}
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => setShowNewWorkflow(false)}
                  >
                    Cancel
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        ) : selectedWorkflow && workflowDetail ? (
          <div className="flex-1 flex flex-col overflow-hidden p-4">
            {/* Workflow Header */}
            <div className="flex items-center justify-between mb-4">
              <div>
                <div className="flex items-center gap-2">
                  {getPhaseIcon(workflowDetail.status.phase)}
                  <h2 className="text-xl font-semibold">{workflowDetail.status.name}</h2>
                  {getPhaseBadge(workflowDetail.status.phase)}
                </div>
                {workflowDetail.status.message && (
                  <p className="text-sm text-muted-foreground mt-1">
                    {workflowDetail.status.message}
                  </p>
                )}
              </div>
              <div className="flex gap-2">
                {workflowDetail.status.phase === "Running" && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleStop(workflowDetail.status.name)}
                  >
                    <Square className="h-4 w-4 mr-1" />
                    Stop
                  </Button>
                )}
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleDelete(workflowDetail.status.name)}
                >
                  <Trash2 className="h-4 w-4 mr-1" />
                  Delete
                </Button>
              </div>
            </div>

            {/* Workflow Nodes */}
            <div className="mb-4">
              <h3 className="text-sm font-medium mb-2 flex items-center gap-2">
                <GitBranch className="h-4 w-4" />
                Steps
              </h3>
              <div className="space-y-1">
                {workflowDetail.nodes.map((node) => (
                  <div
                    key={node.id}
                    className="flex items-center gap-2 p-2 rounded bg-muted/50 text-sm"
                  >
                    {getPhaseIcon(node.phase)}
                    <span className="flex-1">{node.displayName || node.name}</span>
                    <span className="text-xs text-muted-foreground">
                      {node.nodeType}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {/* Real-time Log Viewer */}
            <div className="flex-1 flex flex-col min-h-0 border rounded-lg overflow-hidden">
              <div className="bg-muted px-3 py-2 border-b flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <h3 className="text-sm font-medium flex items-center gap-2">
                    <Terminal className="h-4 w-4" />
                    Live Logs
                  </h3>
                  {isStreaming && (
                    <Badge variant="secondary" className="text-xs">
                      <span className="w-2 h-2 rounded-full bg-green-500 mr-1 animate-pulse" />
                      Streaming
                    </Badge>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  {/* Namespace filter */}
                  <Select value={selectedNamespace} onValueChange={setSelectedNamespace}>
                    <SelectTrigger className="w-32 h-8 text-xs">
                      <SelectValue placeholder="Namespace" />
                    </SelectTrigger>
                    <SelectContent>
                      {namespaces.map((ns) => (
                        <SelectItem key={ns} value={ns}>{ns}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  {/* Pod filter */}
                  <Select value={selectedPod} onValueChange={setSelectedPod}>
                    <SelectTrigger className="w-48 h-8 text-xs">
                      <SelectValue placeholder="Select pod" />
                    </SelectTrigger>
                    <SelectContent>
                      {availablePods.map((pod) => (
                        <SelectItem key={pod.name} value={pod.name}>
                          {pod.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  {/* Follow toggle */}
                  <Button
                    variant={followEnabled ? "default" : "outline"}
                    size="icon"
                    className="h-8 w-8"
                    onClick={() => setFollowEnabled(!followEnabled)}
                    title={followEnabled ? "Disable auto-scroll" : "Enable auto-scroll"}
                  >
                    {followEnabled ? (
                      <Eye className="h-4 w-4" />
                    ) : (
                      <EyeOff className="h-4 w-4" />
                    )}
                  </Button>

                  {/* Stream controls */}
                  {!isStreaming ? (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={startLogStreaming}
                      disabled={!selectedPod}
                    >
                      <Play className="h-3 w-3 mr-1" />
                      Start
                    </Button>
                  ) : (
                    <Button
                      size="sm"
                      variant="destructive"
                      onClick={stopLogStreaming}
                    >
                      <Square className="h-3 w-3 mr-1" />
                      Stop
                    </Button>
                  )}
                </div>
              </div>

              {/* Log content */}
              <ScrollArea ref={scrollRef} className="flex-1 bg-black p-3">
                {logLines.length === 0 ? (
                  <div className="text-muted-foreground text-sm text-center py-8">
                    {isStreaming
                      ? "Waiting for log entries..."
                      : "Click 'Start' to begin streaming logs"}
                  </div>
                ) : (
                  <div className="font-mono text-xs space-y-1">
                    {logLines.map((line, idx) => (
                      <div key={idx} className="flex gap-2 hover:bg-muted/30 px-1 rounded">
                        <span className="text-muted-foreground shrink-0">
                          {formatTimestamp(line.timestamp)}
                        </span>
                        <span className="text-blue-400 shrink-0">
                          [{line.container}]
                        </span>
                        <span className="text-green-400 whitespace-pre-wrap break-all">
                          {line.message}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </ScrollArea>

              {/* Status bar */}
              <div className="bg-muted px-3 py-1 border-t text-xs text-muted-foreground flex items-center justify-between">
                <span>{logLines.length} lines</span>
                <span>Following: {followEnabled ? "On" : "Off"}</span>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex-1 flex items-center justify-center text-muted-foreground">
            <div className="text-center">
              <Terminal className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>Select a workflow to view details</p>
              <p className="text-sm">or click + to create a new one</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
