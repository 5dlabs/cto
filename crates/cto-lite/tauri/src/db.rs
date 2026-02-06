//! SQLite database for local state storage

use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};

/// Database wrapper with thread-safe connection
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Run database migrations
    pub fn migrate(&self) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute_batch(
            r#"
            -- Configuration key-value store
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Setup wizard progress
            CREATE TABLE IF NOT EXISTS setup_progress (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                current_step INTEGER DEFAULT 0,
                completed_at TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Ensure single row for setup progress
            INSERT OR IGNORE INTO setup_progress (id, current_step) VALUES (1, 0);

            -- Workflow history
            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                repository TEXT NOT NULL,
                task_id INTEGER,
                status TEXT NOT NULL,
                phase TEXT,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                error_message TEXT
            );

            -- Workflow logs (streaming append)
            CREATE TABLE IF NOT EXISTS workflow_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                FOREIGN KEY (workflow_id) REFERENCES workflows(id)
            );

            -- Connected GitHub repositories
            CREATE TABLE IF NOT EXISTS repositories (
                id TEXT PRIMARY KEY,
                full_name TEXT NOT NULL UNIQUE,
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                default_branch TEXT DEFAULT 'main',
                webhook_id TEXT,
                connected_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Tunnel configuration
            CREATE TABLE IF NOT EXISTS tunnels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                cloudflare_tunnel_id TEXT,
                url TEXT,
                status TEXT DEFAULT 'inactive',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Index for workflow queries
            CREATE INDEX IF NOT EXISTS idx_workflows_repository ON workflows(repository);
            CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
            CREATE INDEX IF NOT EXISTS idx_workflow_logs_workflow_id ON workflow_logs(workflow_id);
            "#,
        )?;

        Ok(())
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> AppResult<Option<String>> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM config WHERE key = ?",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a configuration value
    pub fn set_config(&self, key: &str, value: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
            params![key, value],
        )?;

        Ok(())
    }

    /// Get setup progress
    pub fn get_setup_progress(&self) -> AppResult<(i32, bool)> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        let (step, completed): (i32, Option<String>) = conn.query_row(
            "SELECT current_step, completed_at FROM setup_progress WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        Ok((step, completed.is_some()))
    }

    /// Update setup progress
    #[allow(dead_code)]
    pub fn set_setup_progress(&self, step: i32) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute(
            "UPDATE setup_progress SET current_step = ? WHERE id = 1",
            params![step],
        )?;

        Ok(())
    }

    /// Mark setup as complete
    pub fn mark_setup_complete(&self) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute(
            "UPDATE setup_progress SET completed_at = CURRENT_TIMESTAMP WHERE id = 1",
            [],
        )?;

        Ok(())
    }

    /// Add or update a repository
    #[allow(dead_code)]
    pub fn upsert_repository(
        &self,
        id: &str,
        full_name: &str,
        owner: &str,
        name: &str,
        default_branch: &str,
    ) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO repositories (id, full_name, owner, name, default_branch, connected_at)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
            params![id, full_name, owner, name, default_branch],
        )?;

        Ok(())
    }

    /// Set webhook ID for a repository
    #[allow(dead_code)]
    pub fn set_repository_webhook(&self, full_name: &str, webhook_id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute(
            "UPDATE repositories SET webhook_id = ? WHERE full_name = ?",
            params![webhook_id, full_name],
        )?;

        Ok(())
    }

    /// Insert a workflow record
    #[allow(dead_code)]
    pub fn insert_workflow(
        &self,
        id: &str,
        name: &str,
        repository: &str,
        task_id: Option<i32>,
        status: &str,
    ) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute(
            r#"
            INSERT INTO workflows (id, name, repository, task_id, status, started_at)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
            params![id, name, repository, task_id, status],
        )?;

        Ok(())
    }

    /// Update workflow status
    #[allow(dead_code)]
    pub fn update_workflow_status(
        &self,
        id: &str,
        status: &str,
        phase: Option<&str>,
        error: Option<&str>,
    ) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        if status == "Succeeded" || status == "Failed" || status == "Error" {
            conn.execute(
                r#"
                UPDATE workflows 
                SET status = ?, phase = ?, error_message = ?, finished_at = CURRENT_TIMESTAMP
                WHERE id = ?
                "#,
                params![status, phase, error, id],
            )?;
        } else {
            conn.execute(
                "UPDATE workflows SET status = ?, phase = ?, error_message = ? WHERE id = ?",
                params![status, phase, error, id],
            )?;
        }

        Ok(())
    }

    /// Append a log entry
    #[allow(dead_code)]
    pub fn append_workflow_log(
        &self,
        workflow_id: &str,
        level: &str,
        message: &str,
    ) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        conn.execute(
            r#"
            INSERT INTO workflow_logs (workflow_id, timestamp, level, message)
            VALUES (?, CURRENT_TIMESTAMP, ?, ?)
            "#,
            params![workflow_id, level, message],
        )?;

        Ok(())
    }
}
