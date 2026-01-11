import postgres from "postgres";
import { drizzle } from "drizzle-orm/postgres-js";
import * as schema from "./schema";

// Check if we're in build phase (Next.js static generation)
const isBuildPhase = process.env.NEXT_PHASE === "phase-production-build";

// Get database URL with build-time fallback
const getDatabaseUrl = (): string => {
  const envUrl = process.env.DATABASE_URL;
  if (!envUrl || envUrl.trim().length === 0) {
    // During build phase, use a dummy URL that won't actually be used
    if (isBuildPhase) {
      return "postgresql://dummy:password@localhost:5432/dummy"; // pragma: allowlist secret
    }
    console.error("[DB] DATABASE_URL is not set!");
    return "";
  }
  return envUrl.trim();
};

const databaseUrl = getDatabaseUrl();

// Create postgres.js client
// During build phase, create a dummy client that won't connect
const sql = databaseUrl
  ? postgres(databaseUrl, {
      // Connection pool settings
      max: 10,
      idle_timeout: 20,
      connect_timeout: 10,
    })
  : (null as unknown as ReturnType<typeof postgres>);

export const db = sql ? drizzle(sql, { schema }) : (null as unknown as ReturnType<typeof drizzle>);
