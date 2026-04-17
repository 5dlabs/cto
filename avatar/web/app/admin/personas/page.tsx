/**
 * Admin Personas Page
 *
 * Drag-drop upload for persona source files (image/video).
 * List view with status badges, thumbnails, delete action.
 * 2-second polling while any persona is in "preprocessing" state.
 */

"use client";

import { useCallback, useEffect, useRef, useState } from "react";

interface Persona {
  id: string;
  name: string;
  state: "pending" | "preprocessing" | "ready" | "failed";
  error?: string;
  progress_percent: number;
  created_at?: string;
}

interface UploadState {
  isUploading: boolean;
  progress: number;
  error?: string;
}

const STATUS_COLORS: Record<string, string> = {
  pending: "bg-yellow-500/20 text-yellow-300 border-yellow-500/30",
  preprocessing: "bg-blue-500/20 text-blue-300 border-blue-500/30",
  ready: "bg-emerald-500/20 text-emerald-300 border-emerald-500/30",
  failed: "bg-red-500/20 text-red-300 border-red-500/30",
};

function StatusBadge({ state }: { state: string }) {
  return (
    <span
      className={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium ${
        STATUS_COLORS[state] || STATUS_COLORS.pending
      }`}
    >
      {state}
    </span>
  );
}

function ProgressBar({ progress }: { progress: number }) {
  return (
    <div className="h-2 w-full rounded-full bg-slate-700">
      <div
        className="h-2 rounded-full bg-blue-500 transition-all duration-300"
        style={{ width: `${progress}%` }}
      />
    </div>
  );
}

export default function AdminPersonasPage() {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uploadState, setUploadState] = useState<UploadState>({
    isUploading: false,
    progress: 0,
  });
  const [newPersonaName, setNewPersonaName] = useState("");
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const pollingRef = useRef<NodeJS.Timeout | null>(null);

  // Fetch personas list
  const fetchPersonas = useCallback(async () => {
    try {
      const response = await fetch("/api/admin/personas");
      if (!response.ok) {
        throw new Error(`Failed to fetch: ${response.status}`);
      }
      const data = await response.json();
      setPersonas(data.personas || []);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load personas");
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    fetchPersonas();
  }, [fetchPersonas]);

  // Polling while preprocessing
  useEffect(() => {
    const hasPreprocessing = personas.some((p) => p.state === "preprocessing");

    if (hasPreprocessing && !pollingRef.current) {
      pollingRef.current = setInterval(() => {
        fetchPersonas();
      }, 2000);
    } else if (!hasPreprocessing && pollingRef.current) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }

    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, [personas, fetchPersonas]);

  // File selection handlers
  const handleFileSelect = (file: File | null) => {
    if (!file) return;

    // Validate file type
    const allowedTypes = [
      "image/png",
      "image/jpeg",
      "image/webp",
      "video/mp4",
      "video/webm",
    ];
    if (!allowedTypes.includes(file.type)) {
      setUploadState({
        isUploading: false,
        progress: 0,
        error: `Invalid file type: ${file.type}. Allowed: PNG, JPG, WebP, MP4, WebM`,
      });
      return;
    }

    // Validate file size (50MB)
    const maxSize = 50 * 1024 * 1024;
    if (file.size > maxSize) {
      setUploadState({
        isUploading: false,
        progress: 0,
        error: `File too large: ${(file.size / 1024 / 1024).toFixed(1)}MB. Max: 50MB`,
      });
      return;
    }

    setSelectedFile(file);
    setUploadState({ isUploading: false, progress: 0 });
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const file = e.dataTransfer.files[0];
    handleFileSelect(file);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  // Upload handler
  const handleUpload = async () => {
    if (!selectedFile || !newPersonaName.trim()) return;

    setUploadState({ isUploading: true, progress: 0 });

    const formData = new FormData();
    formData.append("name", newPersonaName.trim());
    formData.append("file", selectedFile);

    try {
      const response = await fetch("/api/admin/personas", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || `Upload failed: ${response.status}`);
      }

      const result = await response.json();

      // Reset form
      setNewPersonaName("");
      setSelectedFile(null);
      setUploadState({ isUploading: false, progress: 100 });

      // Refresh list
      await fetchPersonas();
    } catch (err) {
      setUploadState({
        isUploading: false,
        progress: 0,
        error: err instanceof Error ? err.message : "Upload failed",
      });
    }
  };

  // Delete handler
  const handleDelete = async (id: string) => {
    if (!confirm(`Delete persona "${id}"? This cannot be undone.`)) return;

    try {
      const response = await fetch(`/api/admin/personas/${id}`, {
        method: "DELETE",
      });

      if (!response.ok) {
        throw new Error(`Delete failed: ${response.status}`);
      }

      await fetchPersonas();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Delete failed");
    }
  };

  // Format date
  const formatDate = (dateStr?: string) => {
    if (!dateStr) return "Unknown";
    try {
      return new Date(dateStr).toLocaleString();
    } catch {
      return dateStr;
    }
  };

  return (
    <main className="min-h-screen bg-slate-950 p-6 text-slate-100">
      <div className="mx-auto max-w-6xl">
        <header className="mb-8">
          <h1 className="text-3xl font-bold">Persona Admin</h1>
          <p className="mt-2 text-slate-400">
            Manage avatar personas for MuseTalk lip-sync generation
          </p>
        </header>

        {/* Upload Section */}
        <section className="mb-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
          <h2 className="mb-4 text-xl font-semibold">Upload New Persona</h2>

          <div className="space-y-4">
            {/* Name Input */}
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-300">
                Persona Name
              </label>
              <input
                type="text"
                value={newPersonaName}
                onChange={(e) => setNewPersonaName(e.target.value)}
                placeholder="e.g., Morgan v1"
                className="w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-2 text-white placeholder-slate-500 focus:border-blue-500 focus:outline-none"
              />
            </div>

            {/* File Drop Zone */}
            <div
              onDrop={handleDrop}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onClick={() => fileInputRef.current?.click()}
              className={`cursor-pointer rounded-lg border-2 border-dashed p-8 text-center transition-colors ${
                isDragging
                  ? "border-blue-500 bg-blue-500/10"
                  : selectedFile
                    ? "border-emerald-500 bg-emerald-500/10"
                    : "border-slate-700 bg-slate-800/50 hover:border-slate-600"
              }`}
            >
              <input
                ref={fileInputRef}
                type="file"
                accept="image/png,image/jpeg,image/webp,video/mp4,video/webm"
                onChange={(e) => handleFileSelect(e.target.files?.[0] || null)}
                className="hidden"
              />

              {selectedFile ? (
                <div>
                  <p className="font-medium text-emerald-300">{selectedFile.name}</p>
                  <p className="mt-1 text-sm text-slate-400">
                    {(selectedFile.size / 1024 / 1024).toFixed(2)} MB ·{" "}
                    {selectedFile.type}
                  </p>
                </div>
              ) : (
                <div>
                  <p className="font-medium">
                    Drop image or video here, or click to select
                  </p>
                  <p className="mt-1 text-sm text-slate-400">
                    PNG, JPG, WebP, MP4, WebM · Max 50MB · Video max 30s
                  </p>
                </div>
              )}
            </div>

            {/* Upload Error */}
            {uploadState.error && (
              <div className="rounded-lg bg-red-500/10 p-3 text-sm text-red-300">
                {uploadState.error}
              </div>
            )}

            {/* Upload Progress */}
            {uploadState.isUploading && (
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span>Uploading...</span>
                  <span>{uploadState.progress}%</span>
                </div>
                <ProgressBar progress={uploadState.progress} />
              </div>
            )}

            {/* Upload Button */}
            <button
              onClick={handleUpload}
              disabled={!selectedFile || !newPersonaName.trim() || uploadState.isUploading}
              className="w-full rounded-lg bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {uploadState.isUploading ? "Uploading..." : "Upload Persona"}
            </button>
          </div>
        </section>

        {/* Personas List */}
        <section className="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-xl font-semibold">Personas</h2>
            <button
              onClick={fetchPersonas}
              disabled={isLoading}
              className="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm transition-colors hover:bg-slate-700 disabled:opacity-50"
            >
              {isLoading ? "Loading..." : "Refresh"}
            </button>
          </div>

          {error && (
            <div className="mb-4 rounded-lg bg-red-500/10 p-4 text-red-300">
              {error}
            </div>
          )}

          {personas.length === 0 && !isLoading && !error ? (
            <div className="py-12 text-center text-slate-500">
              No personas yet. Upload one above.
            </div>
          ) : (
            <div className="space-y-3">
              {personas.map((persona) => (
                <div
                  key={persona.id}
                  className="flex items-center gap-4 rounded-lg border border-slate-800 bg-slate-800/50 p-4"
                >
                  {/* Thumbnail Placeholder */}
                  <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-lg bg-slate-700 text-2xl">
                    {persona.state === "ready" ? "🎭" : "⏳"}
                  </div>

                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <h3 className="truncate font-medium">{persona.name}</h3>
                      <StatusBadge state={persona.state} />
                    </div>
                    <p className="mt-1 text-sm text-slate-400">
                      ID: {persona.id} · Created: {formatDate(persona.created_at)}
                    </p>
                    {persona.state === "preprocessing" && (
                      <div className="mt-2">
                        <ProgressBar progress={persona.progress_percent} />
                      </div>
                    )}
                    {persona.error && (
                      <p className="mt-1 text-sm text-red-400">{persona.error}</p>
                    )}
                  </div>

                  {/* Actions */}
                  <button
                    onClick={() => handleDelete(persona.id)}
                    className="shrink-0 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-1.5 text-sm text-red-300 transition-colors hover:bg-red-500/20"
                  >
                    Delete
                  </button>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}
