import { betterAuth } from "better-auth";
import { drizzleAdapter } from "better-auth/adapters/drizzle";
import { db } from "@/lib/db";
import * as schema from "@/lib/db/schema";

export const auth = betterAuth({
  // Secret for encryption, signing, and hashing operations
  // Required for production - defaults to dummy for build time only
  secret: process.env.BETTER_AUTH_SECRET || "build-time-dummy-secret-not-for-production",
  database: drizzleAdapter(db, {
    provider: "pg",
    schema: {
      user: schema.users,
      session: schema.sessions,
      account: schema.accounts,
      verification: schema.verifications,
    },
  }),
  emailAndPassword: {
    enabled: false, // GitHub OAuth only for now
  },
  socialProviders: {
    github: {
      clientId: process.env.GITHUB_CLIENT_ID || "dummy-client-id",
      clientSecret: process.env.GITHUB_CLIENT_SECRET || "dummy-client-secret",
      scope: ["user:email", "read:user", "repo"],
    },
  },
  session: {
    cookieCache: {
      enabled: true,
      maxAge: 60 * 5, // 5 minutes
    },
  },
  // Base URL for auth callbacks - uses NEXT_PUBLIC_APP_URL if set, otherwise auto-detects
  baseURL: process.env.NEXT_PUBLIC_APP_URL,
  trustedOrigins: [
    // Production URL
    "https://app.5dlabs.ai",
    // Development
    "http://localhost:3000",
    // Allow env override
    ...(process.env.NEXT_PUBLIC_APP_URL ? [process.env.NEXT_PUBLIC_APP_URL] : []),
  ],
});

export type Session = typeof auth.$Infer.Session;
