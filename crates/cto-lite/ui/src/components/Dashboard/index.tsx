import React, { useState, useEffect, useCallback } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
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
  Terminal
} from "lucide-react";
import * as tauri from "@/lib/tauri";

interface WorkflowListItem extends tauri.WorkflowStatus {}

export function Dashboard() {
  const [workflows, setWorkflows] = useState<WorkflowListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedWorkflow, setSelectedWorkflow] = useState<string | null>(null);
  const [workflowDetail, setWorkflowDetail] = useState<tauri.WorkflowDetail | null>(null);
  const [logs, setLogs] = useState<string>("");
  const [showNewWorkflow, setShowNewWorkflow] = useState(false);
  
  // New workflow form
  const [repoUrl, setRepoUrl] = useState("");
  const [branch, setBranch] = useState("main");
  const [prompt, setPrompt] = useState("");
  const [submitting, setSubmitting] = useState(false);

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

  const loadLogs = useCallback(async (workflowName: string, nodeName?: string) => {
    try {
      const logContent = await tauri.getWorkflowLogs(workflowName, nodeName);
      setLogs(logContent);
    } catch (e) {
      console.error("Failed to load logs:", e);
      setLogs("Failed to load logs");
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
      loadLogs(selectedWorkflow);
      // Poll selected workflow every 5 seconds
      const interval = setInterval(() => {
        loadWorkflowDetail(selectedWorkflow);
        loadLogs(selectedWorkflow);
      }, 5000);
      return () => clearInterval(interval);
    }
  }, [selectedWorkflow, loadWorkflowDetail, loadLogs]);

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
        setLogs("");
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

            {/* Logs */}
            <div className="flex-1 flex flex-col min-h-0">
              <h3 className="text-sm font-medium mb-2 flex items-center gap-2">
                <Terminal className="h-4 w-4" />
                Logs
              </h3>
              <ScrollArea className="flex-1 bg-black rounded-lg p-3">
                <pre className="text-xs text-green-400 font-mono whitespace-pre-wrap">
                  {logs || "No logs available"}
                </pre>
              </ScrollArea>
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
