/**
 * Database Migration Runner
 * Executes SQL migrations in order
 */

const fs = require('fs').promises;
const path = require('path');
const dbPool = require('./pool');

class MigrationRunner {
  constructor() {
    this.migrationsDir = path.join(__dirname, 'migrations');
  }

  /**
   * Run all pending migrations
   */
  async runMigrations() {
    try {
      console.log('[Migration] Starting database migrations...');

      // Create migrations tracking table
      await this.createMigrationsTable();

      // Get all migration files
      const files = await fs.readdir(this.migrationsDir);
      const sqlFiles = files
        .filter(f => f.endsWith('.sql'))
        .sort();

      // Get applied migrations
      const applied = await this.getAppliedMigrations();

      // Run pending migrations
      for (const file of sqlFiles) {
        if (!applied.includes(file)) {
          await this.runMigration(file);
        }
      }

      console.log('[Migration] All migrations completed successfully');
    } catch (error) {
      console.error('[Migration] Failed:', error.message);
      throw error;
    }
  }

  /**
   * Create migrations tracking table
   */
  async createMigrationsTable() {
    const query = `
      CREATE TABLE IF NOT EXISTS schema_migrations (
        id SERIAL PRIMARY KEY,
        migration VARCHAR(255) NOT NULL UNIQUE,
        applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );
    `;
    await dbPool.query(query);
  }

  /**
   * Get list of applied migrations
   */
  async getAppliedMigrations() {
    const result = await dbPool.query(
      'SELECT migration FROM schema_migrations ORDER BY migration'
    );
    return result.rows.map(row => row.migration);
  }

  /**
   * Run a single migration
   */
  async runMigration(filename) {
    const filePath = path.join(this.migrationsDir, filename);
    const sql = await fs.readFile(filePath, 'utf8');

    console.log(`[Migration] Running: ${filename}`);

    await dbPool.transaction(async (client) => {
      // Execute migration SQL
      await client.query(sql);

      // Record migration
      await client.query(
        'INSERT INTO schema_migrations (migration) VALUES ($1)',
        [filename]
      );
    });

    console.log(`[Migration] Completed: ${filename}`);
  }

  /**
   * Rollback last migration (for development)
   */
  async rollbackLast() {
    try {
      const result = await dbPool.query(
        'SELECT migration FROM schema_migrations ORDER BY applied_at DESC LIMIT 1'
      );

      if (result.rows.length === 0) {
        console.log('[Migration] No migrations to rollback');
        return;
      }

      const migration = result.rows[0].migration;
      console.log(`[Migration] Rolling back: ${migration}`);

      // Note: Rollback would require down migrations
      // This is a simplified example
      await dbPool.query(
        'DELETE FROM schema_migrations WHERE migration = $1',
        [migration]
      );

      console.log('[Migration] Rollback completed');
    } catch (error) {
      console.error('[Migration] Rollback failed:', error.message);
      throw error;
    }
  }
}

module.exports = new MigrationRunner();
