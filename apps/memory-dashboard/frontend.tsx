import React, { useState, useEffect, useCallback } from "react";
import { createRoot } from "react-dom/client";
import "./style.css";

interface Memory {
  id: string;
  payload: {
    memory?: string;
    data?: string;
    user_id?: string;
    category?: string;
    agent_id?: string;
    hash?: string;
    created_at?: string;
    updated_at?: string;
    [key: string]: unknown;
  };
  score?: number;
}

interface NamespaceInfo {
  namespace: string;
  count: number;
  tier: string;
}

interface CollectionInfo {
  points_count: number;
  vectors_count: number;
  segments_count: number;
}

const API_BASE = "/api";

function App() {
  const [memories, setMemories] = useState<Memory[]>([]);
  const [namespaces, setNamespaces] = useState<NamespaceInfo[]>([]);
  const [collectionInfo, setCollectionInfo] = useState<CollectionInfo | null>(null);
  const [selectedNs, setSelectedNs] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const fetchCollection = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/collection`);
      if (res.ok) setCollectionInfo(await res.json());
    } catch {}
  }, []);

  const fetchNamespaces = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/namespaces`);
      if (res.ok) setNamespaces(await res.json());
    } catch {}
  }, []);

  const fetchMemories = useCallback(async (ns?: string, query?: string, category?: string) => {
    setLoading(true);
    setError("");
    try {
      const params = new URLSearchParams();
      if (ns) params.set("namespace", ns);
      if (query) params.set("query", query);
      if (category) params.set("category", category);
      const res = await fetch(`${API_BASE}/memories?${params}`);
      if (!res.ok) throw new Error(await res.text());
      setMemories(await res.json());
    } catch (e: any) {
      setError(e.message || "Failed to fetch memories");
      setMemories([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchCollection();
    fetchNamespaces();
    fetchMemories();
  }, []);

  const handleSearch = () => {
    fetchMemories(selectedNs, searchQuery, categoryFilter);
  };

  const handleNsClick = (ns: string) => {
    const newNs = selectedNs === ns ? "" : ns;
    setSelectedNs(newNs);
    fetchMemories(newNs, searchQuery, categoryFilter);
  };

  const categories = [
    "", "task_objective", "task_progress", "task_completion",
    "blocker", "handoff", "intake_decision", "architecture",
    "debugging", "user_preference", "configuration", "identity",
    "decision", "technical", "project", "operational",
  ];

  const tierLabel = (ns: string): string => {
    if (!ns || ns === "jonathon") return "portfolio";
    if (ns.match(/^jonathon:agent:/)) return "morgan";
    if (ns.match(/:task:\d+:/)) return "agent";
    if (ns.match(/:task:\d+$/)) return "task";
    if (ns.match(/:project:/)) return "project";
    return "other";
  };

  return (
    <div className="app">
      <header>
        <h1><span>●</span> CTO Memory Dashboard</h1>
        <div className="stats-bar">
          <div>Points: <span className="stat-value">{collectionInfo?.points_count ?? "–"}</span></div>
          <div>Namespaces: <span className="stat-value">{namespaces.length}</span></div>
          <div>Collection: <span className="stat-value">cto_memory</span></div>
        </div>
      </header>

      <div className="controls">
        <input
          type="text"
          placeholder="Search memories..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSearch()}
        />
        <select value={categoryFilter} onChange={(e) => { setCategoryFilter(e.target.value); }}>
          <option value="">All categories</option>
          {categories.filter(Boolean).map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>
        <button onClick={handleSearch}>Search</button>
        <button className="secondary" onClick={() => {
          setSearchQuery("");
          setCategoryFilter("");
          setSelectedNs("");
          fetchMemories();
        }}>Reset</button>
      </div>

      <div className="layout">
        <div className="sidebar">
          <h3>Namespaces</h3>
          <div
            className={`ns-item ${selectedNs === "" ? "active" : ""}`}
            onClick={() => handleNsClick("")}
          >
            <span>All</span>
            <span className="count">{collectionInfo?.points_count ?? 0}</span>
          </div>
          {namespaces.map((ns) => (
            <div
              key={ns.namespace}
              className={`ns-item ${selectedNs === ns.namespace ? "active" : ""}`}
              onClick={() => handleNsClick(ns.namespace)}
            >
              <span>{ns.namespace.replace("jonathon:", "").replace("jonathon", "⊕ portfolio") || "⊕ portfolio"}</span>
              <span className="count">{ns.count}</span>
            </div>
          ))}
        </div>

        <div className="memories">
          {error && <div className="error">{error}</div>}
          {loading && <div className="loading">Searching memories...</div>}
          {!loading && memories.length === 0 && !error && (
            <div className="empty-state">
              <h3>No memories found</h3>
              <p>Memories will appear here as agents work on tasks and store information.</p>
            </div>
          )}
          {memories.map((m) => (
            <div key={m.id} className="memory-card">
              <div className="memory-meta">
                {m.payload.category && <span className="tag category">{m.payload.category}</span>}
                {m.payload.agent_id && <span className="tag agent">{m.payload.agent_id}</span>}
                {m.payload.user_id && <span className="tag tier">{tierLabel(m.payload.user_id)}</span>}
                {m.score != null && <span className="tag score">{m.score.toFixed(3)}</span>}
              </div>
              <div className="memory-text">{m.payload.memory || m.payload.data || JSON.stringify(m.payload)}</div>
              <div className="memory-id">
                {m.id.slice(0, 12)}
                {m.payload.created_at && ` · ${new Date(m.payload.created_at).toLocaleString()}`}
                {m.payload.user_id && ` · ${m.payload.user_id}`}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

const root = createRoot(document.getElementById("root")!);
root.render(<App />);
