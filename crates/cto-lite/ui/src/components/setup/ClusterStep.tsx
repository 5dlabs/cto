import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Label } from "@/components/ui/label";
import { Loader2, CheckCircle, AlertCircle, Server, Plus } from "lucide-react";

interface DetectedCluster {
  name: string;
  context: string;
  cluster_type: string;
  server: string | null;
  is_running: boolean;
  is_current: boolean;
}

interface ClusterDetectionResult {
  clusters: DetectedCluster[];
  has_existing: boolean;
  recommendation: {
    UseExisting?: { context: string; reason: string };
    CreateKind?: { reason: string };
  };
}

interface ClusterStepProps {
  onComplete: (clusterContext: string) => void;
  onBack: () => void;
}

const clusterTypeLabels: Record<string, string> = {
  Kind: "Kind",
  DockerDesktop: "Docker Desktop",
  RancherDesktop: "Rancher Desktop",
  Minikube: "Minikube",
  K3d: "K3d",
  OrbStack: "OrbStack",
  Other: "Other",
};

export function ClusterStep({ onComplete, onBack }: ClusterStepProps) {
  const [detecting, setDetecting] = useState(true);
  const [detection, setDetection] = useState<ClusterDetectionResult | null>(null);
  const [selected, setSelected] = useState<string>("create-new");
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    detectClusters();
  }, []);

  const detectClusters = async () => {
    setDetecting(true);
    setError(null);
    try {
      const result = await invoke<ClusterDetectionResult>("detect_existing_clusters");
      setDetection(result);
      
      // Pre-select based on recommendation
      if (result.recommendation.UseExisting) {
        setSelected(result.recommendation.UseExisting.context);
      } else {
        setSelected("create-new");
      }
    } catch (err) {
      console.error("Failed to detect clusters:", err);
      setError("Failed to detect existing clusters");
    } finally {
      setDetecting(false);
    }
  };

  const handleContinue = async () => {
    setCreating(true);
    setError(null);
    
    try {
      if (selected === "create-new") {
        // Create a new Kind cluster
        await invoke("create_cluster");
        onComplete("kind-cto-lite");
      } else {
        // Use an existing cluster
        await invoke("use_existing_cluster", { context: selected });
        onComplete(selected);
      }
    } catch (err: any) {
      console.error("Failed to setup cluster:", err);
      setError(err.message || "Failed to setup cluster");
      setCreating(false);
    }
  };

  const runningClusters = detection?.clusters.filter(c => c.is_running) || [];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Kubernetes Cluster</h2>
        <p className="text-muted-foreground mt-2">
          CTO Lite runs on a local Kubernetes cluster. Choose an existing cluster or create a new one.
        </p>
      </div>

      {detecting ? (
        <Card>
          <CardContent className="py-12 text-center">
            <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-muted-foreground" />
            <p className="text-muted-foreground">Detecting existing Kubernetes clusters...</p>
          </CardContent>
        </Card>
      ) : (
        <>
          {detection?.recommendation && (
            <Card className="border-blue-200 bg-blue-50/50 dark:border-blue-800 dark:bg-blue-950/20">
              <CardContent className="pt-4">
                <p className="text-sm">
                  <strong>Recommendation:</strong>{" "}
                  {detection.recommendation.UseExisting?.reason || 
                   detection.recommendation.CreateKind?.reason}
                </p>
              </CardContent>
            </Card>
          )}

          <RadioGroup value={selected} onValueChange={setSelected}>
            {/* Create new option */}
            <Card className={`cursor-pointer transition-colors ${selected === "create-new" ? "border-primary" : ""}`}>
              <CardHeader className="pb-3">
                <div className="flex items-start gap-3">
                  <RadioGroupItem value="create-new" id="create-new" className="mt-1" />
                  <div className="flex-1">
                    <Label htmlFor="create-new" className="cursor-pointer">
                      <CardTitle className="text-base flex items-center gap-2">
                        <Plus className="h-4 w-4" />
                        Create new Kind cluster
                      </CardTitle>
                      <CardDescription className="mt-1">
                        Creates a dedicated "cto-lite" cluster using Kind (Kubernetes in Docker).
                        Recommended for isolation.
                      </CardDescription>
                    </Label>
                  </div>
                </div>
              </CardHeader>
            </Card>

            {/* Existing clusters */}
            {runningClusters.map((cluster) => (
              <Card 
                key={cluster.context}
                className={`cursor-pointer transition-colors ${selected === cluster.context ? "border-primary" : ""}`}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start gap-3">
                    <RadioGroupItem value={cluster.context} id={cluster.context} className="mt-1" />
                    <div className="flex-1">
                      <Label htmlFor={cluster.context} className="cursor-pointer">
                        <CardTitle className="text-base flex items-center gap-2">
                          <Server className="h-4 w-4" />
                          {cluster.context}
                          <Badge variant="secondary" className="ml-2">
                            {clusterTypeLabels[cluster.cluster_type] || cluster.cluster_type}
                          </Badge>
                          {cluster.is_current && (
                            <Badge variant="outline" className="ml-1">Current</Badge>
                          )}
                        </CardTitle>
                        <CardDescription className="mt-1">
                          {cluster.server || "Local cluster"}
                        </CardDescription>
                      </Label>
                    </div>
                    <CheckCircle className="h-5 w-5 text-green-500" />
                  </div>
                </CardHeader>
              </Card>
            ))}

            {/* Non-running clusters (collapsed) */}
            {detection?.clusters.filter(c => !c.is_running).length ? (
              <details className="text-sm text-muted-foreground">
                <summary className="cursor-pointer hover:text-foreground">
                  {detection.clusters.filter(c => !c.is_running).length} inactive cluster(s) found
                </summary>
                <div className="mt-2 space-y-1 pl-4">
                  {detection.clusters.filter(c => !c.is_running).map((cluster) => (
                    <div key={cluster.context} className="flex items-center gap-2">
                      <AlertCircle className="h-4 w-4 text-yellow-500" />
                      <span>{cluster.context}</span>
                      <span className="text-xs">({clusterTypeLabels[cluster.cluster_type] || cluster.cluster_type})</span>
                    </div>
                  ))}
                </div>
              </details>
            ) : null}
          </RadioGroup>

          {error && (
            <Card className="border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20">
              <CardContent className="pt-4">
                <p className="text-sm text-red-600 dark:text-red-400 flex items-center gap-2">
                  <AlertCircle className="h-4 w-4" />
                  {error}
                </p>
              </CardContent>
            </Card>
          )}
        </>
      )}

      <div className="flex justify-between pt-4">
        <Button variant="outline" onClick={onBack} disabled={creating}>
          Back
        </Button>
        <Button onClick={handleContinue} disabled={detecting || creating}>
          {creating ? (
            <>
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              {selected === "create-new" ? "Creating cluster..." : "Configuring..."}
            </>
          ) : (
            "Continue"
          )}
        </Button>
      </div>
    </div>
  );
}
