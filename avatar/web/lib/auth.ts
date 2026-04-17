/**
 * Better Auth configuration for the avatar web app.
 *
 * Email/password + GitHub OAuth with Postgres session storage.
 */

import { betterAuth } from "better-auth";
import { drizzleAdapter } from "better-auth/adapters/drizzle";
// Placeholder db import - actual implementation when Postgres is available
// import { db } from "./db";

// Mock db for development
const db = {} as any;

export const auth = betterAuth({
  database: drizzleAdapter(db, {
    provider: "pg", // PostgreSQL
  }),
  socialProviders: {
    github: {
      clientId: process.env.GITHUB_CLIENT_ID!,
      clientSecret: process.env.GITHUB_CLIENT_SECRET!,
    },
  },
  session: {
    // Session stored in Postgres via drizzle adapter
    expiresIn: 60 * 60 * 24 * 7, // 7 days
    updateAge: 60 * 60 * 24, // 1 day
  },
  advanced: {
    // Allow cross-origin requests from the embed domain
    crossSubDomainCookies: {
      enabled: true,
    },
  },
});

// Export types for use in API routes
export type Auth = typeof auth;
