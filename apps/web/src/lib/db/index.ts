import { neon } from "@neondatabase/serverless";
import { drizzle } from "drizzle-orm/neon-http";
import * as schema from "./schema";

// Provide a dummy connection string for build time if DATABASE_URL is not set or empty
// The neon client will be initialized but won't actually connect during build
// Use a valid Neon connection string format even for dummy values
const getDatabaseUrl = (): string => {
  const envUrl = process.env.DATABASE_URL;
  if (!envUrl) {
    return "postgresql://dummy:password@ep-dummy-12345678.us-east-2.aws.neon.tech/dbname?sslmode=require"; // pragma: allowlist secret
  }
  const trimmed = envUrl.trim();
  if (trimmed.length === 0) {
    return "postgresql://dummy:password@ep-dummy-12345678.us-east-2.aws.neon.tech/dbname?sslmode=require"; // pragma: allowlist secret
  }
  return trimmed;
};

const databaseUrl = getDatabaseUrl();
const sql = neon(databaseUrl);
export const db = drizzle(sql, { schema });
