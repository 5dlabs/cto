/**
 * Database migration script for Better Auth tables
 *
 * This script uses drizzle-kit push to sync the schema with the database.
 * It's designed to be run as an init container before the web app starts.
 *
 * Usage:
 *   npx tsx src/lib/db/migrate.ts
 *
 * Required environment variables:
 *   DATABASE_URL - PostgreSQL connection string
 */

import { execSync } from 'child_process';

const databaseUrl = process.env.DATABASE_URL;

if (!databaseUrl) {
  console.error('[Migrate] ERROR: DATABASE_URL is not set');
  process.exit(1);
}

console.log('[Migrate] Starting database migration...');
console.log('[Migrate] Database URL:', databaseUrl.replace(/:[^:@]+@/, ':***@')); // Mask password

try {
  // Run drizzle-kit push to sync schema
  execSync('npx drizzle-kit push --force', {
    stdio: 'inherit',
    cwd: process.cwd(),
    env: {
      ...process.env,
      DATABASE_URL: databaseUrl,
    },
  });

  console.log('[Migrate] Migration completed successfully!');
  process.exit(0);
} catch (error) {
  console.error('[Migrate] Migration failed:', error);
  process.exit(1);
}
