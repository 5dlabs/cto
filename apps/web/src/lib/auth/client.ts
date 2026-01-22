import { createAuthClient } from 'better-auth/react';

// Use relative URLs - Better Auth will auto-detect the current host
// This avoids build-time vs runtime URL mismatches
export const authClient = createAuthClient();

export const { signIn, signOut, useSession } = authClient;
