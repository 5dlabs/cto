#[derive(Debug, Clone, PartialEq, strum::Display)]
pub enum InstallationState {
    #[strum(serialize = "ready")]
    Ready,

    #[strum(serialize = "rescue_mode")]
    RescueMode,

    #[strum(serialize = "rsync_image")]
    RsyncImage,

    #[strum(serialize = "dd_write")]
    DdWrite,

    #[strum(serialize = "installing")]
    Installing,

    #[strum(serialize = "booting")]
    Booting,

    #[strum(serialize = "bootstrapped")]
    Bootstrapped,

    #[strum(serialize = "failed")]
    Failed(String),
}

impl Default for InstallationState {
    fn default() -> Self {
        Self::Ready
    }
}

pub mod orchestrator;
pub use orchestrator::Orchestrator;
