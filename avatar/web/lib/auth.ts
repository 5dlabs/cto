/**
 * Better Auth configuration for the avatar web app.
 *
 * For the Phase 1 avatar runtime proof, auth is intentionally lightweight so
 * the app can build and the protected admin surface can still evolve later.
 */

import { betterAuth } from "better-auth";

const githubEnabled = Boolean(
  process.env.GITHUB_CLIENT_ID && process.env.GITHUB_CLIENT_SECRET,
);

export const auth = betterAuth({
  ...(githubEnabled
    ? {
        socialProviders: {
          github: {
            clientId: process.env.GITHUB_CLIENT_ID as string,
            clientSecret: process.env.GITHUB_CLIENT_SECRET as string,
          },
        },
      }
    : {}),
  session: {
    expiresIn: 60 * 60 * 24 * 7,
    updateAge: 60 * 60 * 24,
  },
  advanced: {
    crossSubDomainCookies: {
      enabled: true,
    },
  },
});

export type Auth = typeof auth;
