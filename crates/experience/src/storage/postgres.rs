//! `PostgreSQL` storage implementation.

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use super::ExperienceStore;
use crate::models::{SessionRecord, Skill, Space, TaskRecord};

/// `PostgreSQL`-based storage for experience data.
pub struct PostgresStore {
    #[allow(dead_code)]
    pool: sqlx::PgPool,
}

impl PostgresStore {
    /// Create a new `PostgreSQL` store.
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Create from an existing pool.
    #[must_use]
    pub fn from_pool(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExperienceStore for PostgresStore {
    async fn create_space(&self, space: &Space) -> Result<Space> {
        // TODO: Implement SQL insert
        Ok(space.clone())
    }

    async fn get_space(&self, _id: Uuid) -> Result<Option<Space>> {
        // TODO: Implement SQL select
        Ok(None)
    }

    async fn get_space_by_project(&self, _project_id: &str) -> Result<Option<Space>> {
        // TODO: Implement SQL select
        Ok(None)
    }

    async fn create_session(&self, session: &SessionRecord) -> Result<SessionRecord> {
        // TODO: Implement SQL insert
        Ok(session.clone())
    }

    async fn update_session(&self, _session: &SessionRecord) -> Result<()> {
        // TODO: Implement SQL update
        Ok(())
    }

    async fn get_session(&self, _id: Uuid) -> Result<Option<SessionRecord>> {
        // TODO: Implement SQL select
        Ok(None)
    }

    async fn get_session_by_play_id(&self, _play_id: &str) -> Result<Option<SessionRecord>> {
        // TODO: Implement SQL select
        Ok(None)
    }

    async fn create_task(&self, task: &TaskRecord, _session_id: Uuid) -> Result<TaskRecord> {
        // TODO: Implement SQL insert
        Ok(task.clone())
    }

    async fn update_task(&self, _task: &TaskRecord) -> Result<()> {
        // TODO: Implement SQL update
        Ok(())
    }

    async fn get_tasks_for_session(&self, _session_id: Uuid) -> Result<Vec<TaskRecord>> {
        // TODO: Implement SQL select
        Ok(Vec::new())
    }

    async fn create_skill(&self, skill: &Skill) -> Result<Skill> {
        // TODO: Implement SQL insert with pgvector
        Ok(skill.clone())
    }

    async fn update_skill(&self, _skill: &Skill) -> Result<()> {
        // TODO: Implement SQL update
        Ok(())
    }

    async fn get_skill(&self, _id: Uuid) -> Result<Option<Skill>> {
        // TODO: Implement SQL select
        Ok(None)
    }

    async fn search_skills_by_embedding(
        &self,
        _embedding: &[f32],
        _space_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<Skill>> {
        // TODO: Implement vector similarity search
        Ok(Vec::new())
    }

    async fn get_skills_for_space(&self, _space_id: Uuid, _limit: usize) -> Result<Vec<Skill>> {
        // TODO: Implement SQL select
        Ok(Vec::new())
    }
}
