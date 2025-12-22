//! Webhook handlers for PM integration.

pub mod agent_comms;
pub mod callbacks;
pub mod github;
pub mod intake;
pub mod play;

pub use agent_comms::{
    broadcast_to_session, find_agents_by_issue, find_running_agents, init_global_router,
    route_message_global, send_message_to_agent, AgentMessage, AgentRouter, CachedPodInfo,
    RunningAgent, SessionCache,
};
pub use callbacks::{handle_intake_complete, handle_tasks_json_callback, CallbackState};
pub use github::{handle_github_webhook, IntakeMetadata, PullRequestEvent};
pub use intake::{IntakeRequest, IntakeResult, IntakeTask, TasksJson};
pub use play::{PlayRequest, PlayResult};
