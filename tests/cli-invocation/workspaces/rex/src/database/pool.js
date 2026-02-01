/**
 * Database Connection Pool
 * Manages PostgreSQL connections with proper pooling and error handling
 */

const { Pool } = require('pg');
const config = require('../config');

class DatabasePool {
  constructor() {
    this.pool = null;
    this.isConnected = false;
  }

  /**
   * Initialize database connection pool
   */
  async connect() {
    try {
      this.pool = new Pool({
        host: config.database.host,
        port: config.database.port,
        database: config.database.name,
        user: config.database.user,
        password: config.database.password,
        max: config.database.poolMax,
        idleTimeoutMillis: config.database.idleTimeout,
        connectionTimeoutMillis: config.database.connectionTimeout,
      });

      // Test connection
      const client = await this.pool.connect();
      await client.query('SELECT NOW()');
      client.release();

      this.isConnected = true;
      console.log(`[Database] Connected to PostgreSQL at ${config.database.host}:${config.database.port}`);

      // Handle pool errors
      this.pool.on('error', (err) => {
        console.error('[Database] Unexpected pool error:', err);
        this.isConnected = false;
      });

      return this.pool;
    } catch (error) {
      console.error('[Database] Connection failed:', error.message);
      throw new Error('Database connection failed');
    }
  }

  /**
   * Get a client from the pool
   */
  async getClient() {
    if (!this.isConnected) {
      throw new Error('Database not connected');
    }
    return this.pool.connect();
  }

  /**
   * Execute a query with automatic client management
   */
  async query(text, params) {
    if (!this.isConnected) {
      throw new Error('Database not connected');
    }

    const start = Date.now();
    try {
      const result = await this.pool.query(text, params);
      const duration = Date.now() - start;

      if (config.env === 'development') {
        console.log(`[Database Query] ${duration}ms - ${text.substring(0, 100)}`);
      }

      return result;
    } catch (error) {
      console.error('[Database Query Error]:', error.message);
      throw error;
    }
  }

  /**
   * Execute transaction with automatic rollback on error
   */
  async transaction(callback) {
    const client = await this.getClient();

    try {
      await client.query('BEGIN');
      const result = await callback(client);
      await client.query('COMMIT');
      return result;
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Close all database connections
   */
  async disconnect() {
    if (this.pool) {
      await this.pool.end();
      this.isConnected = false;
      console.log('[Database] Disconnected from PostgreSQL');
    }
  }

  /**
   * Get pool statistics
   */
  getStats() {
    if (!this.pool) {
      return null;
    }

    return {
      total: this.pool.totalCount,
      idle: this.pool.idleCount,
      waiting: this.pool.waitingCount,
    };
  }

  /**
   * Check if database is healthy
   */
  async healthCheck() {
    try {
      if (!this.isConnected) {
        return { status: 'disconnected', message: 'Not connected to database' };
      }

      const result = await this.query('SELECT 1 as health');
      const stats = this.getStats();

      return {
        status: 'healthy',
        connected: true,
        stats,
      };
    } catch (error) {
      return {
        status: 'unhealthy',
        message: error.message,
      };
    }
  }
}

// Export singleton instance
const dbPool = new DatabasePool();

module.exports = dbPool;
