//! `PostgreSQL` storage layer for experience data.

mod postgres;

pub use postgres::PostgresStore;

use crate::models::{SessionRecord, Skill, Space, TaskRecord};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Storage interface for experience data.
#[async_trait]
pub trait ExperienceStore: Send + Sync {
    // Space operations
    async fn create_space(&self, space: &Space) -> Result<Space>;
    async fn get_space(&self, id: Uuid) -> Result<Option<Space>>;
    async fn get_space_by_project(&self, project_id: &str) -> Result<Option<Space>>;

    // Session operations
    async fn create_session(&self, session: &SessionRecord) -> Result<SessionRecord>;
    async fn update_session(&self, session: &SessionRecord) -> Result<()>;
    async fn get_session(&self, id: Uuid) -> Result<Option<SessionRecord>>;
    async fn get_session_by_play_id(&self, play_id: &str) -> Result<Option<SessionRecord>>;

    // Task operations
    async fn create_task(&self, task: &TaskRecord, session_id: Uuid) -> Result<TaskRecord>;
    async fn update_task(&self, task: &TaskRecord) -> Result<()>;
    async fn get_tasks_for_session(&self, session_id: Uuid) -> Result<Vec<TaskRecord>>;

    // Skill operations
    async fn create_skill(&self, skill: &Skill) -> Result<Skill>;
    async fn update_skill(&self, skill: &Skill) -> Result<()>;
    async fn get_skill(&self, id: Uuid) -> Result<Option<Skill>>;
    async fn search_skills_by_embedding(
        &self,
        embedding: &[f32],
        space_id: Uuid,
        limit: usize,
    ) -> Result<Vec<Skill>>;
    async fn get_skills_for_space(&self, space_id: Uuid, limit: usize) -> Result<Vec<Skill>>;
}
