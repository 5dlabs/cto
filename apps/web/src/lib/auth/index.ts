import { betterAuth } from "better-auth";
import { drizzleAdapter } from "better-auth/adapters/drizzle";
import { db } from "@/lib/db";
import * as schema from "@/lib/db/schema";

// Check if we're in build phase (Next.js static generation)
// During build, env vars may not be available
const isBuildPhase = process.env.NEXT_PHASE === "phase-production-build";

// Determine the base URL - critical for OAuth callbacks
// This MUST be set correctly for OAuth to work
const getBaseURL = (): string => {
  // Explicit app URL takes priority
  if (process.env.NEXT_PUBLIC_APP_URL) {
    return process.env.NEXT_PUBLIC_APP_URL;
  }
  // Vercel deployment URL
  if (process.env.VERCEL_URL) {
    return `https://${process.env.VERCEL_URL}`;
  }
  // Default for development
  return "http://localhost:3000";
};

const baseURL = getBaseURL();

// Get GitHub OAuth credentials with validation
const getGitHubCredentials = () => {
  const clientId = process.env.GITHUB_CLIENT_ID;
  const clientSecret = process.env.GITHUB_CLIENT_SECRET;

  // During build phase, use placeholder values
  if (isBuildPhase) {
    return {
      clientId: "build-time-placeholder",
      clientSecret: "build-time-placeholder",  // pragma: allowlist secret
    };
  }

  // At runtime, validate that credentials are set
  if (!clientId || !clientSecret) {
    console.error("[Auth] CRITICAL: GitHub OAuth credentials are not configured!", {
      hasClientId: !!clientId,
      hasClientSecret: !!clientSecret,
      baseURL,
      nodeEnv: process.env.NODE_ENV,
    });
  }

  return {
    // Use empty string instead of dummy values - this will cause a clear OAuth error
    // instead of a confusing 500 from GitHub
    clientId: clientId || "",
    clientSecret: clientSecret || "",
  };
};

const githubCredentials = getGitHubCredentials();

// Log configuration at startup for debugging (non-production only)
if (process.env.NODE_ENV !== "production" && !isBuildPhase) {
  console.log("[Auth] Configuration:", {
    baseURL,
    hasGitHubClientId: !!process.env.GITHUB_CLIENT_ID,
    hasGitHubClientSecret: !!process.env.GITHUB_CLIENT_SECRET,
    hasBetterAuthSecret: !!process.env.BETTER_AUTH_SECRET,
    hasDatabaseUrl: !!process.env.DATABASE_URL,
  });
}

export const auth = betterAuth({
  // Secret for encryption, signing, and hashing operations
  // Required for production - uses a build-time placeholder during static generation
  secret: process.env.BETTER_AUTH_SECRET || (isBuildPhase ? "build-phase-placeholder" : "development-fallback-secret"),
  database: drizzleAdapter(db, {
    provider: "pg",
    // Spread entire schema and map plural table names to Better Auth's expected singular keys
    schema: {
      ...schema,
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
      clientId: githubCredentials.clientId,
      clientSecret: githubCredentials.clientSecret,
      // user:email is required for GitHub OAuth to get the user's email
      scope: ["user:email", "read:user", "repo"],
    },
  },
  session: {
    cookieCache: {
      enabled: true,
      maxAge: 60 * 5, // 5 minutes
    },
  },
  // Base URL for auth callbacks - critical for OAuth redirect URLs
  // OAuth callback URL will be: {baseURL}/api/auth/callback/github
  baseURL,
  trustedOrigins: [
    // Production URL
    "https://app.5dlabs.ai",
    // Development
    "http://localhost:3000",
    // Allow env override
    ...(process.env.NEXT_PUBLIC_APP_URL ? [process.env.NEXT_PUBLIC_APP_URL] : []),
    // Vercel preview URLs
    ...(process.env.VERCEL_URL ? [`https://${process.env.VERCEL_URL}`] : []),
  ],
});

export type Session = typeof auth.$Infer.Session;
