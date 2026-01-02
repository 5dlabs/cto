//! Webhook handlers for PM integration.

pub mod agent_comms;
pub mod agent_session;
pub mod callbacks;
pub mod github;
pub mod intake;
pub mod oauth;
pub mod play;
pub mod play_state;

pub use agent_comms::{
    broadcast_to_session, find_agents_by_issue, find_running_agents, init_global_router,
    route_message_global, send_message_to_agent, AgentMessage, AgentRouter, CachedPodInfo,
    RunningAgent, SessionCache,
};
pub use agent_session::{
    handle_agent_session_created, handle_agent_session_prompted, AgentSessionContext,
};
pub use callbacks::{handle_intake_complete, handle_tasks_json_callback, CallbackState};
pub use github::{handle_github_webhook, IntakeMetadata, PullRequestEvent};
pub use intake::{IntakeRequest, IntakeResult, IntakeTask, TasksJson};
pub use oauth::{handle_oauth_callback, handle_oauth_start};
pub use play::{PlayRequest, PlayResult};
pub use play_state::{
    determine_bolt_stage, get_state_for_agent, mark_task_done, update_play_stage, BoltStage,
};
