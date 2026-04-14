import index from "./index.html";

const QDRANT_URL = process.env.QDRANT_URL || "http://localhost:6333";
const COLLECTION = process.env.QDRANT_COLLECTION || "cto_memory";
const PORT = parseInt(process.env.PORT || "3847");

async function qdrant(path: string, body?: unknown) {
  const url = `${QDRANT_URL}/collections/${COLLECTION}${path}`;
  const opts: RequestInit = {
    headers: { "Content-Type": "application/json" },
  };
  if (body) {
    opts.method = "POST";
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(url, opts);
  if (!res.ok) throw new Error(`Qdrant ${path}: ${res.status} ${await res.text()}`);
  return res.json();
}

async function getCollection() {
  const data = await qdrant("");
  const info = data.result;
  return {
    points_count: info.points_count,
    vectors_count: info.vectors_count,
    segments_count: info.segments_count,
  };
}

async function getNamespaces() {
  const data = await qdrant("/points/scroll", {
    limit: 100,
    with_payload: { include: ["user_id"] },
    with_vector: false,
  });

  const nsCounts: Record<string, number> = {};
  for (const point of data.result.points) {
    const uid = point.payload?.user_id || "unknown";
    nsCounts[uid] = (nsCounts[uid] || 0) + 1;
  }

  return Object.entries(nsCounts)
    .map(([namespace, count]) => ({
      namespace,
      count,
      tier: getTier(namespace),
    }))
    .sort((a, b) => a.namespace.localeCompare(b.namespace));
}

function getTier(ns: string): string {
  if (!ns || ns === "jonathon") return "portfolio";
  if (ns.startsWith("jonathon:agent:")) return "morgan";
  if (ns.match(/:task:\d+:/)) return "agent";
  if (ns.match(/:task:\d+$/)) return "task";
  if (ns.match(/:project:/)) return "project";
  return "other";
}

async function getMemories(namespace?: string, category?: string, _query?: string) {
  const filter: { must: unknown[] } = { must: [] };
  if (namespace) {
    filter.must.push({ key: "user_id", match: { text: namespace } });
  }
  if (category) {
    filter.must.push({ key: "category", match: { value: category } });
  }

  const body: Record<string, unknown> = {
    limit: 50,
    with_payload: true,
    with_vector: false,
  };

  if (filter.must.length > 0) {
    body.filter = filter;
  }

  const data = await qdrant("/points/scroll", body);
  return data.result.points.map((p: any) => ({
    id: p.id,
    payload: p.payload,
    score: null,
  }));
}

Bun.serve({
  port: PORT,
  routes: {
    "/": index,
    "/api/collection": {
      async GET() {
        try {
          return Response.json(await getCollection());
        } catch (e: any) {
          return Response.json({ error: e.message }, { status: 502 });
        }
      },
    },
    "/api/namespaces": {
      async GET() {
        try {
          return Response.json(await getNamespaces());
        } catch (e: any) {
          return Response.json({ error: e.message }, { status: 502 });
        }
      },
    },
    "/api/memories": {
      async GET(req: Request) {
        try {
          const url = new URL(req.url);
          const namespace = url.searchParams.get("namespace") || undefined;
          const category = url.searchParams.get("category") || undefined;
          const query = url.searchParams.get("query") || undefined;
          return Response.json(await getMemories(namespace, category, query));
        } catch (e: any) {
          return Response.json({ error: e.message }, { status: 502 });
        }
      },
    },
  },
  development: {
    hmr: true,
    console: true,
  },
});

console.log(`Memory Dashboard running on http://localhost:${PORT}`);
console.log(`Qdrant: ${QDRANT_URL}/collections/${COLLECTION}`);
