import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { RefreshCw, Download, CheckCircle, AlertCircle } from "lucide-react";

interface ImageVersion {
  image: string;
  current: string | null;
  latest: string | null;
  has_update: boolean;
}

interface UpdateStatus {
  images: ImageVersion[];
  has_updates: boolean;
  last_checked: string | null;
}

interface PullResult {
  image: string;
  success: boolean;
  message: string;
}

export function Updates() {
  const [status, setStatus] = useState<UpdateStatus | null>(null);
  const [checking, setChecking] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [pullResults, setPullResults] = useState<PullResult[]>([]);

  const checkForUpdates = async () => {
    setChecking(true);
    try {
      const result = await invoke<UpdateStatus>("check_updates");
      setStatus(result);
    } catch (err) {
      console.error("Failed to check updates:", err);
    } finally {
      setChecking(false);
    }
  };

  const pullUpdates = async () => {
    setUpdating(true);
    setPullResults([]);
    try {
      // Pull the images
      const results = await invoke<PullResult[]>("pull_updates");
      setPullResults(results);
      
      // Apply to cluster
      await invoke<string>("apply_updates");
      
      // Refresh status
      await checkForUpdates();
    } catch (err) {
      console.error("Failed to pull updates:", err);
    } finally {
      setUpdating(false);
    }
  };

  const formatImageName = (image: string) => {
    return image.replace("ghcr.io/5dlabs/", "");
  };

  const formatTime = (iso: string) => {
    return new Date(iso).toLocaleString();
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Updates</CardTitle>
            <CardDescription>
              Keep CTO App components up to date
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={checkForUpdates}
              disabled={checking}
            >
              <RefreshCw className={`h-4 w-4 mr-2 ${checking ? "animate-spin" : ""}`} />
              Check
            </Button>
            {status?.has_updates && (
              <Button
                size="sm"
                onClick={pullUpdates}
                disabled={updating}
              >
                <Download className={`h-4 w-4 mr-2 ${updating ? "animate-bounce" : ""}`} />
                Update All
              </Button>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {status?.last_checked && (
          <p className="text-sm text-muted-foreground mb-4">
            Last checked: {formatTime(status.last_checked)}
          </p>
        )}

        {!status && !checking && (
          <p className="text-muted-foreground text-center py-8">
            Click "Check" to check for updates
          </p>
        )}

        {checking && (
          <p className="text-muted-foreground text-center py-8">
            Checking for updates...
          </p>
        )}

        {status && (
          <div className="space-y-2">
            {status.images.map((img) => (
              <div
                key={img.image}
                className="flex items-center justify-between p-3 rounded-lg bg-muted/50"
              >
                <div>
                  <p className="font-medium">{formatImageName(img.image)}</p>
                  <p className="text-xs text-muted-foreground">
                    {img.current ? `Current: ${img.current.slice(0, 12)}...` : "Not installed"}
                  </p>
                </div>
                <div>
                  {img.has_update ? (
                    <Badge variant="default" className="bg-blue-500">
                      Update available
                    </Badge>
                  ) : img.current ? (
                    <Badge variant="secondary">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      Up to date
                    </Badge>
                  ) : (
                    <Badge variant="outline">
                      Not installed
                    </Badge>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}

        {pullResults.length > 0 && (
          <div className="mt-4 space-y-1">
            <p className="text-sm font-medium">Update Results:</p>
            {pullResults.map((result) => (
              <div
                key={result.image}
                className="flex items-center gap-2 text-sm"
              >
                {result.success ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-red-500" />
                )}
                <span>{formatImageName(result.image)}</span>
                {!result.success && (
                  <span className="text-red-500 text-xs">{result.message}</span>
                )}
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
