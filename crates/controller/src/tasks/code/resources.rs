use super::agent::AgentClassifier;
use super::naming::ResourceNaming;
use super::watcher::coordination_configmap_name;
use crate::cli::types::{CLIType, Provider};
use crate::crds::coderun::HarnessAgent;
use crate::crds::{CLIConfig, CodeRun};
use crate::tasks::cleanup::{
    LABEL_CLEANUP_KIND, LABEL_CLEANUP_RUN, LABEL_CLEANUP_SCOPE, SCOPE_RUN,
};
use crate::tasks::config::{ControllerConfig, ResolvedSecretBinding};
use crate::tasks::types::{github_app_secret_name, Context, Error, Result};
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim, Pod},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::ResourceExt;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tracing::{error, info, warn};

// ─── Shared code-server config ────────────────────────────────────────────
//
// Canonical VS Code user settings and workbench storage state shared with the
// persistent Helm deployment (`infra/charts/openclaw-agent`). Edit the source
// files in `shared/code-server-config/`; both consumers pick up changes at
// build/render time. See `shared/code-server-config/README.md`.
const CODE_SERVER_SETTINGS_JSON: &str =
    include_str!("../../../../../shared/code-server-config/settings.json");
const CODE_SERVER_STORAGE_JSON: &str =
    include_str!("../../../../../shared/code-server-config/storage.json");

// ─── Effective Provider Resolution ────────────────────────────────────────
//
// Single source of truth for how a CodeRun resolves its provider, base URL,
// secret key, and model identity. Every env-var emitter, ConfigMap generator,
// and DD-tag builder MUST go through `resolve_effective_provider`.

/// Fully resolved provider configuration for a CodeRun.
///
/// All env var logic, config file generation, and observability tags
/// should read from this struct instead of inspecting raw CRD fields.
#[derive(Debug, Clone)]
pub struct EffectiveProviderConfig {
    /// The resolved provider.
    pub provider: Provider,
    /// How the provider was determined — for debugging / log messages.
    pub source: ProviderSource,
    /// Base URL for inference (provider default or CRD override).
    pub base_url: Option<String>,
    /// Secret key name in `cto-secrets` (e.g. `FIREWORKS_API_KEY`).
    pub secret_key: String,
    /// The model ID as-is from the CRD (before any CLI-specific transforms).
    pub raw_model: String,
    /// CLI type for this run.
    pub cli_type: CLIType,
}

/// How the provider was resolved — logged for debuggability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSource {
    /// Explicitly set in `cliConfig.provider`.
    Explicit,
    /// Carried in legacy `cliConfig.settings.provider` / `settings.modelProvider`.
    LegacySetting,
    /// Resolved from operator-level `cliProviders` config map.
    OperatorConfig,
    /// Inferred from model ID string patterns.
    ModelInference,
}

impl std::fmt::Display for ProviderSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderSource::Explicit => write!(f, "explicit"),
            ProviderSource::LegacySetting => write!(f, "legacy-setting"),
            ProviderSource::OperatorConfig => write!(f, "operator-default"),
            ProviderSource::ModelInference => write!(f, "inferred"),
        }
    }
}

impl EffectiveProviderConfig {
    /// Resolve the effective provider for a CodeRun.
    ///
    /// Precedence (first match wins):
    /// 1. `cliConfig.provider` (explicit CRD field)
    /// 2. `cliConfig.settings.provider` or `settings.modelProvider.name` (legacy)
    /// 3. Controller `cliProviders` config (operator-level default)
    /// 4. `Provider::infer_from_model()` (model ID string patterns)
    /// 5. Fallback to `Fireworks` (current default for the platform)
    #[allow(clippy::too_many_lines)]
    pub fn resolve(code_run: &CodeRun, controller_config: &ControllerConfig) -> Self {
        let cli_type = code_run
            .spec
            .cli_config
            .as_ref()
            .map_or(CLIType::Claude, |cfg| cfg.cli_type);

        let raw_model = code_run
            .spec
            .cli_config
            .as_ref()
            .map_or_else(|| code_run.spec.model.clone(), |cfg| cfg.model.clone());

        // Helper: resolve secret_key from CRD passthrough or provider default
        let resolve_secret_key = |cli_cfg: Option<&CLIConfig>, provider: &Provider| -> String {
            cli_cfg
                .and_then(|c| c.api_key_env_var.clone())
                .unwrap_or_else(|| provider.secret_key().to_string())
        };

        // 1. Explicit cliConfig.provider
        if let Some(ref cli_cfg) = code_run.spec.cli_config {
            if let Some(provider) = cli_cfg.provider {
                let base_url = cli_cfg
                    .provider_base_url
                    .clone()
                    .or_else(|| provider.default_base_url().map(String::from));
                let secret_key = resolve_secret_key(Some(cli_cfg), &provider);
                info!(
                    "Provider resolved ({}): {} for {} (model={})",
                    ProviderSource::Explicit,
                    provider,
                    cli_type,
                    raw_model
                );
                return Self {
                    provider,
                    source: ProviderSource::Explicit,
                    base_url,
                    secret_key,
                    raw_model,
                    cli_type,
                };
            }
        }

        // 2. Legacy settings.provider / settings.modelProvider
        if let Some(ref cli_cfg) = code_run.spec.cli_config {
            let legacy_provider_str = cli_cfg
                .settings
                .get("provider")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    cli_cfg
                        .settings
                        .get("modelProvider")
                        .and_then(|v| v.get("name"))
                        .and_then(|v| v.as_str())
                });
            if let Some(ps) = legacy_provider_str {
                if let Some(provider) = Provider::from_str_ci(ps) {
                    let base_url = cli_cfg
                        .provider_base_url
                        .clone()
                        .or_else(|| provider.default_base_url().map(String::from));
                    let secret_key = resolve_secret_key(Some(cli_cfg), &provider);
                    info!(
                        "Provider resolved ({}): {} for {} (model={})",
                        ProviderSource::LegacySetting,
                        provider,
                        cli_type,
                        raw_model
                    );
                    return Self {
                        provider,
                        source: ProviderSource::LegacySetting,
                        base_url,
                        secret_key,
                        raw_model,
                        cli_type,
                    };
                }
            }
        }

        // 3. Operator-level cliProviders config
        let cli_key = cli_type.to_string().to_lowercase();
        let cli_cfg_ref = code_run.spec.cli_config.as_ref();
        if let Some(provider_name) = controller_config.agent.cli_providers.get(&cli_key) {
            if let Some(provider) = Provider::from_str_ci(provider_name) {
                let base_url = cli_cfg_ref
                    .and_then(|c| c.provider_base_url.clone())
                    .or_else(|| provider.default_base_url().map(String::from));
                let secret_key = resolve_secret_key(cli_cfg_ref, &provider);
                info!(
                    "Provider resolved ({}): {} for {} (model={})",
                    ProviderSource::OperatorConfig,
                    provider,
                    cli_type,
                    raw_model
                );
                return Self {
                    provider,
                    source: ProviderSource::OperatorConfig,
                    base_url,
                    secret_key,
                    raw_model,
                    cli_type,
                };
            }
        }

        // 4. Infer from model ID
        if let Some(provider) = Provider::infer_from_model(&raw_model) {
            let base_url = cli_cfg_ref
                .and_then(|c| c.provider_base_url.clone())
                .or_else(|| provider.default_base_url().map(String::from));
            let secret_key = resolve_secret_key(cli_cfg_ref, &provider);
            info!(
                "Provider resolved ({}): {} for {} (model={})",
                ProviderSource::ModelInference,
                provider,
                cli_type,
                raw_model
            );
            return Self {
                provider,
                source: ProviderSource::ModelInference,
                base_url,
                secret_key,
                raw_model,
                cli_type,
            };
        }

        // 5. Fallback to Fireworks
        warn!(
            "Could not resolve provider for {} (model={}), defaulting to Fireworks",
            cli_type, raw_model
        );
        Self {
            provider: Provider::Fireworks,
            source: ProviderSource::ModelInference,
            base_url: cli_cfg_ref
                .and_then(|c| c.provider_base_url.clone())
                .or_else(|| Provider::Fireworks.default_base_url().map(String::from)),
            secret_key: resolve_secret_key(cli_cfg_ref, &Provider::Fireworks),
            raw_model,
            cli_type,
        }
    }

    /// Build the env vars specific to this provider × CLI combination.
    ///
    /// Returns a Vec of `serde_json::Value` env var objects ready for the
    /// pod spec. This replaces all the scattered `if cli_type ==` blocks.
    #[allow(clippy::too_many_lines)]
    pub fn build_env_vars(&self) -> Vec<serde_json::Value> {
        let mut vars = Vec::new();
        let secret_name = "cto-secrets";

        // ── Provider-level env vars (apply to all CLIs using this provider) ──

        match self.provider {
            Provider::Fireworks => {
                if let Some(ref url) = self.base_url {
                    vars.push(json!({ "name": "ANTHROPIC_BASE_URL", "value": url }));
                }
                vars.push(json!({
                    "name": "ANTHROPIC_AUTH_TOKEN",
                    "valueFrom": { "secretKeyRef": { "name": secret_name, "key": &self.secret_key } }
                }));
                vars.push(json!({
                    "name": "ANTHROPIC_API_KEY",
                    "valueFrom": { "secretKeyRef": { "name": secret_name, "key": &self.secret_key } }
                }));
                vars.push(json!({
                    "name": "FIREWORKS_AI_API_KEY",
                    "valueFrom": { "secretKeyRef": { "name": secret_name, "key": &self.secret_key } }
                }));

                // Model identity env vars for Claude Code / subagents
                let raw = self
                    .raw_model
                    .strip_prefix("fireworks/")
                    .unwrap_or(&self.raw_model);
                vars.push(json!({ "name": "ANTHROPIC_MODEL", "value": raw }));
                vars.push(json!({ "name": "ANTHROPIC_SMALL_FAST_MODEL", "value": raw }));
                vars.push(json!({ "name": "ANTHROPIC_DEFAULT_SONNET_MODEL", "value": raw }));
                vars.push(json!({ "name": "ANTHROPIC_DEFAULT_HAIKU_MODEL", "value": raw }));
                vars.push(json!({ "name": "ANTHROPIC_DEFAULT_OPUS_MODEL", "value": raw }));
            }
            Provider::Google => {
                vars.push(json!({ "name": "GEMINI_MODEL", "value": &self.raw_model }));
            }
            Provider::Anthropic | Provider::OpenAI | Provider::Cursor | Provider::Factory => {
                // Native providers — API keys come from cto-secrets envFrom
            }
            Provider::Moonshot => {
                if let Some(ref url) = self.base_url {
                    vars.push(json!({ "name": "KIMI_BASE_URL", "value": url }));
                }
            }
        }

        // ── CLI-specific env vars (independent of provider) ──

        // ACPX_AGENT: maps CLI type to acpx agent name (factory → droid, rest → identity)
        let acpx_agent = if self.cli_type == CLIType::Factory {
            "droid".to_string()
        } else {
            self.cli_type.to_string()
        };
        vars.push(json!({ "name": "ACPX_AGENT", "value": &acpx_agent }));

        // ACPX_MODEL: the model ID acpx should use, with CLI-specific transforms
        let acpx_model = match (self.cli_type, self.provider) {
            (CLIType::OpenCode, Provider::Fireworks) => format!("fireworks-ai/{}", self.raw_model),
            (CLIType::Kimi, Provider::Fireworks) => "kimi-k2p5-turbo".to_string(),
            (CLIType::Copilot, _) => "gpt-4.1".to_string(),
            (CLIType::Cursor, _) => String::new(), // cursor resolves internally
            _ => self.raw_model.clone(),
        };
        if !acpx_model.is_empty() {
            vars.push(json!({ "name": "ACPX_MODEL", "value": &acpx_model }));
        }

        if self.cli_type == CLIType::Codex {
            vars.push(json!({ "name": "HOME", "value": "/root" }));
            vars.push(json!({ "name": "XDG_CONFIG_HOME", "value": "/root/.config" }));
            vars.push(json!({ "name": "CODEX_HOME", "value": "/root/.codex" }));
        }

        // ── CLI × Provider specializations ──

        if self.cli_type == CLIType::Kimi {
            vars.push(json!({ "name": "KIMI_MODEL_NAME", "value": &self.raw_model }));
            if let Some(ref url) = self.base_url {
                vars.push(json!({ "name": "KIMI_BASE_URL", "value": format!("{}/v1", url.trim_end_matches("/v1").trim_end_matches('/')) }));
            }
        }

        if self.cli_type == CLIType::Copilot {
            vars.push(json!({
                "name": "COPILOT_GITHUB_TOKEN",
                "valueFrom": { "secretKeyRef": { "name": secret_name, "key": "COPILOT_GITHUB_TOKEN" } }
            }));
            if let Some(ref url) = self.base_url {
                let v1_url = format!("{}/v1", url.trim_end_matches("/v1").trim_end_matches('/'));
                vars.push(json!({ "name": "COPILOT_PROVIDER_BASE_URL", "value": &v1_url }));
                vars.push(json!({ "name": "COPILOT_PROVIDER_TYPE", "value": "openai" }));
                vars.push(json!({
                    "name": "COPILOT_PROVIDER_API_KEY",
                    "valueFrom": { "secretKeyRef": { "name": secret_name, "key": &self.secret_key } }
                }));
                vars.push(json!({ "name": "COPILOT_MODEL", "value": "gpt-4.1" }));
                vars.push(json!({ "name": "COPILOT_PROVIDER_MODEL_ID", "value": "gpt-4.1" }));
                vars.push(
                    json!({ "name": "COPILOT_PROVIDER_WIRE_MODEL", "value": &self.raw_model }),
                );
            }
        }

        // Disable ANSI color codes for cleaner Datadog log ingestion
        vars.push(json!({ "name": "NO_COLOR", "value": "1" }));
        vars.push(json!({ "name": "TERM", "value": "dumb" }));

        vars
    }
}

pub struct CodeResourceManager<'a> {
    pub jobs: &'a Api<Job>,
    pub configmaps: &'a Api<ConfigMap>,
    pub pvcs: &'a Api<PersistentVolumeClaim>,
    pub config: &'a Arc<ControllerConfig>,
    pub ctx: &'a Arc<Context>,
}

impl<'a> CodeResourceManager<'a> {
    #[must_use]
    pub fn new(
        jobs: &'a Api<Job>,
        configmaps: &'a Api<ConfigMap>,
        pvcs: &'a Api<PersistentVolumeClaim>,
        config: &'a Arc<ControllerConfig>,
        ctx: &'a Arc<Context>,
    ) -> Self {
        Self {
            jobs,
            configmaps,
            pvcs,
            config,
            ctx,
        }
    }

    #[allow(clippy::too_many_lines)] // Complex function not easily split
    pub async fn reconcile_create_or_update(&self, code_run: &Arc<CodeRun>) -> Result<Action> {
        let name = code_run.name_any();
        let cli_type = code_run
            .spec
            .cli_config
            .as_ref()
            .map_or_else(|| "unknown".to_string(), |c| c.cli_type.to_string());
        let model = &code_run.spec.model;
        let has_acp = code_run.spec.acp.is_some();
        let has_openclaw = code_run.spec.openclaw.is_some();
        let has_skills = code_run.spec.skills_url.is_some();

        info!(
            coderun = %name,
            cli = %cli_type,
            model = %model,
            acp = has_acp,
            openclaw = has_openclaw,
            skills = has_skills,
            "🚀 Creating/updating code resources"
        );

        // STEP: Auto-populate CLI config based on agent (if not already specified)
        let code_run = self.populate_cli_config_if_needed(code_run);
        let code_run_ref = &*code_run;

        // Determine PVC name based on agent classification and CodeRun type
        let service_name = &code_run_ref.spec.service;
        let classifier = AgentClassifier::new();

        // Check if this is a healer CodeRun (Remediation)
        let template_setting = code_run_ref
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.settings.get("template"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let is_healer = template_setting.starts_with("healer/")
            || service_name.to_lowercase().contains("healer");

        // Get the appropriate PVC name
        let pvc_name = if is_healer {
            // Healer CodeRuns (Remediation agents) share a dedicated PVC
            let healer_pvc = AgentClassifier::get_healer_pvc_name(service_name);
            info!(
                "🏥 Healer CodeRun detected, using dedicated healer PVC: {}",
                healer_pvc
            );
            healer_pvc
        } else if let Some(github_app) = &code_run_ref.spec.github_app {
            match classifier.get_pvc_name(service_name, github_app) {
                Ok(name) => {
                    // Log the agent classification for visibility
                    match classifier.extract_agent_name(github_app) {
                        Ok(agent_name) => {
                            if classifier.is_implementation_agent(&agent_name) {
                                info!("🤝 Agent '{}' identified as implementation agent, using shared workspace", agent_name);
                            } else {
                                info!("🔒 Agent '{}' identified as non-implementation agent, using isolated workspace", agent_name);
                            }
                        }
                        Err(e) => {
                            info!("⚠️ Could not extract agent name: {}", e);
                        }
                    }
                    name
                }
                Err(e) => {
                    // Fallback to default naming if extraction fails
                    error!(
                        "Failed to determine agent-specific PVC name: {}, using default",
                        e
                    );
                    format!("workspace-{service_name}")
                }
            }
        } else {
            // No GitHub App specified, use default naming
            info!("No GitHub App specified, using default PVC naming");
            format!("workspace-{service_name}")
        };

        // Check if fresh workspace is requested (explicit or defaulted for intake)
        if Self::should_use_fresh_workspace(code_run_ref) {
            info!(
                "🧹 Fresh workspace requested for run_type='{}' - will delete existing PVC",
                code_run_ref.spec.run_type
            );
            self.delete_pvc_if_exists(&pvc_name).await?;
        }

        info!("📦 Ensuring PVC exists: {}", pvc_name);
        self.ensure_pvc_exists(
            &pvc_name,
            service_name,
            code_run_ref.spec.github_app.as_deref(),
            code_run_ref.spec.implementation_agent.as_deref(),
        )
        .await?;
        info!("✅ PVC check completed");

        // Don't cleanup resources at start - let idempotent creation handle it
        info!("🔄 Using idempotent resource creation (no aggressive cleanup)");

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = Self::generate_configmap_name(code_run_ref);
        info!("📄 Generated ConfigMap name: {}", cm_name);

        info!("🔧 Creating ConfigMap template data...");
        let configmap = match self.create_configmap(code_run_ref, &cm_name, None) {
            Ok(cm) => {
                info!("✅ ConfigMap template created successfully");
                cm
            }
            Err(e) => {
                // Template rendering failed before Job creation
                // Update CodeRun status with clear error message
                error!(
                    "❌ Template rendering failed for CodeRun {}: {}",
                    code_run_ref.name_any(),
                    e
                );

                // Update status to Failed with detailed message
                let error_msg =
                    format!("Template rendering failed: {e}. Check controller logs for details.");

                if let Err(status_err) = super::status::CodeStatusManager::update_status(
                    &Arc::new(code_run_ref.clone()),
                    self.ctx,
                    "Failed",
                    &error_msg,
                    None,
                )
                .await
                {
                    warn!(
                        "Failed to update CodeRun status after template error: {}",
                        status_err
                    );
                }

                return Err(e);
            }
        };

        // Always create or update ConfigMap to ensure latest template content
        info!("📤 Attempting to create ConfigMap: {}", cm_name);
        match self
            .configmaps
            .create(&PostParams::default(), &configmap)
            .await
        {
            Ok(_) => {
                info!("✅ Created ConfigMap: {}", cm_name);
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                // ConfigMap exists, update it with latest content
                info!(
                    "📝 ConfigMap exists, updating with latest content: {}",
                    cm_name
                );
                match self
                    .configmaps
                    .replace(&cm_name, &PostParams::default(), &configmap)
                    .await
                {
                    Ok(_) => {
                        info!("✅ Updated ConfigMap: {}", cm_name);
                    }
                    Err(e) => {
                        error!("❌ Failed to update ConfigMap {}: {}", cm_name, e);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to create ConfigMap {}: {}", cm_name, e);
                return Err(e.into());
            }
        }

        // Create Job using idempotent creation (now it can successfully mount the existing ConfigMap)
        info!("🚀 Creating job with ConfigMap: {}", cm_name);
        let job_ref = self.create_or_get_job(code_run_ref, &cm_name).await?;
        info!("✅ Job creation completed");

        // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
        if let Some(owner_ref) = job_ref {
            info!("🔗 Updating ConfigMap owner reference");
            self.update_configmap_owner(&code_run, &cm_name, owner_ref)
                .await?;
            info!("✅ ConfigMap owner reference updated");
        } else {
            info!("⚠️ No job owner reference to set");
        }

        info!("🎉 Reconciliation completed successfully for: {}", name);
        Ok(Action::await_change())
    }

    pub async fn cleanup_resources(&self, code_run: &Arc<CodeRun>) -> Result<Action> {
        let name = code_run.name_any();
        info!("Cleaning up code resources for: {}", name);

        // Clean up any remaining jobs and configmaps (but keep PVCs for session continuity)
        self.cleanup_old_jobs(code_run).await?;
        self.cleanup_old_configmaps(code_run).await?;

        // Clean up the workspace subdirectory to prevent disk exhaustion
        self.cleanup_workspace_subdir(code_run).await?;

        Ok(Action::await_change())
    }

    /// Creates a cleanup Job to remove the workspace subdirectory for this CodeRun.
    /// This prevents workspace directories from accumulating on the shared PVC.
    async fn cleanup_workspace_subdir(&self, code_run: &CodeRun) -> Result<()> {
        let coderun_name = code_run.name_any();
        let coderun_uid = code_run.metadata.uid.as_deref().unwrap_or("nouid");
        let workspace_subdir = format!(
            "runs/{}-{}",
            coderun_name,
            &coderun_uid[..coderun_uid.len().min(8)]
        );

        // Determine the PVC name (same logic as in create_resources)
        let classifier = AgentClassifier::new();
        let template_setting = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.settings.get("template"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let is_healer = template_setting.starts_with("healer/")
            || code_run.spec.service.to_lowercase().contains("healer");

        let pvc_name = if is_healer {
            AgentClassifier::get_healer_pvc_name(&code_run.spec.service)
        } else if let Some(github_app) = &code_run.spec.github_app {
            classifier
                .get_pvc_name(&code_run.spec.service, github_app)
                .unwrap_or_else(|_| format!("workspace-{}", code_run.spec.service))
        } else {
            format!("workspace-{}", code_run.spec.service)
        };

        let cleanup_job_name = ResourceNaming::cleanup_job_name(code_run);
        let cleanup_run_label = Self::sanitize_label_value(&coderun_name);
        info!(
            "Creating cleanup job {} to remove /workspace/{}",
            cleanup_job_name, workspace_subdir
        );

        let cleanup_job = json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": cleanup_job_name,
                "namespace": code_run.namespace().as_ref().unwrap_or(&"default".to_string()),
                "labels": {
                    LABEL_CLEANUP_SCOPE: SCOPE_RUN,
                    LABEL_CLEANUP_RUN: cleanup_run_label,
                    LABEL_CLEANUP_KIND: "workspace-cleanup",
                }
            },
            "spec": {
                "ttlSecondsAfterFinished": 300,
                "backoffLimit": 0,
                "template": {
                    "spec": {
                        "restartPolicy": "Never",
                        "containers": [{
                            "name": "cleanup",
                            "image": "busybox:1.36",
                            "command": ["/bin/sh", "-c", format!("rm -rf /workspace/{}", workspace_subdir)],
                            "volumeMounts": [{
                                "name": "workspace",
                                "mountPath": "/workspace"
                            }]
                        }],
                        "volumes": [{
                            "name": "workspace",
                            "persistentVolumeClaim": {
                                "claimName": pvc_name
                            }
                        }]
                    }
                }
            }
        });

        match self
            .jobs
            .create(
                &PostParams::default(),
                &serde_json::from_value(cleanup_job)?,
            )
            .await
        {
            Ok(_) => {
                info!("✅ Cleanup job {} created successfully", cleanup_job_name);
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                info!("Cleanup job {} already exists", cleanup_job_name);
            }
            Err(e) => {
                warn!("Failed to create cleanup job {}: {}", cleanup_job_name, e);
                // Don't fail the entire cleanup if workspace cleanup fails
            }
        }

        Ok(())
    }

    /// Determines if a fresh workspace should be used.
    /// Returns true if:
    /// - `fresh_workspace` is explicitly set to `true`, OR
    /// - `fresh_workspace` is not set AND `run_type` is "intake"
    fn should_use_fresh_workspace(code_run: &CodeRun) -> bool {
        match code_run.spec.fresh_workspace {
            Some(true) => true,
            Some(false) => false,
            None => code_run.spec.run_type == "intake",
        }
    }

    /// Deletes the PVC if it exists, waiting for deletion to complete.
    /// Used when `fresh_workspace` is true to ensure a clean slate.
    async fn delete_pvc_if_exists(&self, pvc_name: &str) -> Result<()> {
        match self.pvcs.get(pvc_name).await {
            Ok(_) => {
                info!(
                    "🗑️ Fresh workspace requested - deleting existing PVC: {}",
                    pvc_name
                );
                match self.pvcs.delete(pvc_name, &DeleteParams::default()).await {
                    Ok(_) => {
                        info!("✅ PVC {} deletion initiated", pvc_name);
                        // Wait for PVC to be fully deleted (up to 30 seconds)
                        for i in 0..30 {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            match self.pvcs.get(pvc_name).await {
                                Err(kube::Error::Api(ae)) if ae.code == 404 => {
                                    info!("✅ PVC {} fully deleted after {}s", pvc_name, i + 1);
                                    return Ok(());
                                }
                                Ok(_) => {
                                    if i % 5 == 0 {
                                        info!(
                                            "⏳ Waiting for PVC {} deletion... ({}s)",
                                            pvc_name, i
                                        );
                                    }
                                }
                                Err(e) => {
                                    error!("Error checking PVC deletion status: {}", e);
                                }
                            }
                        }
                        // If still exists after 30s, log warning but continue
                        info!(
                            "⚠️ PVC {} still exists after 30s, proceeding anyway",
                            pvc_name
                        );
                        Ok(())
                    }
                    Err(kube::Error::Api(ae)) if ae.code == 404 => {
                        info!("PVC {} already deleted", pvc_name);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to delete PVC {}: {}", pvc_name, e);
                        Err(e.into())
                    }
                }
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                info!("PVC {} doesn't exist, nothing to delete", pvc_name);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn ensure_pvc_exists(
        &self,
        pvc_name: &str,
        service_name: &str,
        github_app: Option<&str>,
        implementation_agent: Option<&str>,
    ) -> Result<()> {
        match self.pvcs.get(pvc_name).await {
            Ok(_) => {
                info!("PVC {} already exists", pvc_name);
                Ok(())
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                info!("Creating PVC: {}", pvc_name);
                let pvc =
                    self.build_pvc_spec(pvc_name, service_name, github_app, implementation_agent);
                match self.pvcs.create(&PostParams::default(), &pvc).await {
                    Ok(_) => {
                        info!("Successfully created PVC: {}", pvc_name);
                        Ok(())
                    }
                    Err(kube::Error::Api(ae)) if ae.code == 409 => {
                        info!("PVC {} was created concurrently", pvc_name);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn build_pvc_spec(
        &self,
        pvc_name: &str,
        service_name: &str,
        github_app: Option<&str>,
        implementation_agent: Option<&str>,
    ) -> PersistentVolumeClaim {
        let mut spec = json!({
            "accessModes": ["ReadWriteOnce"],
            "resources": {
                "requests": {
                    "storage": self.config.storage.workspace_size.clone()
                }
            }
        });

        // Add storageClassName if specified in config
        if let Some(ref storage_class) = self.config.storage.storage_class_name {
            spec["storageClassName"] = json!(storage_class);
        }

        // Determine labels based on agent classification
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "orchestrator".to_string());
        labels.insert("component".to_string(), "code-runner".to_string());
        labels.insert("service".to_string(), service_name.to_string());

        // Add agent-specific labels if GitHub App is provided
        if let Some(app) = github_app {
            let classifier = AgentClassifier::new();
            if let Ok(agent_name) = classifier.extract_agent_name(app) {
                labels.insert("agent".to_string(), agent_name.clone());
                // Dual-write: implementation-agent label
                labels.insert(
                    "implementation-agent".to_string(),
                    implementation_agent
                        .filter(|a| !a.is_empty())
                        .map_or_else(|| agent_name.clone(), str::to_lowercase),
                );

                // Add workspace type label
                if classifier.is_implementation_agent(&agent_name) {
                    labels.insert("workspace-type".to_string(), "shared".to_string());
                } else {
                    labels.insert("workspace-type".to_string(), "isolated".to_string());
                }
            }
        }

        let pvc_spec = json!({
            "apiVersion": "v1",
            "kind": "PersistentVolumeClaim",
            "metadata": {
                "name": pvc_name,
                "labels": labels
            },
            "spec": spec
        });

        serde_json::from_value(pvc_spec).expect("Failed to build PVC spec")
    }

    fn generate_configmap_name(code_run: &CodeRun) -> String {
        // Generate unique ConfigMap name per CodeRun to prevent conflicts between sequential jobs
        let namespace = code_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = code_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = code_run
            .metadata
            .uid
            .as_deref()
            .map_or("nouid", |uid| &uid[..8]);
        let task_id = code_run.spec.task_id.unwrap_or(0);
        let service_name = code_run.spec.service.replace('_', "-");
        let context_version = code_run.spec.context_version;

        format!("code-{namespace}-{name}-{uid_suffix}-{service_name}-t{task_id}-v{context_version}-files")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    fn create_configmap(
        &self,
        code_run: &CodeRun,
        name: &str,
        owner_ref: Option<OwnerReference>,
    ) -> Result<ConfigMap> {
        let mut data = BTreeMap::new();

        // Generate all templates for code
        let templates =
            super::templates::CodeTemplateGenerator::generate_all_templates(code_run, self.config)
                .map_err(|e| {
                    // Enhance error message with context for template failures
                    let enhanced_error = match e {
                        crate::tasks::types::Error::ConfigError(msg)
                            if msg.contains("Partial not found")
                                || msg.contains("Failed to load") =>
                        {
                            let partial_name = msg
                                .split("Partial not found")
                                .nth(1)
                                .or_else(|| msg.split("Failed to load").nth(1))
                                .and_then(|s| s.split_whitespace().next())
                                .unwrap_or("unknown");

                            crate::tasks::types::Error::ConfigError(format!(
                                "Template rendering failed for CodeRun {}: {}. \
                        This typically indicates missing template files in the controller image. \
                        Expected template path: /app/templates/_shared/partials/{}.hbs. \
                        Check controller logs at startup for template verification warnings.",
                                code_run.name_any(),
                                msg,
                                partial_name
                            ))
                        }
                        other => other,
                    };
                    error!(
                        coderun = %code_run.name_any(),
                        github_app = ?code_run.spec.github_app,
                        "Template generation failed: {}",
                        enhanced_error
                    );
                    enhanced_error
                })?;

        for (filename, content) in templates {
            data.insert(filename, content);
        }

        let labels = Self::create_task_labels(code_run);
        let mut metadata = ObjectMeta {
            name: Some(name.to_string()),
            labels: Some(labels),
            ..Default::default()
        };

        if let Some(owner) = owner_ref {
            metadata.owner_references = Some(vec![owner]);
        }

        Ok(ConfigMap {
            metadata,
            data: Some(data),
            ..Default::default()
        })
    }

    /// Idempotent job creation: create if doesn't exist, get if it does
    async fn create_or_get_job(
        &self,
        code_run: &CodeRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = Self::generate_job_name(code_run);

        // Try to get existing job first (idempotent check)
        if let Ok(existing_job) = self.jobs.get(&job_name).await {
            info!("Found existing job: {}, checking for active pods", job_name);

            // Check if there are any pods for this job (regardless of controller UID)
            // This prevents duplicate pods when controller restarts
            let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(
                self.ctx.client.clone(),
                code_run.metadata.namespace.as_deref().unwrap_or("default"),
            );

            let pod_list = pods
                .list(&ListParams::default().labels(&format!("job-name={job_name}")))
                .await?;

            if pod_list.items.is_empty() {
                info!(
                    "Job {} exists but has no pods, will let Job controller handle it",
                    job_name
                );
            } else {
                info!(
                    "Found {} existing pod(s) for job {}, using existing job",
                    pod_list.items.len(),
                    job_name
                );
            }

            // Return the existing job's owner reference
            Ok(Some(OwnerReference {
                api_version: "batch/v1".to_string(),
                kind: "Job".to_string(),
                name: job_name,
                uid: existing_job.metadata.uid.unwrap_or_default(),
                controller: Some(false),
                block_owner_deletion: Some(true),
            }))
        } else {
            // Job doesn't exist, create it
            info!("Job {} doesn't exist, creating it", job_name);
            self.create_job(code_run, cm_name).await
        }
    }

    async fn create_job(
        &self,
        code_run: &CodeRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = Self::generate_job_name(code_run);
        let job = self.build_job_spec(code_run, &job_name, cm_name)?;

        match self.jobs.create(&PostParams::default(), &job).await {
            Ok(created_job) => {
                info!("Created code job: {}", job_name);
                // Update status
                super::status::CodeStatusManager::update_job_started(
                    &Arc::new(code_run.clone()),
                    self.ctx,
                    &job_name,
                    cm_name,
                )
                .await?;

                // Return owner reference for the created job
                if let (Some(uid), Some(name)) =
                    (created_job.metadata.uid, created_job.metadata.name)
                {
                    Ok(Some(OwnerReference {
                        api_version: "batch/v1".to_string(),
                        kind: "Job".to_string(),
                        name,
                        uid,
                        controller: Some(true),
                        block_owner_deletion: Some(true),
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                info!("Job already exists: {}", job_name);
                // Try to get existing job for owner reference
                match self.jobs.get(&job_name).await {
                    Ok(existing_job) => {
                        if let (Some(uid), Some(name)) =
                            (existing_job.metadata.uid, existing_job.metadata.name)
                        {
                            Ok(Some(OwnerReference {
                                api_version: "batch/v1".to_string(),
                                kind: "Job".to_string(),
                                name,
                                uid,
                                controller: Some(true),
                                block_owner_deletion: Some(true),
                            }))
                        } else {
                            Ok(None)
                        }
                    }
                    Err(_) => Ok(None),
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn generate_job_name(code_run: &CodeRun) -> String {
        // Use unified naming system to ensure consistency with controller lookups
        ResourceNaming::job_name(code_run)
    }

    #[allow(clippy::too_many_lines)] // Complex function not easily split
    fn build_job_spec(&self, code_run: &CodeRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = Self::create_task_labels(code_run);

        // Create owner reference to CodeRun for proper event handling
        let owner_ref = OwnerReference {
            api_version: "agents.platform/v1".to_string(),
            kind: "CodeRun".to_string(),
            name: code_run.name_any(),
            uid: code_run.metadata.uid.clone().unwrap_or_default(),
            controller: Some(true),
            block_owner_deletion: Some(true),
        };

        // Build volumes for code (PVC for persistence)
        let mut volumes = vec![];
        let mut volume_mounts = vec![];

        // ConfigMap volume (always needed)
        volumes.push(json!({
            "name": "task-files",
            "configMap": {
                "name": cm_name
            }
        }));
        volume_mounts.push(json!({
            "name": "task-files",
            "mountPath": "/task-files"
        }));

        // Get ConfigMap prefix from environment (set by Helm based on release name)
        // Used for all ConfigMap references below
        let cm_prefix =
            std::env::var("CONFIGMAP_PREFIX").unwrap_or_else(|_| "controller".to_string());

        // Agents ConfigMap volume for system prompts
        let agents_cm_name = format!("{cm_prefix}-agents");
        volumes.push(json!({
            "name": "agents-config",
            "configMap": {
                "name": agents_cm_name
            }
        }));
        volume_mounts.push(json!({
            "name": "agents-config",
            "mountPath": "/config/agents"
        }));

        // Agent templates: Use projected volume to merge shared + integration ConfigMaps
        // This allows Atlas/Bolt to access integration templates from /templates
        let shared_templates_cm_name = format!("{cm_prefix}-templates-shared");
        let integration_templates_cm_name = format!("{cm_prefix}-templates-integration");
        let healer_templates_cm_name = format!("{cm_prefix}-templates-healer");

        // Check if this is a Healer workflow (service contains "healer" or template starts with "healer/")
        let is_healer_workflow = code_run.spec.service.to_lowercase().contains("healer")
            || code_run
                .spec
                .cli_config
                .as_ref()
                .and_then(|c| c.settings.get("template"))
                .and_then(|v| v.as_str())
                .is_some_and(|t| t.starts_with("healer/"));

        // Build projected volume sources - always include shared and integration
        let mut projected_sources = vec![
            json!({
                "configMap": {
                    "name": shared_templates_cm_name
                }
            }),
            json!({
                "configMap": {
                    "name": integration_templates_cm_name
                }
            }),
        ];

        // Add healer templates for healer workflows
        if is_healer_workflow {
            projected_sources.push(json!({
                "configMap": {
                    "name": healer_templates_cm_name
                }
            }));
        }

        volumes.push(json!({
            "name": "templates-shared",
            "projected": {
                "sources": projected_sources
            }
        }));
        volume_mounts.push(json!({
            "name": "templates-shared",
            "mountPath": "/templates"
        }));

        // Integration templates ConfigMap volume for Atlas/Bolt guardian scripts
        // Note: cm_prefix already defined above
        let integration_templates_cm_name = format!("{cm_prefix}-templates-integration");
        volumes.push(json!({
            "name": "templates-integration",
            "configMap": {
                "name": integration_templates_cm_name
            }
        }));
        volume_mounts.push(json!({
            "name": "templates-integration",
            "mountPath": "/templates-integration"
        }));

        // Blaze agent scripts ConfigMap volume for frontend workflows
        // Only mount for Blaze agent to avoid unnecessary volumes on other agents
        let is_blaze_agent = code_run
            .spec
            .github_app
            .as_ref()
            .is_some_and(|app| app.to_lowercase().contains("blaze"));

        if is_blaze_agent {
            let blaze_scripts_cm_name = format!("{cm_prefix}-agent-scripts-blaze");
            volumes.push(json!({
                "name": "blaze-scripts",
                "configMap": {
                    "name": blaze_scripts_cm_name,
                    "defaultMode": 0o755,
                    "optional": true
                }
            }));
            volume_mounts.push(json!({
                "name": "blaze-scripts",
                "mountPath": "/workspace/scripts/blaze"
            }));
        }

        // Intake ConfigMap volume for intake workflows
        // When run_type is "intake" and INTAKE_CONFIGMAP env is set, mount the intake files
        // This ConfigMap contains the PRD, architecture, and config files from the PM server
        if code_run.spec.run_type == "intake" {
            if let Some(intake_cm_name) = code_run.spec.env.get("INTAKE_CONFIGMAP") {
                if !intake_cm_name.is_empty() {
                    info!(
                        "📁 Mounting intake ConfigMap for intake workflow: {}",
                        intake_cm_name
                    );
                    volumes.push(json!({
                        "name": "intake-files",
                        "configMap": {
                            "name": intake_cm_name
                        }
                    }));
                    volume_mounts.push(json!({
                        "name": "intake-files",
                        "mountPath": "/intake-files"
                    }));
                }
            }
        }

        // Project-specific CTO config from Linear
        // When LINEAR_PROJECT_ID is set, mount the project's cto-config ConfigMap
        // This ConfigMap is synced from Linear documents by the PM server
        if let Some(project_id) = code_run.spec.env.get("LINEAR_PROJECT_ID") {
            if !project_id.is_empty() {
                let project_config_cm_name =
                    format!("cto-config-project-{}", project_id.to_lowercase());
                info!(
                    "📁 Mounting project-specific CTO config: {} (project: {})",
                    project_config_cm_name, project_id
                );
                volumes.push(json!({
                    "name": "project-config",
                    "configMap": {
                        "name": project_config_cm_name,
                        "optional": true  // Don't fail if ConfigMap doesn't exist yet
                    }
                }));
                volume_mounts.push(json!({
                    "name": "project-config",
                    "mountPath": "/config/project"
                }));
            }
        }

        let cli_type = Self::code_run_cli_type(code_run);

        if cli_type == CLIType::Claude {
            volume_mounts.push(json!({
                "name": "task-files",
                "mountPath": "/etc/claude-code/managed-settings.json",
                "subPath": "settings.json"
            }));
        }

        // CLI-specific config file mounts from ConfigMap
        // These are rendered from templates/cli-configs/ and added to the ConfigMap
        if cli_type == CLIType::Copilot {
            volume_mounts.push(json!({
                "name": "task-files",
                "mountPath": "/home/node/.copilot/config.json",
                "subPath": "copilot-config.json"
            }));
        }
        // Kimi config + OAuth token written by harness at startup
        // (can't use subPath mount — blocks credentials dir creation)
        if cli_type == CLIType::OpenCode {
            volume_mounts.push(json!({
                "name": "task-files",
                "mountPath": "/home/node/.config/opencode/opencode.json",
                "subPath": "opencode.json"
            }));
        }
        if cli_type == CLIType::Factory {
            volume_mounts.push(json!({
                "name": "task-files",
                "mountPath": "/home/node/.factory/config.json",
                "subPath": "factory-config.json"
            }));
        }

        // PVC workspace volume for code (persistent across sessions)
        // Use conditional naming based on CodeRun type and agent classification
        let classifier = AgentClassifier::new();

        // Check if this is a healer CodeRun
        let template_setting = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.settings.get("template"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let is_healer = template_setting.starts_with("healer/")
            || code_run.spec.service.to_lowercase().contains("healer");

        let pvc_name = if is_healer {
            // Healer CodeRuns share a dedicated PVC
            AgentClassifier::get_healer_pvc_name(&code_run.spec.service)
        } else if let Some(github_app) = &code_run.spec.github_app {
            match classifier.get_pvc_name(&code_run.spec.service, github_app) {
                Ok(name) => name,
                Err(e) => {
                    error!("Failed to determine agent-specific PVC name for volume mount: {}, using default", e);
                    format!("workspace-{}", code_run.spec.service)
                }
            }
        } else {
            format!("workspace-{}", code_run.spec.service)
        };

        // Generate unique workspace subdirectory per CodeRun to prevent git lock conflicts
        // when multiple pods share the same PVC
        let coderun_name = code_run.name_any();
        let coderun_uid = code_run.metadata.uid.as_deref().unwrap_or("nouid");
        let workspace_subdir = format!(
            "runs/{}-{}",
            coderun_name,
            &coderun_uid[..coderun_uid.len().min(8)]
        );

        volumes.push(json!({
            "name": "workspace",
            "persistentVolumeClaim": {
                "claimName": pvc_name
            }
        }));
        volume_mounts.push(json!({
            "name": "workspace",
            "mountPath": "/workspace"
        }));

        // Mount shared GitHub App key volume (init container writes, main container reads)
        volume_mounts.push(json!({
            "name": "github-app-key",
            "mountPath": "/var/run/secrets/github-app-key"
        }));

        // Docker-in-Docker volumes (enabled by default, can be disabled via enableDocker: false)
        let enable_docker = code_run.spec.enable_docker;
        if enable_docker {
            volumes.push(json!({
                "name": "docker-sock-dir",
                "emptyDir": {}
            }));
            volume_mounts.push(json!({
                "name": "docker-sock-dir",
                "mountPath": "/var/run"
            }));

            // Docker data volume for DinD daemon
            volumes.push(json!({
                "name": "docker-data",
                "emptyDir": {}
            }));
        }

        // Watcher coordination ConfigMap volume
        // When this is a watcher CodeRun, mount the coordination ConfigMap so the watcher
        // can read the initial coordination state. An init container copies it to the workspace.
        if let Some(executor_name) = &code_run.spec.watcher_for {
            let coordination_cm_name = coordination_configmap_name(executor_name);
            info!(
                "🔗 Watcher CodeRun detected for executor {}, mounting coordination ConfigMap: {}",
                executor_name, coordination_cm_name
            );
            volumes.push(json!({
                "name": "coordination-config",
                "configMap": {
                    "name": coordination_cm_name
                }
            }));
            volume_mounts.push(json!({
                "name": "coordination-config",
                "mountPath": "/config/coordination"
            }));
        }

        // CLI OAuth credential mounts (Phase B1/B2)
        // Claude OAuth: copied by openclaw.sh.hbs into ~/.claude/.credentials.json at startup
        if cli_type == CLIType::Claude {
            volumes.push(json!({
                "name": "claude-oauth",
                "secret": {
                    "secretName": "claude-oauth",
                    "optional": true
                }
            }));
            volume_mounts.push(json!({
                "name": "claude-oauth",
                "mountPath": "/root/.claude-oauth",
                "readOnly": true
            }));
        }

        // Shared volume for GitHub App private key - shared between init and main containers
        // This allows the init container to write the key and main container (Ruby/Go) to read it
        volumes.push(json!({
            "name": "github-app-key",
            "emptyDir": {}
        }));

        // Shared emptyDir for OpenClaw peer deps — init container installs, main container reads.
        // Must be added before container_spec is built (which consumes volume_mounts).
        // Skipped for Hermes pods which don't use OpenClaw.
        let openclaw_nm_path = "/usr/local/share/npm-global/lib/node_modules/openclaw/node_modules";
        if !matches!(code_run.spec.effective_harness(), HarnessAgent::Hermes) {
            volumes.push(json!({
                "name": "openclaw-node-modules",
                "emptyDir": {}
            }));
            volume_mounts.push(json!({
                "name": "openclaw-node-modules",
                "mountPath": openclaw_nm_path
            }));
        }

        // GitHub App authentication only - no SSH volumes needed
        // Validate github_app is present and non-empty
        let github_app = code_run
            .spec
            .github_app
            .as_ref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                tracing::error!(
                    "GitHub App is required for CodeRun authentication and cannot be empty"
                );
                crate::tasks::types::Error::ConfigError(
                    "GitHub App is required for CodeRun authentication and cannot be empty"
                        .to_string(),
                )
            })?;

        tracing::info!(
            "Using GitHub App authentication for CodeRun: {}",
            github_app
        );

        // Select image based on CLI type (if specified) or fallback to default
        let image = self.select_image_for_cli(code_run)?;
        let cli_type_str = cli_type.to_string();
        let cli_model = code_run.spec.model.clone();
        let container_name = Self::container_name_for_cli(cli_type, &cli_model);

        // ── Single source of truth for provider resolution ──
        let provider_config = EffectiveProviderConfig::resolve(code_run, self.config);

        // Resolve CLI-specific API key binding (env var + secret reference)
        let legacy_provider = self
            .config
            .agent
            .cli_providers
            .get(&cli_type.to_string().to_lowercase())
            .map(std::string::String::as_str);

        let api_key_binding = self
            .config
            .secrets
            .resolve_cli_binding(&cli_type, legacy_provider);
        let ResolvedSecretBinding {
            env_var: api_env_var,
            secret_name: api_secret_name,
            secret_key: api_secret_key,
        } = api_key_binding;

        // Build environment variables for code tasks
        // Note: Critical system vars (CODERUN_NAME, WORKFLOW_NAME, NAMESPACE) are added
        // AFTER requirements processing to prevent overrides
        let env_vars = vec![
            json!({
                "name": "GITHUB_APP_ID",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": github_app_secret_name(github_app),
                        "key": "app-id"
                    }
                }
            }),
            json!({
                "name": "GITHUB_APP_PRIVATE_KEY",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": github_app_secret_name(github_app),
                        "key": "private-key"
                    }
                }
            }),
            json!({
                "name": "GITHUB_APP_INSTALLATION_ID",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": github_app_secret_name(github_app),
                        "key": "installation-id",
                        "optional": true
                    }
                }
            }),
            json!({
                "name": api_env_var,
                "valueFrom": {
                    "secretKeyRef": {
                        "name": api_secret_name,
                        "key": api_secret_key
                    }
                }
            }),
        ];

        // Process task requirements if present
        let (mut final_env_vars, mut env_from) =
            Self::process_task_requirements(code_run, env_vars)?;

        // Critical system variables that must not be overridden
        // Add these AFTER requirements processing to ensure they take precedence
        let mut critical_env_vars = vec![
            json!({
                "name": "CODERUN_NAME",
                "value": code_run.name_any()
            }),
            json!({
                "name": "WORKFLOW_NAME",
                "value": code_run.metadata.labels.as_ref()
                    .and_then(|labels| labels.get("workflow-name"))
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string())
            }),
            json!({
                "name": "NAMESPACE",
                "valueFrom": {
                    "fieldRef": {
                        "fieldPath": "metadata.namespace"
                    }
                }
            }),
            json!({
                "name": "CLI_TYPE",
                "value": cli_type_str
            }),
            json!({
                "name": "CLI_MODEL",
                "value": cli_model
            }),
            json!({
                "name": "CLI_CONTAINER_NAME",
                "value": container_name.clone()
            }),
            json!({
                "name": "MCP_CLIENT_CONFIG",
                "value": "/workspace/client-config.json"
            }),
            json!({
                "name": "PROGRESS_FILE",
                "value": "/workspace/progress.jsonl"
            }),
            // WORKSPACE_DIR provides isolation for concurrent CodeRuns sharing the same PVC
            // Each CodeRun gets a unique subdirectory: /workspace/runs/{coderun-name}-{uid}/
            json!({
                "name": "WORKSPACE_DIR",
                "value": format!("/workspace/{}", workspace_subdir)
            }),
            json!({
                "name": "TASK_ID",
                "value": code_run.spec.task_id.map_or("0".to_string(), |id| id.to_string())
            }),
            json!({
                "name": "PROJECT_ID",
                "value": code_run.spec.project_id.clone().unwrap_or_default()
            }),
        ];

        // ── Provider × CLI env vars (replaces scattered if-blocks) ──
        critical_env_vars.extend(provider_config.build_env_vars());
        // Export resolved provider as env var for templates/observability
        critical_env_vars.push(json!({
            "name": "RESOLVED_PROVIDER",
            "value": provider_config.provider.to_string()
        }));
        critical_env_vars.push(json!({
            "name": "RESOLVED_PROVIDER_SOURCE",
            "value": provider_config.source.to_string()
        }));

        // Comprehensive deduplication: remove all duplicates by name, keeping the last occurrence
        // This ensures that later additions (like critical system vars) take precedence
        let mut seen_names = std::collections::HashSet::new();
        let mut deduplicated_env_vars = Vec::new();

        // Process in reverse order to keep the last occurrence of each variable
        for env_var in final_env_vars.into_iter().rev() {
            if let Some(name) = env_var.get("name").and_then(|n| n.as_str()) {
                if !seen_names.contains(name) {
                    seen_names.insert(name.to_string());
                    deduplicated_env_vars.push(env_var);
                }
            } else {
                // Keep env vars without names (shouldn't happen, but be safe)
                deduplicated_env_vars.push(env_var);
            }
        }

        // Reverse back to original order
        deduplicated_env_vars.reverse();
        final_env_vars = deduplicated_env_vars;

        // Add critical system vars (these will override any duplicates due to deduplication logic)
        final_env_vars.extend(critical_env_vars);

        // Add Docker environment variable if Docker is enabled
        // Socket is at /var/run/docker/docker.sock to avoid overwriting /var/run/secrets
        if enable_docker {
            final_env_vars.push(json!({
                "name": "DOCKER_HOST",
                "value": "unix:///var/run/docker/docker.sock"
            }));
        }

        // OpenClaw requires a valid TZ — containers default to Etc/Unknown which crashes the logger
        final_env_vars.push(json!({ "name": "TZ", "value": "UTC" }));

        // Disable xAI code_execution tool (cto-secrets has a placeholder XAI_API_KEY)
        final_env_vars.push(json!({ "name": "XAI_API_KEY", "value": "" }));

        // Pass ACP configuration as JSON env var (first entry = primary for now)
        if let Some(acp) = &code_run.spec.acp {
            if let Ok(acp_json) = serde_json::to_string(acp) {
                final_env_vars.push(json!({ "name": "CTO_ACP_CONFIG", "value": acp_json }));
            }
        }

        // Pass OpenClaw configuration as JSON env var
        if let Some(openclaw) = &code_run.spec.openclaw {
            if let Ok(oc_json) = serde_json::to_string(openclaw) {
                final_env_vars.push(json!({ "name": "CTO_OPENCLAW_CONFIG", "value": oc_json }));
            }
        }

        // HARNESS_AGENT: runtime detection of which harness is running
        let harness_name = match code_run.spec.effective_harness() {
            HarnessAgent::OpenClaw => "openclaw",
            HarnessAgent::Hermes => "hermes",
        };
        final_env_vars.push(json!({ "name": "HARNESS_AGENT", "value": harness_name }));

        // Hermes-specific env vars
        if matches!(code_run.spec.effective_harness(), HarnessAgent::Hermes) {
            final_env_vars.push(json!({ "name": "NOUS_BASE_URL", "value": "https://inference-api.nousresearch.com/v1" }));
            final_env_vars
                .push(json!({ "name": "NOUS_MODEL", "value": "nousresearch/hermes-4-70b" }));
            // NOUS_API_KEY comes from cto-secrets via envFrom, not hardcoded here
        }

        // Provider env vars are now handled by EffectiveProviderConfig.build_env_vars()
        // above (in the critical_env_vars section). No more model-string detection here.

        // Final deduplication pass to handle any remaining duplicates from critical vars and Docker
        let mut final_seen_names = std::collections::HashSet::new();
        let mut final_deduplicated_env_vars = Vec::new();

        // Process in reverse order to keep the last occurrence (critical vars take precedence)
        for env_var in final_env_vars.into_iter().rev() {
            if let Some(name) = env_var.get("name").and_then(|n| n.as_str()) {
                if !final_seen_names.contains(name) {
                    final_seen_names.insert(name.to_string());
                    final_deduplicated_env_vars.push(env_var);
                }
            } else {
                final_deduplicated_env_vars.push(env_var);
            }
        }

        // Reverse back to original order
        final_deduplicated_env_vars.reverse();
        final_env_vars = final_deduplicated_env_vars;

        // Use Always pull policy for :latest and :dev tags to ensure fresh images,
        // unless explicitly overridden in config
        let image_pull_policy = self.resolve_image_pull_policy(&image);

        let mut container_spec = json!({
            "name": container_name,
            "image": image,
            "imagePullPolicy": image_pull_policy,
            "env": final_env_vars,
            "command": ["/bin/bash"],
            "args": ["/task-files/container.sh"],
            "workingDir": format!("/workspace/{}", workspace_subdir),
            "volumeMounts": volume_mounts
        });

        if enable_docker {
            // NOTE: Do NOT run the main container as root (runAsUser: 0).
            // Claude CLI refuses --dangerously-skip-permissions when running as root.
            // The Docker daemon sidecar runs as root separately; the main container
            // only needs KILL capability to signal it, not root privileges.
            container_spec["securityContext"] = json!({
                "allowPrivilegeEscalation": false,
                "capabilities": {
                    "add": ["KILL"]
                }
            });
        }

        // Mount cto-secrets for API keys (includes Context7, Anthropic, OpenAI, etc.)
        env_from.push(json!({
            "secretRef": {
                "name": "cto-secrets"
            }
        }));

        // Mount intake-api-keys for Tavily and Gemini API keys
        // This secret is NOT ArgoCD-managed (won't be reverted by Helm syncs)
        env_from.push(json!({
            "secretRef": {
                "name": "intake-api-keys",
                "optional": serde_json::Value::Bool(true)
            }
        }));

        // Mount Discord secrets only for OpenClaw harness (Hermes doesn't use Discord)
        if !matches!(code_run.spec.effective_harness(), HarnessAgent::Hermes) {
            // Mount discord-pm-bot for DISCORD_PM_BOT_TOKEN (live debate channel posting)
            env_from.push(json!({
                "secretRef": {
                    "name": "discord-pm-bot",
                    "optional": serde_json::Value::Bool(true)
                }
            }));

            // Mount discord-agent-bots for per-agent Discord tokens and channels
            // (DISCORD_TOKEN_<AGENT>, DISCORD_CHANNEL_<AGENT>)
            env_from.push(json!({
                "secretRef": {
                    "name": "discord-agent-bots",
                    "optional": serde_json::Value::Bool(true)
                }
            }));
        }

        // Mount solana-api-keys for Helius RPC, Birdeye, and Solana env vars
        // (HELIUS_API_KEY, SOLANA_RPC_URL, SOLANA_DEVNET_RPC_URL, BIRDEYE_API_KEY)
        env_from.push(json!({
            "secretRef": {
                "name": "solana-api-keys",
                "optional": serde_json::Value::Bool(true)
            }
        }));

        // Add envFrom if we have secrets to mount
        if !env_from.is_empty() {
            container_spec["envFrom"] = json!(env_from);
        }

        // Build containers array
        let mut containers = vec![container_spec];
        #[allow(unused_assignments)]
        let mut promtail_init_config: Option<String> = None;

        // Add Docker daemon if enabled (kept as-is for DIND workflows)
        if enable_docker {
            let docker_daemon_spec = json!({
                "name": "docker-daemon",
                "image": "docker:dind",
                "command": ["/bin/sh", "-c"],
                "args": [
                    // Watch for agent completion signal at /workspace/.agent_done
                    // This matches the completion.sh.hbs template which writes to this path
                    "dockerd-entrypoint.sh & DOCKER_PID=$!; \
                     while true; do \
                       if [ -f /workspace/.agent_done ]; then \
                         echo 'Agent done signal detected, stopping docker daemon...'; \
                         kill -TERM $DOCKER_PID 2>/dev/null || true; \
                         sleep 2; \
                         kill -KILL $DOCKER_PID 2>/dev/null || true; \
                         exit 0; \
                       fi; \
                       if ! kill -0 $DOCKER_PID 2>/dev/null; then \
                         echo 'Docker daemon exited unexpectedly'; \
                         exit 1; \
                       fi; \
                       sleep 5; \
                     done".to_string()
                ],
                "securityContext": {
                    "privileged": true,
                    "allowPrivilegeEscalation": true,
                    "runAsUser": 0,
                    "runAsGroup": 0,
                    "runAsNonRoot": false
                },
                "env": [
                    {
                        "name": "DOCKER_TLS_CERTDIR",
                        "value": ""
                    }
                ],
                "volumeMounts": [
                    {
                        "name": "docker-sock-dir",
                        "mountPath": "/var/run"
                    },
                    {
                        "name": "docker-data",
                        "mountPath": "/var/lib/docker"
                    },
                    {
                        "name": "workspace",
                        "mountPath": "/workspace"
                    }
                ],
                "lifecycle": {
                    "preStop": {
                        "exec": {
                            "command": [
                                "/bin/sh",
                                "-c",
                                "pkill -TERM dockerd; sleep 5; pkill -KILL dockerd || killall -9 dockerd || kill -9 $(pidof dockerd) || true"
                            ]
                        }
                    }
                },
                "resources": {
                    "requests": {
                        "cpu": "100m",
                        "memory": "128Mi"
                    },
                    "limits": {
                        "cpu": "500m",
                        "memory": "512Mi"
                    }
                }
            });
            containers.push(docker_daemon_spec);
        }

        // Add Linear sidecar if enabled (status sync + log streaming + 2-way comms + whip cracking)
        if let Some(linear) = &code_run.spec.linear_integration {
            if linear.enabled {
                let session_id = linear.session_id.clone().unwrap_or_default();
                let issue_id = linear.issue_id.clone().unwrap_or_default();
                let team_id = linear.team_id.clone().unwrap_or_default();
                let workflow_name = format!("coderun-{}", code_run.name_any());

                // Add shared volume for status file
                volumes.push(json!({
                    "name": "linear-status",
                    "emptyDir": {}
                }));

                // Add volume mount and env vars to main container for status sync
                if let Some(main_container) = containers.first_mut() {
                    // Ensure volumeMounts array exists and add linear-status mount
                    let mounts = main_container
                        .as_object_mut()
                        .and_then(|obj| {
                            if !obj.contains_key("volumeMounts") {
                                obj.insert("volumeMounts".to_string(), json!([]));
                            }
                            obj.get_mut("volumeMounts")
                        })
                        .and_then(|v| v.as_array_mut());

                    if let Some(mounts_arr) = mounts {
                        mounts_arr.push(json!({
                            "name": "linear-status",
                            "mountPath": "/status"
                        }));
                    } else {
                        warn!("Failed to add linear-status volume mount to main container");
                    }

                    // Ensure env array exists and add status/log env vars
                    let env = main_container
                        .as_object_mut()
                        .and_then(|obj| {
                            if !obj.contains_key("env") {
                                obj.insert("env".to_string(), json!([]));
                            }
                            obj.get_mut("env")
                        })
                        .and_then(|v| v.as_array_mut());

                    if let Some(env_arr) = env {
                        env_arr.push(json!({
                            "name": "STATUS_FILE",
                            "value": "/status/current.json"
                        }));
                        // Agent should write logs to file for sidecar to stream
                        env_arr.push(json!({
                            "name": "LOG_FILE_PATH",
                            "value": "/workspace/agent.log"
                        }));
                        // Enable info-level logging so progress is visible (and streamed to Linear)
                        env_arr.push(json!({
                            "name": "RUST_LOG",
                            "value": "info"
                        }));
                    } else {
                        warn!("Failed to add STATUS_FILE/LOG_FILE_PATH/RUST_LOG env vars to main container");
                    }
                } else {
                    warn!("No main container found to configure for Linear integration");
                }

                // Build sidecar environment variables
                let mut sidecar_env = vec![
                    json!({ "name": "STATUS_FILE", "value": "/status/current.json" }),
                    json!({ "name": "LINEAR_SERVICE_URL", "value": self.config.linear.service_url }),
                    json!({ "name": "STATUS_POLL_INTERVAL_MS", "value": "5000" }),
                    json!({ "name": "LOG_POST_INTERVAL_MS", "value": "5000" }),
                    json!({ "name": "INPUT_POLL_INTERVAL_MS", "value": "2000" }),
                    json!({ "name": "LINEAR_SESSION_ID", "value": session_id.clone() }),
                    json!({ "name": "LINEAR_ISSUE_ID", "value": issue_id }),
                    json!({ "name": "LINEAR_TEAM_ID", "value": team_id }),
                    json!({ "name": "WORKFLOW_NAME", "value": workflow_name }),
                    json!({ "name": "RUST_LOG", "value": "info" }),
                    // New: Log streaming and input polling paths
                    json!({ "name": "LOG_FILE_PATH", "value": "/workspace/agent.log" }),
                    json!({ "name": "PROGRESS_FILE", "value": "/workspace/progress.jsonl" }),
                    json!({ "name": "INPUT_FIFO_PATH", "value": "/workspace/agent-input.jsonl" }),
                    json!({ "name": "HTTP_PORT", "value": "8080" }),
                    // Whip cracking - progress monitoring with escalating nudges
                    json!({ "name": "WHIP_CRACK_ENABLED", "value": "true" }),
                    json!({ "name": "STALL_THRESHOLD_SECS", "value": "120" }),
                    json!({ "name": "NUDGE_INTERVAL_SECS", "value": "60" }),
                    json!({ "name": "MAX_NUDGE_LEVEL", "value": "3" }),
                ];

                // Add LINEAR_OAUTH_TOKEN - prefer access_token from CodeRun spec (from webhook),
                // fall back to agent-specific secret
                if let Some(access_token) = linear.access_token.as_ref() {
                    info!("Using OAuth access token from CodeRun spec (webhook payload)");
                    sidecar_env.push(json!({
                        "name": "LINEAR_OAUTH_TOKEN",
                        "value": access_token
                    }));
                } else {
                    // Fall back to agent-specific secret
                    // Extract agent name from github_app (e.g., "5DLabs-Rex" -> "rex", "cto-dev" -> "morgan")
                    let agent_name = code_run
                        .spec
                        .github_app
                        .as_deref()
                        .and_then(|app| {
                            app.strip_prefix("5DLabs-")
                                .or_else(|| app.strip_prefix("5dlabs-"))
                        })
                        .map_or_else(|| "morgan".to_string(), str::to_lowercase);
                    let agent_secret_name = format!("linear-app-{agent_name}");

                    info!(
                        "No access token in CodeRun spec, falling back to secret {}",
                        agent_secret_name
                    );
                    sidecar_env.push(json!({
                        "name": "LINEAR_OAUTH_TOKEN",
                        "valueFrom": {
                            "secretKeyRef": {
                                "name": agent_secret_name.clone(),
                                "key": "access_token",
                                "optional": true
                            }
                        }
                    }));
                }
                // Also add LINEAR_API_KEY as fallback (status-sync uses either)
                sidecar_env.push(json!({
                    "name": "LINEAR_API_KEY",
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": "linear-secrets",
                            "key": "LINEAR_API_KEY",
                            "optional": true
                        }
                    }
                }));

                let sidecar_image = self.config.linear.sidecar_image.clone().unwrap_or_else(|| {
                    "registry.5dlabs.ai/5dlabs/linear-sidecar:latest".to_string()
                });
                let sidecar_pull_policy =
                    if sidecar_image.ends_with(":latest") || sidecar_image.ends_with(":dev") {
                        "Always"
                    } else {
                        "IfNotPresent"
                    };
                let sidecar_spec = json!({
                    "name": "linear-sidecar",
                    "image": sidecar_image,
                    "imagePullPolicy": sidecar_pull_policy,
                    "env": sidecar_env,
                    "volumeMounts": [
                        { "name": "linear-status", "mountPath": "/status" },
                        { "name": "workspace", "mountPath": "/workspace" }
                    ],
                    "ports": [
                        { "containerPort": 8080, "name": "http" }
                    ],
                    "resources": {
                        "requests": { "cpu": "10m", "memory": "32Mi" },
                        "limits": { "cpu": "100m", "memory": "64Mi" }
                    }
                });
                containers.push(sidecar_spec);
                info!("Added Linear sidecar for session {} (status sync + log streaming + 2-way comms + whip cracking)", session_id);
            }
        }

        // Add promtail sidecar for shipping CLI logs to Loki
        // Mirrors Morgan's promtail config — scrapes CLI-specific log paths
        // and ships them to the in-cluster Loki gateway via the OTLP pipeline
        {
            let agent_name = code_run
                .spec
                .github_app
                .as_deref()
                .unwrap_or("unknown")
                .replace("5DLabs-", "")
                .to_lowercase();

            let loki_url = std::env::var("LOKI_PUSH_URL").unwrap_or_else(|_| {
                "http://openclaw-observability-loki-gateway.openclaw.svc.cluster.local/loki/api/v1/push".to_string()
            });

            let workspace_path = format!("/workspace/{workspace_subdir}");

            let promtail_config = format!(
                r#"server:
  http_listen_port: 0
  grpc_listen_port: 0
positions:
  filename: /run/promtail/positions.yaml
clients:
  - url: {loki_url}
    tenant_id: openclaw
scrape_configs:
  - job_name: acp-cli
    static_configs:
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-claude
          cli_name: claude
          __path__: {workspace_path}/.claude/**/*.{{jsonl,log}}
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-droid
          cli_name: droid
          __path__: {workspace_path}/.factory/**/*.{{jsonl,log}}
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-codex
          cli_name: codex
          __path__: {workspace_path}/.codex/**/*.{{jsonl,log}}
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-gemini
          cli_name: gemini
          __path__: {workspace_path}/.gemini/**/*.{{jsonl,log}}
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-cursor
          cli_name: cursor
          __path__: {workspace_path}/.cursor-agent/**/*.{{jsonl,log}}
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-kimi
          cli_name: kimi
          __path__: {workspace_path}/.kimi/**/*.{{jsonl,log}}
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-opencode
          cli_name: opencode
          __path__: {workspace_path}/.local/share/opencode/**/*.{{jsonl,log}}
      - targets: [localhost]
        labels:
          job: acp-cli
          agent_id: "{agent_name}"
          coderun: "{coderun_name}"
          namespace: "cto"
          source: acp-copilot
          cli_name: copilot
          __path__: {workspace_path}/.copilot/**/*.{{jsonl,log}}
    pipeline_stages:
      - json:
          expressions:
            timestamp: time
            level: level
            message: message
      - regex:
          source: message
          expression: '^(?P<timestamp>\d{{4}}-\d{{2}}-\d{{2}}[T ]\d{{2}}:\d{{2}}:\d{{2}}[^\s]*)\s*(?:\[(?P<level>[A-Z]+)\])?\s*(?P<content>.*)$'
      - template:
          source: level
          template: '{{{{ if .level }}}}{{{{ .level | ToLower }}}}{{{{ else }}}}info{{{{ end }}}}'
      - labels:
          level:
      - timestamp:
          source: timestamp
          format: RFC3339
          fallback_formats:
            - "2006-01-02T150405"
            - "2006-01-02 15:04:05"
      - drop:
          older_than: 6h
          drop_counter_reason: stale_cli_entry
"#,
            );

            // Store the promtail YAML config — write via init container to emptyDir
            // This avoids needing a separate ConfigMap resource
            volumes.push(json!({
                "name": "promtail-config",
                "emptyDir": {}
            }));

            volumes.push(json!({
                "name": "promtail-positions",
                "emptyDir": {}
            }));

            // Promtail watches /workspace/.agent_done and self-terminates after the
            // main agent container exits, matching the docker-daemon sidecar pattern.
            let promtail_container = json!({
                "name": "promtail",
                "image": "grafana/promtail:latest",
                "imagePullPolicy": "IfNotPresent",
                "command": ["/bin/sh", "-c"],
                "args": [
                    "/usr/bin/promtail -config.file=/etc/promtail/promtail.yaml & PROM_PID=$!; \
                     while true; do \
                       if [ -f /workspace/.agent_done ]; then \
                         echo '[promtail] Agent done signal detected, flushing and stopping...'; \
                         sleep 10; \
                         kill -TERM $PROM_PID 2>/dev/null || true; \
                         wait $PROM_PID 2>/dev/null; \
                         exit 0; \
                       fi; \
                       if ! kill -0 $PROM_PID 2>/dev/null; then \
                         echo '[promtail] Promtail exited unexpectedly'; \
                         exit 1; \
                       fi; \
                       sleep 5; \
                     done"
                ],
                "resources": {
                    "requests": { "cpu": "10m", "memory": "32Mi" },
                    "limits": { "cpu": "100m", "memory": "64Mi" }
                },
                "volumeMounts": [
                    { "name": "workspace", "mountPath": "/workspace", "readOnly": true },
                    { "name": "promtail-config", "mountPath": "/etc/promtail" },
                    { "name": "promtail-positions", "mountPath": "/run/promtail" }
                ]
            });

            containers.push(promtail_container);

            // Store config for init container to write
            promtail_init_config = Some(promtail_config);

            info!(
                "Added promtail sidecar for CodeRun {} (agent: {}, Loki: {})",
                coderun_name, agent_name, loki_url
            );
        }

        // Add code-server + cloudflared sidecars if enabled
        if code_run.spec.enable_code_server {
            let cs_data_dir = format!("/workspace/{workspace_subdir}/.code-server");
            let cs_url_path = format!("/workspace/{workspace_subdir}/.code-server-url");

            // Bootstrap script: settings, CTO sidebar state, extension install.
            // settings.json and storage.json bodies come from shared/code-server-config/
            // so drift with the persistent Helm deployment is impossible.
            let bootstrap_script = format!(
                r#"set -eu
CS_DIR="{cs_data_dir}"
mkdir -p "$CS_DIR/User/globalStorage"

# VS Code settings (shared/code-server-config/settings.json)
if [ ! -f "$CS_DIR/User/settings.json" ]; then
cat > "$CS_DIR/User/settings.json" << 'SETTINGS'
{settings_json}
SETTINGS
fi

# Pre-seed state: CTO sidebar active (shared/code-server-config/storage.json)
if [ ! -f "$CS_DIR/User/globalStorage/storage.json" ]; then
cat > "$CS_DIR/User/globalStorage/storage.json" << 'STATE'
{storage_json}
STATE
fi

# Install CTO sidebar extension if available
EXTDIR="$CS_DIR/extensions"
mkdir -p "$EXTDIR"
VSIX_URL="https://github.com/5dlabs/cto/releases/download/cto-sidebar-latest/cto-sidebar.vsix"
VSIX_PATH="/workspace/.code-server-cache/cto-sidebar.vsix"
mkdir -p /workspace/.code-server-cache
if [ ! -f "$VSIX_PATH" ]; then
  curl -sL "$VSIX_URL" -o "$VSIX_PATH" 2>/dev/null || true
fi
if [ -f "$VSIX_PATH" ]; then
  code-server --extensions-dir "$EXTDIR" --install-extension "$VSIX_PATH" 2>/dev/null || true
fi

# Start code-server with agent_done watcher
exec code-server \
  --disable-telemetry \
  --user-data-dir "$CS_DIR" \
  --extensions-dir "$EXTDIR" \
  --bind-addr 0.0.0.0:8080 \
  --auth none \
  /workspace/{workspace_subdir} &
CS_PID=$!

# Watch for agent completion
while true; do
  if [ -f /workspace/.agent_done ]; then
    echo "[code-server] Agent done, shutting down..."
    kill -TERM $CS_PID 2>/dev/null || true
    sleep 2
    kill -KILL $CS_PID 2>/dev/null || true
    exit 0
  fi
  if ! kill -0 $CS_PID 2>/dev/null; then
    echo "[code-server] Process exited unexpectedly"
    exit 1
  fi
  sleep 5
done"#,
                cs_data_dir = cs_data_dir,
                workspace_subdir = workspace_subdir,
                settings_json = CODE_SERVER_SETTINGS_JSON.trim_end(),
                storage_json = CODE_SERVER_STORAGE_JSON.trim_end(),
            );

            let code_server_spec = json!({
                "name": "code-server",
                "image": "codercom/code-server:latest",
                "command": ["/bin/sh", "-c"],
                "args": [bootstrap_script],
                "securityContext": {
                    "runAsUser": 0
                },
                "ports": [{
                    "name": "code-server",
                    "containerPort": 8080,
                    "protocol": "TCP"
                }],
                "volumeMounts": [
                    {"name": "workspace", "mountPath": "/workspace"}
                ],
                "resources": {
                    "requests": {"cpu": "50m", "memory": "256Mi"},
                    "limits": {"cpu": "1", "memory": "1Gi"}
                }
            });
            containers.push(code_server_spec);

            // Cloudflared sidecar for ephemeral tunnel URL
            let tunnel_script = format!(
                r#"cloudflared tunnel --url http://localhost:8080 --no-autoupdate 2>&1 | \
tee /dev/stderr | while IFS= read -r line; do
  case "$line" in
    *trycloudflare.com*)
      URL=$(echo "$line" | grep -oE 'https://[a-z0-9-]+\.trycloudflare\.com')
      if [ -n "$URL" ]; then
        echo "$URL" > {cs_url_path}
        echo "[cloudflared] Tunnel URL: $URL"
        # Patch pod annotation so controller can read it
        KUBE_TOKEN=$(cat /var/run/secrets/kubernetes.io/serviceaccount/token 2>/dev/null || echo "")
        if [ -n "$KUBE_TOKEN" ]; then
          NAMESPACE=$(cat /var/run/secrets/kubernetes.io/serviceaccount/namespace 2>/dev/null || echo "cto")
          POD_NAME=$(hostname)
          curl -sk -X PATCH \
            -H "Authorization: Bearer $KUBE_TOKEN" \
            -H "Content-Type: application/merge-patch+json" \
            -d "{{\\"metadata\":{{\\"annotations\":{{\\"cto.5dlabs.ai/code-server-url\":\"$URL\"}}}}}}" \
            "https://kubernetes.default.svc/api/v1/namespaces/$NAMESPACE/pods/$POD_NAME" \
            >/dev/null 2>&1 || true
        fi
      fi
      ;;
  esac
done &
# Watch for agent_done
while true; do
  if [ -f /workspace/.agent_done ]; then
    echo "[cloudflared] Agent done, exiting..."
    exit 0
  fi
  sleep 10
done"#
            );

            let cloudflared_spec = json!({
                "name": "cloudflared",
                "image": "cloudflare/cloudflared:latest",
                "command": ["/bin/sh", "-c"],
                "args": [tunnel_script],
                "volumeMounts": [
                    {"name": "workspace", "mountPath": "/workspace"}
                ],
                "resources": {
                    "requests": {"cpu": "10m", "memory": "64Mi"},
                    "limits": {"cpu": "200m", "memory": "256Mi"}
                }
            });
            containers.push(cloudflared_spec);

            info!(
                "Added code-server + cloudflared sidecars for CodeRun {}",
                coderun_name
            );
        }

        // Build init containers array
        // Always include the workspace setup init container that:
        // 1. Creates a unique subdirectory per CodeRun to prevent git lock conflicts
        // 2. Sets proper ownership for the agent (uid 1000)
        let workspace_setup_cmd = format!(
            "mkdir -p /workspace/{workspace_subdir} && \
             mkdir -p /workspace/{workspace_subdir}/.claude/logs \
                      /workspace/{workspace_subdir}/.codex/logs \
                      /workspace/{workspace_subdir}/.factory/logs \
                      /workspace/{workspace_subdir}/.gemini/logs \
                      /workspace/{workspace_subdir}/.cursor-agent/logs \
                      /workspace/{workspace_subdir}/.kimi/logs \
                      /workspace/{workspace_subdir}/.local/share/opencode/log \
                      /workspace/{workspace_subdir}/.copilot/logs \
                      /workspace/{workspace_subdir}/.pi/logs && \
             chown -R 1000:1000 /workspace/runs && chmod -R ug+rwX /workspace/runs"
        );
        let mut init_containers = vec![json!({
            "name": "setup-workspace",
            "image": "busybox:1.36",
            "command": ["/bin/sh", "-lc", workspace_setup_cmd],
            "securityContext": {
                "runAsUser": 0,
                "runAsGroup": 0,
                "allowPrivilegeEscalation": false
            },
            "volumeMounts": [
                {"name": "workspace", "mountPath": "/workspace"},
                {"name": "github-app-key", "mountPath": "/var/run/secrets/github-app-key", "readOnly": false}
            ]
        })];

        // For watcher CodeRuns, add init container to copy coordination.json to workspace
        // This copies the initial state from the ConfigMap to the shared workspace PVC
        // The watcher (and executor) can then read/write this file for coordination
        if code_run.spec.watcher_for.is_some() {
            init_containers.push(json!({
                "name": "copy-coordination",
                "image": "busybox:1.36",
                "command": ["/bin/sh", "-c"],
                "args": [
                    // Copy coordination.json if it doesn't exist (preserve existing state)
                    // or always overwrite on watcher start to ensure fresh state
                    "cp /config/coordination/coordination.json /workspace/coordination.json && \
                     chown 1000:1000 /workspace/coordination.json && \
                     chmod 664 /workspace/coordination.json && \
                     echo 'Copied coordination.json to workspace'"
                ],
                "securityContext": {
                    "runAsUser": 0,
                    "runAsGroup": 0,
                    "allowPrivilegeEscalation": false
                },
                "volumeMounts": [
                    {"name": "workspace", "mountPath": "/workspace"},
                    {"name": "coordination-config", "mountPath": "/config/coordination"}
                ]
            }));
            info!("Added coordination copy init container for watcher CodeRun");
        }

        // Fix OpenClaw missing peer dependencies (e.g. @buape/carbon).
        // The init container runs as root and installs any additional modules
        // (like mem0 plugin) into the shared openclaw-node-modules emptyDir.
        // It first copies the image's existing node_modules into the emptyDir
        // so they are preserved, then installs extras on top.
        // Skipped for Hermes pods which don't use OpenClaw.
        if !matches!(code_run.spec.effective_harness(), HarnessAgent::Hermes) {
            let agent_image = self.select_image_for_cli(code_run).unwrap_or_else(|_| {
                format!(
                    "{}:{}",
                    self.config.agent.image.repository, self.config.agent.image.tag
                )
            });
            init_containers.push(json!({
                "name": "fix-openclaw-deps",
                "image": agent_image,
                "imagePullPolicy": self.resolve_image_pull_policy(&agent_image),
                "command": ["/bin/sh", "-c",
                    "set -e && \
                     OC_DIR=/usr/local/share/npm-global/lib/node_modules/openclaw && \
                     NM_MOUNT=/mnt/openclaw-nm && \
                     echo '[fix-openclaw-deps] copying image node_modules into emptyDir...' && \
                     cp -a $OC_DIR/node_modules/. $NM_MOUNT/ 2>/dev/null || true && \
                     echo \"[fix-openclaw-deps] copied $(ls $NM_MOUNT | wc -l) packages\" && \
                     cd $OC_DIR && \
                     npm install @mem0/openclaw-mem0 --no-audit --no-fund --legacy-peer-deps --loglevel=warn 2>&1 | tail -5 && \
                     cp -a $OC_DIR/node_modules/. $NM_MOUNT/ 2>/dev/null || true && \
                     echo \"[fix-openclaw-deps] final: $(ls $NM_MOUNT | wc -l) packages\" && \
                     echo '[fix-openclaw-deps] done'"
                ],
                "securityContext": {
                    "runAsUser": 0,
                    "runAsGroup": 0,
                    "allowPrivilegeEscalation": false
                },
                "volumeMounts": [
                    {"name": "openclaw-node-modules", "mountPath": "/mnt/openclaw-nm"}
                ]
            }));
            info!(
                "Added fix-openclaw-deps init container for CodeRun {}",
                coderun_name
            );
        }

        // Add promtail config writer init container
        if let Some(config_yaml) = promtail_init_config {
            let escaped = config_yaml.replace('\'', "'\\''");
            init_containers.push(json!({
                "name": "write-promtail-config",
                "image": "busybox:1.36",
                "command": ["/bin/sh", "-c", format!("printf '%s' '{}' > /etc/promtail/promtail.yaml", escaped)],
                "volumeMounts": [
                    {"name": "promtail-config", "mountPath": "/etc/promtail"}
                ]
            }));
            info!(
                "Added promtail config init container for CodeRun {}",
                coderun_name
            );
        }

        // Stagger gateway startup to avoid Discord API rate limits (429).
        // When many CodeRuns share the same bot token, simultaneous /gateway/bot
        // calls trigger Discord rate limiting. This init container adds a
        // deterministic delay (0-30s) based on a hash of the CodeRun name,
        // spreading pod startups across time.
        // Skipped for Hermes pods which don't use the Discord gateway.
        if !matches!(code_run.spec.effective_harness(), HarnessAgent::Hermes) {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            coderun_name.hash(&mut hasher);
            let stagger_secs = hasher.finish() % 31; // 0-30 seconds
            init_containers.push(json!({
                "name": "gateway-stagger",
                "image": "busybox:1.36",
                "command": ["/bin/sh", "-c", format!(
                    "echo 'Staggering gateway startup by {stagger_secs}s to avoid Discord rate limits' && sleep {stagger_secs}"
                )],
                "resources": {
                    "requests": { "cpu": "1m", "memory": "4Mi" },
                    "limits": { "cpu": "10m", "memory": "8Mi" }
                }
            }));
            info!(
                "Added gateway-stagger init container ({stagger_secs}s delay) for CodeRun {}",
                coderun_name
            );
        }

        // Build Pod spec and set ServiceAccountName (required by CRD)
        let mut pod_spec = json!({
            "shareProcessNamespace": true,
            "restartPolicy": "Never",
            "terminationGracePeriodSeconds": 60,
            "securityContext": {
                "runAsUser": 1000,
                "runAsGroup": 1000,
                "fsGroup": 1000,
                "fsGroupChangePolicy": "OnRootMismatch"
            },
            "initContainers": init_containers,
            "containers": containers,
            "volumes": volumes
        });

        // Short hostname to avoid mDNS 63-byte label overflow from @homebridge/ciao
        // OpenClaw appends " (OpenClaw)" (11 chars) to hostname for mDNS registration
        let task_id = code_run.spec.task_id.unwrap_or(0);
        let uid_short = code_run
            .metadata
            .uid
            .as_ref()
            .map_or("x", |u| &u[..6.min(u.len())]);
        pod_spec["hostname"] = json!(format!("t{task_id}-{uid_short}"));

        if cli_type == CLIType::Codex {
            pod_spec["securityContext"] = json!({
                "runAsUser": 0,
                "runAsGroup": 0,
                "fsGroupChangePolicy": "OnRootMismatch"
            });
            // Keep init containers (promtail config writer, workspace setup, etc.)
            // Only the securityContext needs root for Codex's sandbox mode.
        }

        // Prefer CRD-provided ServiceAccountName; else use controller default if set
        if let Some(sa_name) = code_run
            .spec
            .service_account_name
            .as_ref()
            .filter(|s| !s.trim().is_empty())
        {
            pod_spec["serviceAccountName"] = json!(sa_name.clone());
        } else if let Some(default_sa) = self
            .config
            .agent
            .service_account_name
            .as_ref()
            .filter(|s| !s.trim().is_empty())
        {
            pod_spec["serviceAccountName"] = json!(default_sa.clone());
        }
        // Add imagePullSecrets from controller config if any are defined
        if !self.config.agent.image_pull_secrets.is_empty() {
            let secrets: Vec<serde_json::Value> = self
                .config
                .agent
                .image_pull_secrets
                .iter()
                .map(|name| json!({"name": name}))
                .collect();
            pod_spec["imagePullSecrets"] = json!(secrets);
        }

        // Datadog autodiscovery annotations for container log collection.
        // Tags are derived from the CRD spec so every facet is individually searchable.
        let dd_agent_name = labels
            .get("agent")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let dd_service = format!("cto-coderun-{dd_agent_name}");
        let dd_cli_type = labels
            .get("cli-type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let dd_provider = provider_config.provider.to_string();
        let dd_model = &code_run.spec.model;
        let dd_task_id = code_run.spec.task_id.unwrap_or(0);
        let dd_run_type = &code_run.spec.run_type;
        let dd_service_target = &code_run.spec.service;
        let dd_ctx_ver = code_run.spec.context_version;

        // Build tags from all available CRD spec fields
        let mut dd_tags: Vec<String> = vec![
            format!("agent:{dd_agent_name}"),
            format!("cli:{dd_cli_type}"),
            format!("provider:{dd_provider}"),
            format!("model:{dd_model}"),
            format!("task:{dd_task_id}"),
            format!("coderun:{coderun_name}"),
            format!("run_type:{dd_run_type}"),
            format!("service_target:{dd_service_target}"),
            "env:production".to_string(),
        ];
        if let Some(ref app) = code_run.spec.github_app {
            dd_tags.push(format!("github_app:{app}"));
        }
        if let Some(ref cli_cfg) = code_run.spec.cli_config {
            if let Some(max_tokens) = cli_cfg.max_tokens {
                dd_tags.push(format!("max_tokens:{max_tokens}"));
            }
            if let Some(temp) = cli_cfg.temperature {
                dd_tags.push(format!("temperature:{temp}"));
            }
        }
        dd_tags.push(format!("context_version:{dd_ctx_ver}"));
        if code_run.spec.continue_session {
            dd_tags.push("continue_session:true".to_string());
        }
        if code_run.spec.enable_docker {
            dd_tags.push("docker:enabled".to_string());
        }

        let tags_json: Vec<String> = dd_tags.iter().map(|t| format!("\"{t}\"")).collect();
        let tags_joined = tags_json.join(",");
        let dd_log_config = format!(
            "[{{\"source\":\"cto-coderun\",\"service\":\"{dd_service}\",\"auto_multi_line_detection\":true,\"tags\":[{tags_joined}]}}]",
        );
        let mut dd_annotations = serde_json::Map::new();
        // Container name is already capped at 58 chars by container_name_for_cli(),
        // so "{name}.logs" always fits within K8s 63-char annotation name limit.
        dd_annotations.insert(
            format!("ad.datadoghq.com/{container_name}.logs"),
            json!(dd_log_config),
        );
        dd_annotations.insert(
            "ad.datadoghq.com/promtail.logs".to_string(),
            json!(format!(
                "[{{\"source\":\"cto-promtail\",\"service\":\"{dd_service}\"}}]",
            )),
        );
        let dd_annotations = serde_json::Value::Object(dd_annotations);

        let mut job_spec = json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": job_name,
                "labels": labels,
                "ownerReferences": [{
                    "apiVersion": owner_ref.api_version,
                    "kind": owner_ref.kind,
                    "name": owner_ref.name,
                    "uid": owner_ref.uid,
                    "controller": owner_ref.controller,
                    "blockOwnerDeletion": owner_ref.block_owner_deletion
                }]
            },
            "spec": {
                "backoffLimit": 0,
                "activeDeadlineSeconds": 86400,  // 24-hour ultimate safety net (tasks can legitimately run for hours)
                "template": {
                    "metadata": {
                        "labels": labels,
                        "annotations": dd_annotations
                    },
                    "spec": pod_spec
                }
            }
        });

        // Only set TTL for non-workflow-managed jobs
        // Workflow-owned jobs should be cleaned up by the workflow itself
        let has_workflow_owner = code_run
            .metadata
            .owner_references
            .as_ref()
            .and_then(|refs| refs.iter().find(|r| r.kind == "Workflow"))
            .is_some();

        if !has_workflow_owner {
            // Standalone CodeRun - set TTL for automatic cleanup
            job_spec["spec"]["ttlSecondsAfterFinished"] = json!(3600);
        }
        // Workflow-owned CodeRuns: no TTL set - workflow manages lifecycle

        Ok(serde_json::from_value(job_spec)?)
    }

    #[allow(clippy::too_many_lines, clippy::items_after_statements)] // Complex task requirement processing
    fn process_task_requirements(
        code_run: &CodeRun,
        mut env_vars: Vec<serde_json::Value>,
    ) -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>)> {
        let mut env_from = Vec::new();

        // Tracking for visibility (names only, never values)
        let mut workflow_env_names: BTreeSet<String> = BTreeSet::new();
        let mut req_env_names: BTreeSet<String> = BTreeSet::new();
        let mut req_secret_sources: BTreeSet<String> = BTreeSet::new();

        // ALWAYS process spec.env first (workflow-provided env vars like PR_URL, PR_NUMBER)
        for (key, value) in &code_run.spec.env {
            env_vars.push(json!({
                "name": key,
                "value": value
            }));
            workflow_env_names.insert(key.clone());
        }

        // Check if we have non-empty task requirements
        let has_valid_requirements = code_run
            .spec
            .task_requirements
            .as_ref()
            .is_some_and(|r| !r.trim().is_empty());

        if has_valid_requirements {
            let requirements_b64 = code_run.spec.task_requirements.as_ref().unwrap();
            use base64::{engine::general_purpose, Engine as _};

            // Decode base64
            let decoded = general_purpose::STANDARD
                .decode(requirements_b64)
                .map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to decode task requirements: {e}"
                    ))
                })?;

            // Parse YAML
            let requirements: serde_yaml::Value =
                serde_yaml::from_slice(&decoded).map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to parse task requirements YAML: {e}"
                    ))
                })?;

            // Process secrets
            if let Some(secrets) = requirements.get("secrets").and_then(|s| s.as_sequence()) {
                for secret in secrets {
                    if let Some(secret_map) = secret.as_mapping() {
                        if let Some(name) = secret_map.get("name").and_then(|n| n.as_str()) {
                            req_secret_sources.insert(name.to_string());
                            // Check if we have specific key mappings
                            if let Some(keys) = secret_map.get("keys").and_then(|k| k.as_sequence())
                            {
                                // Mount specific keys as individual env vars
                                for key_mapping in keys {
                                    if let Some(key_map) = key_mapping.as_mapping() {
                                        for (k8s_key, env_name) in key_map {
                                            if let (Some(k8s_key_str), Some(env_name_str)) =
                                                (k8s_key.as_str(), env_name.as_str())
                                            {
                                                env_vars.push(json!({
                                                    "name": env_name_str,
                                                    "valueFrom": {
                                                        "secretKeyRef": {
                                                            "name": name,
                                                            "key": k8s_key_str
                                                        }
                                                    }
                                                }));
                                                req_env_names.insert(env_name_str.to_string());
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Mount entire secret as env vars
                                env_from.push(json!({
                                    "secretRef": {
                                        "name": name
                                    }
                                }));
                            }
                        }
                    }
                }
            }

            // Process static environment variables
            if let Some(env) = requirements.get("environment").and_then(|e| e.as_mapping()) {
                for (key, value) in env {
                    if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                        env_vars.push(json!({
                            "name": key_str,
                            "value": value_str
                        }));
                        req_env_names.insert(key_str.to_string());
                    }
                }
            }
            // Only process legacy env_from_secrets if task requirements don't exist
            // This prevents conflicts between task_requirements and legacy env_from_secrets
        } else {
            // Process legacy env_from_secrets only when no task requirements are present
            for secret_env in &code_run.spec.env_from_secrets {
                env_vars.push(json!({
                    "name": &secret_env.name,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": &secret_env.secret_name,
                            "key": &secret_env.secret_key
                        }
                    }
                }));
                req_env_names.insert(secret_env.name.clone());
                req_secret_sources.insert(secret_env.secret_name.clone());
            }
        }

        // Surface non-sensitive visibility of env var allowances to the container as JSON strings
        let wf_env_list: Vec<String> = workflow_env_names.into_iter().collect();
        let req_env_list: Vec<String> = req_env_names.into_iter().collect();
        let req_secret_list: Vec<String> = req_secret_sources.into_iter().collect();

        let wf_env_json = serde_json::to_string(&wf_env_list).unwrap_or_else(|_| "[]".to_string());
        let req_env_json =
            serde_json::to_string(&req_env_list).unwrap_or_else(|_| "[]".to_string());
        let req_secret_json =
            serde_json::to_string(&req_secret_list).unwrap_or_else(|_| "[]".to_string());

        env_vars.push(json!({ "name": "WORKFLOW_ENV_VARS", "value": wf_env_json }));
        env_vars.push(json!({ "name": "REQUIREMENTS_ENV_VARS", "value": req_env_json }));
        env_vars.push(json!({ "name": "REQUIREMENTS_SECRET_SOURCES", "value": req_secret_json }));

        Ok((env_vars, env_from))
    }

    #[allow(clippy::too_many_lines)]
    fn create_task_labels(code_run: &CodeRun) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        let cli_type = Self::code_run_cli_type(code_run);
        let container_label = Self::container_name_for_cli(cli_type, &code_run.spec.model);

        // Update legacy orchestrator label to controller
        labels.insert("app".to_string(), "controller".to_string());
        labels.insert("component".to_string(), "code-runner".to_string());

        labels.insert(LABEL_CLEANUP_SCOPE.to_string(), SCOPE_RUN.to_string());
        labels.insert(LABEL_CLEANUP_KIND.to_string(), "coderun".to_string());
        if let Some(name) = code_run.metadata.name.as_deref() {
            labels.insert(
                LABEL_CLEANUP_RUN.to_string(),
                Self::sanitize_label_value(name),
            );
        }

        // Project identification labels
        labels.insert("job-type".to_string(), "code".to_string());

        // Use service as project name for code tasks
        labels.insert(
            "project-name".to_string(),
            Self::sanitize_label_value(&code_run.spec.service),
        );

        let github_identifier = code_run
            .spec
            .github_app
            .as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");
        labels.insert(
            "github-user".to_string(),
            Self::sanitize_label_value(github_identifier),
        );
        labels.insert(
            "context-version".to_string(),
            code_run.spec.context_version.to_string(),
        );

        // Code-specific labels
        labels.insert("task-type".to_string(), code_run.spec.run_type.clone());
        labels.insert(
            "task-id".to_string(),
            code_run
                .spec
                .task_id
                .map_or("0".to_string(), |id| id.to_string()),
        );

        // Add PR number label if available in env for better pod correlation
        if let Some(pr_number) = code_run.spec.env.get("PR_NUMBER") {
            labels.insert(
                "pr-number".to_string(),
                Self::sanitize_label_value(pr_number),
            );
        }
        labels.insert(
            "service".to_string(),
            Self::sanitize_label_value(&code_run.spec.service),
        );
        labels.insert(
            "cli-type".to_string(),
            Self::sanitize_label_value(&cli_type.to_string()),
        );
        if !code_run.spec.model.trim().is_empty() {
            labels.insert(
                "cli-model".to_string(),
                Self::sanitize_label_value(&code_run.spec.model),
            );
        }
        labels.insert(
            "cli-container".to_string(),
            Self::sanitize_label_value(&container_label),
        );

        // Datadog unified service tagging — lets DD filter CodeRun pods distinctly
        // Extract agent name from GitHub app (e.g., "5DLabs-Rex" → "rex")
        let agent_name = code_run
            .spec
            .github_app
            .as_ref()
            .and_then(|app| AgentClassifier::new().extract_agent_name(app).ok())
            .map_or_else(|| "unknown".to_string(), |n| n.to_lowercase());
        labels.insert("agent".to_string(), Self::sanitize_label_value(&agent_name));
        // Dual-write: implementation-agent label for new naming convention
        let impl_agent_name = code_run
            .spec
            .implementation_agent
            .as_ref()
            .filter(|a| !a.is_empty())
            .map_or_else(|| agent_name.clone(), |a| a.to_lowercase());
        labels.insert(
            "implementation-agent".to_string(),
            Self::sanitize_label_value(&impl_agent_name),
        );
        labels.insert(
            "tags.datadoghq.com/service".to_string(),
            format!("cto-coderun-{}", Self::sanitize_label_value(&agent_name)),
        );
        labels.insert(
            "tags.datadoghq.com/env".to_string(),
            "production".to_string(),
        );
        // CLI type and provider as DD-visible labels for faceted search
        labels.insert(
            "tags.datadoghq.com/version".to_string(),
            cli_type.to_string().to_lowercase(),
        );

        // Add Linear session labels for pod discovery (used by PM server for routing input)
        if let Some(linear) = &code_run.spec.linear_integration {
            if linear.enabled {
                if let Some(session_id) = &linear.session_id {
                    labels.insert(
                        "linear-session".to_string(),
                        Self::sanitize_label_value(session_id),
                    );
                }
                if let Some(issue_id) = &linear.issue_id {
                    labels.insert(
                        "cto.5dlabs.io/linear-issue".to_string(),
                        Self::sanitize_label_value(issue_id),
                    );
                }
                // Add agent type label for better observability
                if let Some(github_app) = &code_run.spec.github_app {
                    if let Ok(agent_name) = AgentClassifier::new().extract_agent_name(github_app) {
                        labels.insert(
                            "cto.5dlabs.io/agent-type".to_string(),
                            Self::sanitize_label_value(&agent_name.to_lowercase()),
                        );
                    }
                }
            }
        }

        labels
    }

    async fn update_configmap_owner(
        &self,
        _code_run: &CodeRun,
        cm_name: &str,
        owner_ref: OwnerReference,
    ) -> Result<()> {
        let mut existing_cm = self.configmaps.get(cm_name).await?;

        // Add owner reference
        let owner_refs = existing_cm
            .metadata
            .owner_references
            .get_or_insert_with(Vec::new);
        owner_refs.push(owner_ref);

        // Update the ConfigMap
        self.configmaps
            .replace(cm_name, &PostParams::default(), &existing_cm)
            .await?;
        info!("Updated ConfigMap {} with owner reference", cm_name);

        Ok(())
    }

    // Legacy cleanup method for backward compatibility
    async fn cleanup_old_jobs(&self, code_run: &CodeRun) -> Result<()> {
        // First, try to delete the specific job from CodeRun status if available
        if let Some(status) = &code_run.status {
            if let Some(job_name) = &status.job_name {
                info!("Deleting CodeRun job from status: {}", job_name);
                let delete_params = DeleteParams {
                    propagation_policy: Some(kube::api::PropagationPolicy::Background),
                    ..Default::default()
                };
                match self.jobs.delete(job_name, &delete_params).await {
                    Ok(_) => {
                        info!("Successfully deleted job: {}", job_name);
                        // Job deleted successfully - no need for label-based cleanup
                        return Ok(());
                    }
                    Err(kube::Error::Api(ae)) if ae.code == 404 => {
                        info!("Job {} not found (may already be deleted)", job_name);
                        // Job not found - no need for label-based cleanup
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Failed to delete job {}: {}", job_name, e);
                        // Continue to label-based cleanup as fallback
                    }
                }
            }
        }

        // Fallback: Clean up any remaining jobs by label selector
        // This handles cases where job_name might not be set in status
        // IMPORTANT: Include LABEL_CLEANUP_RUN to only delete jobs belonging to this CodeRun
        let github_identifier = code_run
            .spec
            .github_app
            .as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");

        let code_run_name = code_run.metadata.name.as_deref().unwrap_or("unknown");
        let list_params = ListParams::default().labels(&format!(
            "app=controller,component=code-runner,github-user={},service={},{}={}",
            Self::sanitize_label_value(github_identifier),
            Self::sanitize_label_value(&code_run.spec.service),
            LABEL_CLEANUP_RUN,
            Self::sanitize_label_value(code_run_name)
        ));

        let jobs = self.jobs.list(&list_params).await?;

        for job in jobs {
            if let Some(job_name) = job.metadata.name {
                info!("Deleting old code job: {}", job_name);
                // Use Background propagation to ensure pods are cleaned up
                let delete_params = DeleteParams {
                    propagation_policy: Some(kube::api::PropagationPolicy::Background),
                    ..Default::default()
                };
                let _ = self.jobs.delete(&job_name, &delete_params).await;
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_lines)] // Complex function not easily split
    async fn cleanup_old_configmaps(&self, code_run: &CodeRun) -> Result<()> {
        // Generate current ConfigMap name to avoid deleting it
        let current_cm_name = Self::generate_configmap_name(code_run);

        let github_identifier = code_run
            .spec
            .github_app
            .as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");

        // IMPORTANT: Only cleanup ConfigMaps with BOTH github-user AND service labels
        // This prevents accidentally matching ConfigMaps from other stages in multi-agent workflows
        let list_params = ListParams::default().labels(&format!(
            "app=controller,component=code-runner,github-user={},service={}",
            Self::sanitize_label_value(github_identifier),
            Self::sanitize_label_value(&code_run.spec.service)
        ));

        let configmaps = self.configmaps.list(&list_params).await?;

        // Get pods API for checking if pods are still running
        let namespace = code_run
            .metadata
            .namespace
            .as_deref()
            .unwrap_or(&self.ctx.namespace);
        let pods: Api<Pod> = Api::namespaced(self.ctx.client.clone(), namespace);

        for cm in configmaps {
            if let Some(cm_name) = cm.metadata.name {
                // Skip deleting the current ConfigMap - this prevents deletion of active job's ConfigMap
                if cm_name == current_cm_name {
                    info!("Skipping deletion of current ConfigMap: {}", cm_name);
                    continue;
                }

                // Check if ConfigMap has an owner reference to a Job
                let job_owner_name = cm.metadata.owner_references.as_ref().and_then(|owners| {
                    owners.iter().find_map(|owner| {
                        if owner.kind == "Job" && owner.api_version.starts_with("batch/") {
                            Some(owner.name.clone())
                        } else {
                            None
                        }
                    })
                });

                if let Some(job_name) = job_owner_name {
                    // First check if the Job itself is still active (not completed/failed)
                    // This prevents race conditions where pods might be created after cleanup runs
                    match self.jobs.get(&job_name).await {
                        Ok(job) => {
                            // Job is active if status is None or if it hasn't completed/failed
                            let is_job_active = job.status.as_ref().is_none_or(|status| {
                                status.completion_time.is_none() && status.failed.unwrap_or(0) == 0
                            });

                            if is_job_active {
                                info!(
                                    "Skipping cleanup of ConfigMap {} - job {} is still active",
                                    cm_name, job_name
                                );
                                continue;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to get job {}: {} - skipping ConfigMap deletion for safety",
                                job_name, e
                            );
                            continue;
                        }
                    }

                    // Also check if any pods from this job are still running/pending
                    let pod_list_params = ListParams::default()
                        .labels(&format!("batch.kubernetes.io/job-name={job_name}"));
                    match pods.list(&pod_list_params).await {
                        Ok(pod_list) => {
                            // Check for ANY pods that are Running, Pending, or Init (mounting volumes)
                            let has_active_pods = pod_list.items.iter().any(|pod| {
                                pod.status
                                    .as_ref()
                                    .and_then(|s| s.phase.as_deref())
                                    .is_some_and(|phase| {
                                        // Protect ConfigMaps for pods that might be starting
                                        phase == "Running" || phase == "Pending"
                                    }) ||
                                // Also check for Init containers (still mounting volumes)
                                pod.status
                                    .as_ref()
                                    .and_then(|s| s.init_container_statuses.as_ref())
                                    .is_some_and(|containers| containers.iter().any(|c| {
                                        c.state.as_ref().is_some_and(|state| {
                                            state.running.is_some() || state.waiting.is_some()
                                        })
                                    }))
                            });

                            if has_active_pods {
                                info!(
                                    "Skipping cleanup of ConfigMap {} - job {} still has active pods",
                                    cm_name, job_name
                                );
                                continue;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to list pods for job {}: {} - skipping ConfigMap deletion for safety",
                                job_name, e
                            );
                            continue;
                        }
                    }
                } else {
                    // ConfigMap has no owner reference - check if any jobs are using it
                    // This protects ConfigMaps that were just created but don't have owner ref yet
                    // Use same label filters as ConfigMap listing to scope the search
                    let job_list_params = ListParams::default().labels(&format!(
                        "app=controller,component=code-runner,github-user={},service={}",
                        Self::sanitize_label_value(github_identifier),
                        Self::sanitize_label_value(&code_run.spec.service)
                    ));

                    let all_jobs = match self.jobs.list(&job_list_params).await {
                        Ok(jobs) => jobs,
                        Err(e) => {
                            warn!(
                                "Failed to list jobs to check ConfigMap usage: {} - skipping deletion for safety",
                                e
                            );
                            continue;
                        }
                    };

                    // Check if any job references this ConfigMap in its volumes
                    let is_used_by_job = all_jobs.items.iter().any(|job| {
                        job.spec
                            .as_ref()
                            .and_then(|spec| spec.template.spec.as_ref())
                            .and_then(|pod_spec| pod_spec.volumes.as_ref())
                            .is_some_and(|volumes| {
                                volumes.iter().any(|vol| {
                                    vol.config_map
                                        .as_ref()
                                        .map(|cm| cm.name.clone())
                                        .is_some_and(|name| name == cm_name)
                                })
                            })
                    });

                    if is_used_by_job {
                        info!(
                            "Skipping cleanup of ConfigMap {} - it's referenced by an active job",
                            cm_name
                        );
                        continue;
                    }

                    // Also check if any pods are using this ConfigMap (in case job was deleted but pods remain)
                    // Use same label filters to scope the search
                    let pod_list_params = ListParams::default().labels(&format!(
                        "app=controller,component=code-runner,github-user={},service={}",
                        Self::sanitize_label_value(github_identifier),
                        Self::sanitize_label_value(&code_run.spec.service)
                    ));

                    match pods.list(&pod_list_params).await {
                        Ok(pod_list) => {
                            let is_used_by_pod = pod_list.items.iter().any(|pod| {
                                pod.spec
                                    .as_ref()
                                    .and_then(|spec| spec.volumes.as_ref())
                                    .is_some_and(|volumes| {
                                        volumes.iter().any(|vol| {
                                            vol.config_map
                                                .as_ref()
                                                .map(|cm| cm.name.clone())
                                                .is_some_and(|name| name == cm_name)
                                        })
                                    })
                            });

                            if is_used_by_pod {
                                info!(
                                    "Skipping cleanup of ConfigMap {} - it's referenced by an active pod",
                                    cm_name
                                );
                                continue;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to list pods to check ConfigMap usage: {} - skipping deletion for safety",
                                e
                            );
                            continue;
                        }
                    }
                }

                info!("Deleting old code ConfigMap: {}", cm_name);
                let _ = self
                    .configmaps
                    .delete(&cm_name, &DeleteParams::default())
                    .await;
            }
        }

        Ok(())
    }

    fn sanitize_label_value(input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        // Replace spaces with hyphens, convert to lowercase
        let mut sanitized = input.to_lowercase().replace([' ', '_'], "-");

        // Remove any characters that aren't alphanumeric, hyphens, underscores, or dots
        sanitized.retain(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

        // Ensure it starts and ends with alphanumeric
        let chars: Vec<char> = sanitized.chars().collect();
        let start = chars.iter().position(|c| c.is_alphanumeric()).unwrap_or(0);
        let end = chars
            .iter()
            .rposition(|c| c.is_alphanumeric())
            .unwrap_or(chars.len().saturating_sub(1));

        if start <= end {
            sanitized = chars[start..=end].iter().collect();
        }

        // Truncate to 63 characters (Kubernetes label limit)
        if sanitized.len() > 63 {
            sanitized.truncate(63);
            // Ensure it still ends with alphanumeric after truncation
            if let Some(last_alphanumeric) = sanitized.rfind(|c: char| c.is_alphanumeric()) {
                sanitized.truncate(last_alphanumeric + 1);
            }
        }

        sanitized
    }

    fn container_name_for_cli(cli_type: CLIType, model: &str) -> String {
        let mut name = cli_type.to_string();

        if !model.trim().is_empty() {
            let sanitized_model: String = model
                .chars()
                .map(|c| match c {
                    'a'..='z' | '0'..='9' => c,
                    'A'..='Z' => c.to_ascii_lowercase(),
                    _ => '-',
                })
                .collect();

            let sanitized_model = sanitized_model.trim_matches('-');
            if !sanitized_model.is_empty() {
                name.push('-');
                name.push_str(sanitized_model);
            }
        }

        let collapsed = name
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        let mut final_name = if collapsed.is_empty() {
            cli_type.to_string()
        } else {
            collapsed
        };

        // Cap at 58 chars so DD annotation key `ad.datadoghq.com/{name}.logs`
        // stays within K8s 63-char annotation name limit (58 + 5 = 63).
        if final_name.len() > 58 {
            final_name.truncate(58);
            while final_name.ends_with('-') {
                final_name.pop();
            }
        }

        if final_name.is_empty() {
            "cli".to_string()
        } else {
            final_name
        }
    }

    fn code_run_cli_type(code_run: &CodeRun) -> CLIType {
        code_run
            .spec
            .cli_config
            .as_ref()
            .map_or(CLIType::Claude, |cfg| cfg.cli_type)
    }

    /// Select the appropriate Docker image based on the CLI type specified in the `CodeRun`
    /// Auto-populate CLI config based on agent GitHub app (if not already specified)
    fn populate_cli_config_if_needed(&self, code_run: &Arc<CodeRun>) -> Arc<CodeRun> {
        // If we have no GitHub app context, we cannot enrich the CLI config
        let Some(github_app) = &code_run.spec.github_app else {
            if code_run.spec.cli_config.is_none() {
                info!("No CLI config or GitHub app specified, using defaults");
            }
            return code_run.clone();
        };

        // Extract agent name for logging only—we still continue even if this fails
        let classifier = AgentClassifier::new();
        if let Ok(agent_name) = classifier.extract_agent_name(github_app) {
            info!(
                "🔍 Preparing CLI configuration for agent '{}' ({})",
                agent_name, github_app
            );
        }

        let Some(agent_cli_config) = self.config.agent.agent_cli_configs.get(github_app) else {
            // Nothing to merge, fall back to whatever the CodeRun already provided
            return code_run.clone();
        };

        let mut new_code_run = (**code_run).clone();

        if let Some(existing) = new_code_run.spec.cli_config.as_mut() {
            Self::merge_cli_config(existing, agent_cli_config);
            self.apply_cli_provider(existing);
        } else {
            info!(
                "🔧 Auto-populating CLI config for agent {}: {} ({})",
                github_app, agent_cli_config.cli_type, agent_cli_config.model
            );
            new_code_run.spec.cli_config = Some(agent_cli_config.clone());
            if let Some(existing) = new_code_run.spec.cli_config.as_mut() {
                self.apply_cli_provider(existing);
            }
        }

        Arc::new(new_code_run)
    }

    fn merge_cli_config(existing: &mut CLIConfig, defaults: &CLIConfig) {
        if existing.model.trim().is_empty() {
            existing.model.clone_from(&defaults.model);
        }

        if existing.max_tokens.is_none() {
            existing.max_tokens = defaults.max_tokens;
        }

        if existing.temperature.is_none() {
            existing.temperature = defaults.temperature;
        }

        if existing.model_rotation.is_none() {
            existing.model_rotation.clone_from(&defaults.model_rotation);
        }

        if existing.provider.is_none() {
            existing.provider = defaults.provider;
        }

        if existing.provider_base_url.is_none() {
            existing
                .provider_base_url
                .clone_from(&defaults.provider_base_url);
        }

        if existing.api_key_env_var.is_none() {
            existing
                .api_key_env_var
                .clone_from(&defaults.api_key_env_var);
        }

        for (key, value) in &defaults.settings {
            existing
                .settings
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
    }

    fn apply_cli_provider(&self, existing: &mut CLIConfig) {
        let cli_key = existing.cli_type.to_string().to_lowercase();
        if let Some(provider_name) = self.config.agent.cli_providers.get(&cli_key) {
            // Set simple provider string for backward compatibility
            existing
                .settings
                .entry("provider".to_string())
                .or_insert_with(|| serde_json::Value::String(provider_name.clone()));

            // Build full modelProvider object with env_key from secrets config
            let provider_key = provider_name.to_lowercase();
            if let Some(provider_cfg) = self.config.secrets.provider_api_keys.get(&provider_key) {
                let provider_obj = serde_json::json!({
                    "name": provider_name,
                    "envKey": &provider_cfg.secret_key,
                });
                existing
                    .settings
                    .entry("modelProvider".to_string())
                    .or_insert(provider_obj);
            }
        }
    }

    fn select_image_for_cli(&self, code_run: &CodeRun) -> Result<String> {
        // Check if CLI config is specified
        if let Some(cli_config) = &code_run.spec.cli_config {
            // Try to get CLI-specific image configuration
            let cli_key = cli_config.cli_type.to_string().to_lowercase();
            let cli_image_opt = self.config.agent.cli_images.get(&cli_key).or_else(|| {
                self.config
                    .agent
                    .cli_images
                    .iter()
                    .find(|(key, _)| key.eq_ignore_ascii_case(&cli_key))
                    .map(|(_, img)| img)
            });

            if let Some(cli_image) = cli_image_opt {
                if cli_image.is_configured() {
                    return Ok(format!("{}:{}", cli_image.repository, cli_image.tag));
                }
            }

            return Err(Error::ConfigError(format!(
                "No image configured for CLI type {}. Configure agent.cliImages with an entry for '{}' (available keys: {:?}).",
                cli_config.cli_type,
                cli_key,
                self.config.agent.cli_images.keys().collect::<Vec<_>>()
            )));
        }

        // No CLI config specified - require explicit configuration
        if self.config.agent.image.is_configured() {
            return Ok(format!(
                "{}:{}",
                self.config.agent.image.repository, self.config.agent.image.tag
            ));
        }

        // No fallback - configuration must be explicit
        Err(Error::ConfigError(format!(
            "No CLI configuration provided for CodeRun '{}' and agent.image is not configured. \
             Either specify cli_config in the CodeRun spec or configure agent.image in controller config.",
            code_run.metadata.name.as_deref().unwrap_or("unknown")
        )))
    }

    /// Resolve image pull policy: config override > auto-detect from tag.
    fn resolve_image_pull_policy(&self, image: &str) -> &'static str {
        // Check CLI-specific config first, then default image config
        let config_policy = self
            .config
            .agent
            .cli_images
            .values()
            .find(|img| image.starts_with(&img.repository))
            .and_then(|img| img.pull_policy.as_deref())
            .or(self.config.agent.image.pull_policy.as_deref());

        match config_policy {
            Some(p) if p.eq_ignore_ascii_case("always") => "Always",
            Some(p) if p.eq_ignore_ascii_case("ifnotpresent") => "IfNotPresent",
            Some(p) if p.eq_ignore_ascii_case("never") => "Never",
            _ => {
                // Auto-detect: Always for :latest/:dev, IfNotPresent otherwise
                if image.ends_with(":latest") || image.ends_with(":dev") {
                    "Always"
                } else {
                    "IfNotPresent"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crds::coderun::LinearIntegration;
    use serde_json::json;
    use std::collections::HashMap;

    fn cli_config_with_settings(settings: HashMap<String, serde_json::Value>) -> CLIConfig {
        CLIConfig {
            cli_type: CLIType::Codex,
            model: String::new(),
            settings,
            max_tokens: None,
            temperature: None,
            model_rotation: None,
            provider: None,
            provider_base_url: None,
            api_key_env_var: None,
        }
    }

    /// Create a test CodeRun with optional linear integration for sidecar testing
    /// Note: enable_docker defaults to true (matching production behavior)
    fn create_test_code_run_with_linear(
        github_app: &str,
        cli_type: CLIType,
        linear_enabled: bool,
    ) -> CodeRun {
        use crate::crds::CodeRunSpec;
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

        let mut settings = HashMap::new();
        settings.insert("approvalPolicy".to_string(), json!("never"));

        let linear_integration = if linear_enabled {
            Some(LinearIntegration {
                enabled: true,
                session_id: Some("test-session-123".to_string()),
                access_token: None,
                issue_id: Some("TEST-456".to_string()),
                team_id: Some("team-789".to_string()),
            })
        } else {
            None
        };

        CodeRun {
            metadata: ObjectMeta {
                name: Some("sidecar-test-run".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                cli_config: Some(CLIConfig {
                    cli_type,
                    model: "test-model".to_string(),
                    settings,
                    max_tokens: Some(16000),
                    temperature: Some(0.7),
                    model_rotation: None,
                    provider: None,
                    provider_base_url: None,
                    api_key_env_var: None,
                }),
                task_id: Some(1),
                service: "test-service".to_string(),
                repository_url: "https://github.com/test/repo".to_string(),
                docs_repository_url: "https://github.com/test/docs".to_string(),
                docs_project_directory: Some("docs".to_string()),
                working_directory: Some("src".to_string()),
                model: "test-model".to_string(),
                github_user: Some("test-user".to_string()),
                github_app: Some(github_app.to_string()),
                linear_integration,
                ..Default::default()
            },
            status: None,
        }
    }

    /// Verify the sidecar container spec is correctly constructed
    #[test]
    fn test_linear_sidecar_spec_structure() {
        // When linear_integration is enabled, the sidecar should have these properties:
        // - name: "linear-sidecar"
        // - image: configurable (defaults to registry.5dlabs.ai/5dlabs/linear-sidecar:latest)
        // - volumeMounts: linear-status and workspace
        // - ports: 8080 for HTTP
        // - env: STATUS_FILE, LINEAR_SERVICE_URL, session/issue/team IDs

        let session_id = "test-session-123";
        let issue_id = "TEST-456";
        let team_id = "team-789";
        let workflow_name = "coderun-test";

        // Build the expected sidecar env vars
        let sidecar_env = vec![
            json!({ "name": "STATUS_FILE", "value": "/status/current.json" }),
            json!({ "name": "LINEAR_SERVICE_URL", "value": "http://pm-server:8080" }),
            json!({ "name": "STATUS_POLL_INTERVAL_MS", "value": "5000" }),
            json!({ "name": "LOG_POST_INTERVAL_MS", "value": "5000" }),
            json!({ "name": "INPUT_POLL_INTERVAL_MS", "value": "2000" }),
            json!({ "name": "LINEAR_SESSION_ID", "value": session_id }),
            json!({ "name": "LINEAR_ISSUE_ID", "value": issue_id }),
            json!({ "name": "LINEAR_TEAM_ID", "value": team_id }),
            json!({ "name": "WORKFLOW_NAME", "value": workflow_name }),
            json!({ "name": "RUST_LOG", "value": "info" }),
            json!({ "name": "LOG_FILE_PATH", "value": "/workspace/agent.log" }),
            json!({ "name": "INPUT_FIFO_PATH", "value": "/workspace/agent-input.jsonl" }),
            json!({ "name": "HTTP_PORT", "value": "8080" }),
            // Whip cracking configuration
            json!({ "name": "WHIP_CRACK_ENABLED", "value": "true" }),
            json!({ "name": "STALL_THRESHOLD_SECS", "value": "120" }),
            json!({ "name": "NUDGE_INTERVAL_SECS", "value": "60" }),
            json!({ "name": "MAX_NUDGE_LEVEL", "value": "3" }),
        ];

        // Verify these are the expected env vars (order matters for first N items)
        assert_eq!(sidecar_env[0]["name"], "STATUS_FILE");
        assert_eq!(sidecar_env[5]["name"], "LINEAR_SESSION_ID");
        assert_eq!(sidecar_env[5]["value"], session_id);
        // Verify whip cracking is enabled
        assert_eq!(sidecar_env[13]["name"], "WHIP_CRACK_ENABLED");
        assert_eq!(sidecar_env[13]["value"], "true");

        // Verify volume mounts structure
        let expected_volume_mounts = json!([
            { "name": "linear-status", "mountPath": "/status" },
            { "name": "workspace", "mountPath": "/workspace" }
        ]);
        assert_eq!(expected_volume_mounts.as_array().unwrap().len(), 2);
    }

    /// Verify sidecar is NOT added when linear_integration is None
    #[test]
    fn test_sidecar_not_added_when_linear_disabled() {
        let code_run = create_test_code_run_with_linear("5DLabs-Rex", CLIType::Claude, false);
        assert!(code_run.spec.linear_integration.is_none());
    }

    /// Verify sidecar configuration is set when linear_integration is enabled
    #[test]
    fn test_sidecar_added_when_linear_enabled() {
        let code_run = create_test_code_run_with_linear("5DLabs-Rex", CLIType::Claude, true);
        let linear = code_run.spec.linear_integration.as_ref().unwrap();

        assert!(linear.enabled);
        assert_eq!(linear.session_id, Some("test-session-123".to_string()));
        assert_eq!(linear.issue_id, Some("TEST-456".to_string()));
        assert_eq!(linear.team_id, Some("team-789".to_string()));
    }

    /// Verify sidecar is added for ALL agent types (implementation agents)
    #[test]
    fn test_sidecar_for_all_implementation_agents() {
        let agents = vec![
            "5DLabs-Rex",   // Rust
            "5DLabs-Blaze", // React
            "5DLabs-Grizz", // Go
            "5DLabs-Nova",  // Node.js
            "5DLabs-Tap",   // Expo
            "5DLabs-Spark", // Electron
        ];

        for agent in agents {
            let code_run = create_test_code_run_with_linear(agent, CLIType::Claude, true);
            let linear = code_run.spec.linear_integration.as_ref();

            assert!(
                linear.is_some(),
                "Agent {agent} should have linear_integration when enabled"
            );
            assert!(
                linear.unwrap().enabled,
                "Agent {agent} should have linear_integration.enabled = true"
            );
        }
    }

    /// Verify sidecar is added for ALL support agents
    #[test]
    fn test_sidecar_for_all_support_agents() {
        let agents = vec![
            "5DLabs-Cleo",   // Quality
            "5DLabs-Cipher", // Security
            "5DLabs-Tess",   // Testing
            "5DLabs-Atlas",  // Integration
            "5DLabs-Bolt",   // Infrastructure
            "5DLabs-Morgan", // PM/Docs
        ];

        for agent in agents {
            let code_run = create_test_code_run_with_linear(agent, CLIType::Claude, true);
            let linear = code_run.spec.linear_integration.as_ref();

            assert!(
                linear.is_some(),
                "Support agent {agent} should have linear_integration when enabled"
            );
            assert!(
                linear.unwrap().enabled,
                "Support agent {agent} should have linear_integration.enabled = true"
            );
        }
    }

    /// Verify sidecar is added for ALL CLI types
    #[test]
    fn test_sidecar_for_all_cli_types() {
        let cli_types = vec![
            CLIType::Claude,
            CLIType::Codex,
            CLIType::Cursor,
            CLIType::Factory,
            CLIType::Gemini,
            CLIType::OpenCode,
        ];

        for cli_type in cli_types {
            let code_run = create_test_code_run_with_linear("5DLabs-Rex", cli_type, true);
            let linear = code_run.spec.linear_integration.as_ref();

            assert!(
                linear.is_some(),
                "CLI {cli_type:?} should have linear_integration when enabled",
            );
            assert!(
                linear.unwrap().enabled,
                "CLI {cli_type:?} should have linear_integration.enabled = true",
            );
        }
    }

    /// Verify the complete agent × CLI matrix for sidecar mounting
    #[test]
    fn test_sidecar_agent_cli_matrix() {
        let agents = vec![
            "5DLabs-Rex",
            "5DLabs-Blaze",
            "5DLabs-Grizz",
            "5DLabs-Nova",
            "5DLabs-Tap",
            "5DLabs-Spark",
            "5DLabs-Cleo",
            "5DLabs-Cipher",
            "5DLabs-Tess",
            "5DLabs-Atlas",
            "5DLabs-Bolt",
            "5DLabs-Morgan",
        ];

        let cli_types = vec![
            CLIType::Claude,
            CLIType::Codex,
            CLIType::Cursor,
            CLIType::Factory,
            CLIType::Gemini,
            CLIType::OpenCode,
        ];

        let mut tested_combinations = 0;

        for agent in &agents {
            for cli_type in &cli_types {
                let code_run = create_test_code_run_with_linear(agent, *cli_type, true);
                let linear = code_run.spec.linear_integration.as_ref();

                assert!(
                    linear.is_some() && linear.unwrap().enabled,
                    "Combination {agent} + {cli_type:?} should support sidecar"
                );
                tested_combinations += 1;
            }
        }

        // Verify we tested all 12 agents × 6 CLIs = 72 combinations
        assert_eq!(
            tested_combinations, 72,
            "Should test all 72 agent×CLI combinations"
        );
    }

    // =========================================================================
    // Docker-in-Docker (DinD) Sidecar Tests
    // =========================================================================

    /// Verify Docker is enabled by default (matching production behavior)
    #[test]
    fn test_docker_enabled_by_default() {
        let code_run = create_test_code_run_with_linear("5DLabs-Rex", CLIType::Claude, false);
        assert!(
            code_run.spec.enable_docker,
            "Docker should be enabled by default"
        );
    }

    /// Verify Docker is enabled for ALL implementation agents
    #[test]
    fn test_docker_enabled_for_all_implementation_agents() {
        let agents = vec![
            "5DLabs-Rex",   // Rust
            "5DLabs-Blaze", // React
            "5DLabs-Grizz", // Go
            "5DLabs-Nova",  // Node.js
            "5DLabs-Tap",   // Expo
            "5DLabs-Spark", // Electron
        ];

        for agent in agents {
            let code_run = create_test_code_run_with_linear(agent, CLIType::Claude, false);
            assert!(
                code_run.spec.enable_docker,
                "Implementation agent {agent} should have Docker enabled by default"
            );
        }
    }

    /// Verify Docker is enabled for ALL support agents
    #[test]
    fn test_docker_enabled_for_all_support_agents() {
        let agents = vec![
            "5DLabs-Cleo",   // Quality
            "5DLabs-Cipher", // Security
            "5DLabs-Tess",   // Testing
            "5DLabs-Atlas",  // Integration
            "5DLabs-Bolt",   // Infrastructure
            "5DLabs-Morgan", // PM/Docs
        ];

        for agent in agents {
            let code_run = create_test_code_run_with_linear(agent, CLIType::Claude, false);
            assert!(
                code_run.spec.enable_docker,
                "Support agent {agent} should have Docker enabled by default"
            );
        }
    }

    /// Verify Docker is enabled for ALL CLI types
    #[test]
    fn test_docker_enabled_for_all_cli_types() {
        let cli_types = vec![
            CLIType::Claude,
            CLIType::Codex,
            CLIType::Cursor,
            CLIType::Factory,
            CLIType::Gemini,
            CLIType::OpenCode,
        ];

        for cli_type in cli_types {
            let code_run = create_test_code_run_with_linear("5DLabs-Rex", cli_type, false);
            assert!(
                code_run.spec.enable_docker,
                "CLI {cli_type:?} should have Docker enabled by default"
            );
        }
    }

    /// Verify the complete agent × CLI matrix for Docker enablement
    #[test]
    fn test_docker_agent_cli_matrix() {
        let agents = vec![
            "5DLabs-Rex",
            "5DLabs-Blaze",
            "5DLabs-Grizz",
            "5DLabs-Nova",
            "5DLabs-Tap",
            "5DLabs-Spark",
            "5DLabs-Cleo",
            "5DLabs-Cipher",
            "5DLabs-Tess",
            "5DLabs-Atlas",
            "5DLabs-Bolt",
            "5DLabs-Morgan",
        ];

        let cli_types = vec![
            CLIType::Claude,
            CLIType::Codex,
            CLIType::Cursor,
            CLIType::Factory,
            CLIType::Gemini,
            CLIType::OpenCode,
        ];

        let mut tested_combinations = 0;

        for agent in &agents {
            for cli_type in &cli_types {
                let code_run = create_test_code_run_with_linear(agent, *cli_type, false);
                assert!(
                    code_run.spec.enable_docker,
                    "Combination {agent} + {cli_type:?} should have Docker enabled by default"
                );
                tested_combinations += 1;
            }
        }

        // Verify we tested all 12 agents × 6 CLIs = 72 combinations
        assert_eq!(
            tested_combinations, 72,
            "Should test all 72 agent×CLI combinations for Docker"
        );
    }

    /// Verify both Docker and Linear sidecar can be enabled together
    #[test]
    fn test_docker_and_linear_sidecar_together() {
        let code_run = create_test_code_run_with_linear("5DLabs-Rex", CLIType::Claude, true);

        // Both should be enabled
        assert!(
            code_run.spec.enable_docker,
            "Docker should be enabled by default"
        );
        assert!(
            code_run.spec.linear_integration.is_some(),
            "Linear integration should be present"
        );
        assert!(
            code_run.spec.linear_integration.as_ref().unwrap().enabled,
            "Linear integration should be enabled"
        );
    }

    /// Verify the full container matrix for a Play workflow CodeRun
    /// A typical Play workflow has: Main Agent + Docker Daemon + Linear Sidecar
    #[test]
    fn test_full_container_matrix_play_workflow() {
        let agents = vec![
            "5DLabs-Rex",
            "5DLabs-Blaze",
            "5DLabs-Grizz",
            "5DLabs-Nova",
            "5DLabs-Tap",
            "5DLabs-Spark",
            "5DLabs-Cleo",
            "5DLabs-Cipher",
            "5DLabs-Tess",
            "5DLabs-Atlas",
            "5DLabs-Bolt",
            "5DLabs-Morgan",
        ];

        let cli_types = vec![
            CLIType::Claude,
            CLIType::Codex,
            CLIType::Cursor,
            CLIType::Factory,
            CLIType::Gemini,
            CLIType::OpenCode,
        ];

        let mut tested = 0;

        for agent in &agents {
            for cli_type in &cli_types {
                // Create CodeRun with both Docker and Linear enabled (like Play workflow)
                let code_run = create_test_code_run_with_linear(agent, *cli_type, true);

                // Verify Docker daemon sidecar will be added
                assert!(
                    code_run.spec.enable_docker,
                    "{agent} + {cli_type:?}: Docker daemon should be enabled"
                );

                // Verify Linear sidecar will be added
                let linear = code_run.spec.linear_integration.as_ref();
                assert!(
                    linear.is_some() && linear.unwrap().enabled,
                    "{agent} + {cli_type:?}: Linear sidecar should be enabled"
                );

                tested += 1;
            }
        }

        // All 72 combinations should have both sidecars enabled
        assert_eq!(tested, 72, "Should verify all 72 combinations");
    }

    #[test]
    fn merge_cli_config_adds_missing_fields_and_reasoning_effort() {
        let mut existing = cli_config_with_settings(HashMap::new());

        let mut defaults_settings = HashMap::new();
        defaults_settings.insert("reasoningEffort".to_string(), json!("high"));
        defaults_settings.insert("approvalPolicy".to_string(), json!("never"));

        let defaults = CLIConfig {
            cli_type: CLIType::Codex,
            model: "gpt-5-codex".to_string(),
            settings: defaults_settings,
            max_tokens: Some(16_000),
            temperature: Some(0.7_f32),
            model_rotation: None,
            provider: None,
            provider_base_url: None,
            api_key_env_var: None,
        };

        CodeResourceManager::merge_cli_config(&mut existing, &defaults);

        assert_eq!(existing.model, "gpt-5-codex");
        assert_eq!(existing.max_tokens, Some(16_000));
        assert_eq!(existing.temperature, Some(0.7_f32));
        assert_eq!(
            existing.settings.get("reasoningEffort"),
            Some(&json!("high"))
        );
        assert_eq!(
            existing.settings.get("approvalPolicy"),
            Some(&json!("never"))
        );
    }

    #[test]
    fn merge_cli_config_preserves_existing_values() {
        let mut existing_settings = HashMap::new();
        existing_settings.insert("reasoningEffort".to_string(), json!("medium"));

        let mut existing = CLIConfig {
            cli_type: CLIType::Codex,
            model: "custom-model".to_string(),
            settings: existing_settings,
            max_tokens: Some(8_192),
            temperature: Some(0.3_f32),
            model_rotation: None,
            provider: None,
            provider_base_url: None,
            api_key_env_var: None,
        };

        let mut defaults_settings = HashMap::new();
        defaults_settings.insert("reasoningEffort".to_string(), json!("high"));

        let defaults = CLIConfig {
            cli_type: CLIType::Codex,
            model: "gpt-5-codex".to_string(),
            settings: defaults_settings,
            max_tokens: Some(16_000),
            temperature: Some(0.9_f32),
            model_rotation: None,
            provider: None,
            provider_base_url: None,
            api_key_env_var: None,
        };

        CodeResourceManager::merge_cli_config(&mut existing, &defaults);

        assert_eq!(existing.model, "custom-model");
        assert_eq!(existing.max_tokens, Some(8_192));
        assert_eq!(existing.temperature, Some(0.3_f32));
        assert_eq!(
            existing.settings.get("reasoningEffort"),
            Some(&json!("medium"))
        );
    }

    #[test]
    fn merge_cli_config_handles_model_rotation() {
        // Test that model_rotation is merged when None
        let mut existing = cli_config_with_settings(HashMap::new());
        assert!(existing.model_rotation.is_none());

        let defaults = CLIConfig {
            cli_type: CLIType::Codex,
            model: "gpt-5-codex".to_string(),
            settings: HashMap::new(),
            max_tokens: None,
            temperature: None,
            model_rotation: Some(json!(["model1", "model2", "model3"])),
            provider: None,
            provider_base_url: None,
            api_key_env_var: None,
        };

        CodeResourceManager::merge_cli_config(&mut existing, &defaults);

        assert_eq!(
            existing.model_rotation,
            Some(json!(["model1", "model2", "model3"]))
        );

        // Test that existing model_rotation is preserved
        let mut existing_with_rotation = CLIConfig {
            cli_type: CLIType::Codex,
            model: "custom-model".to_string(),
            settings: HashMap::new(),
            max_tokens: None,
            temperature: None,
            model_rotation: Some(json!(["existing-model"])),
            provider: None,
            provider_base_url: None,
            api_key_env_var: None,
        };

        CodeResourceManager::merge_cli_config(&mut existing_with_rotation, &defaults);

        assert_eq!(
            existing_with_rotation.model_rotation,
            Some(json!(["existing-model"]))
        );
    }

    // ==========================================================================
    // Linear Session Labels Tests (for pod discovery)
    // ==========================================================================

    /// Verify that Linear session labels are added to pods when Linear integration is enabled.
    /// These labels are used by PM server to discover and route messages to running agents.
    #[test]
    fn test_linear_session_labels_added_when_enabled() {
        let code_run = create_test_code_run_with_linear("5DLabs-Rex", CLIType::Claude, true);

        // Create labels using the same function the controller uses
        let labels = CodeResourceManager::create_task_labels(&code_run);

        // Verify the linear-session label is present
        assert!(
            labels.contains_key("linear-session"),
            "linear-session label should be added when Linear integration is enabled"
        );
        assert_eq!(
            labels.get("linear-session"),
            Some(&"test-session-123".to_string()),
            "linear-session label should contain the session ID"
        );

        // Verify the linear-issue label is present (sanitized to lowercase)
        assert!(
            labels.contains_key("cto.5dlabs.io/linear-issue"),
            "cto.5dlabs.io/linear-issue label should be added when Linear integration is enabled"
        );
        assert_eq!(
            labels.get("cto.5dlabs.io/linear-issue"),
            Some(&"test-456".to_string()), // Note: sanitized to lowercase
            "linear-issue label should contain the sanitized issue ID"
        );

        // Verify agent-type label is present
        assert!(
            labels.contains_key("cto.5dlabs.io/agent-type"),
            "cto.5dlabs.io/agent-type label should be added"
        );
        assert_eq!(
            labels.get("cto.5dlabs.io/agent-type"),
            Some(&"rex".to_string()),
            "agent-type label should contain the agent name in lowercase"
        );
    }

    /// Verify that Linear session labels are NOT added when Linear integration is disabled.
    #[test]
    fn test_linear_session_labels_not_added_when_disabled() {
        let code_run = create_test_code_run_with_linear("5DLabs-Rex", CLIType::Claude, false);

        let labels = CodeResourceManager::create_task_labels(&code_run);

        // Verify no Linear-related labels are present
        assert!(
            !labels.contains_key("linear-session"),
            "linear-session label should NOT be added when Linear integration is disabled"
        );
        assert!(
            !labels.contains_key("cto.5dlabs.io/linear-issue"),
            "linear-issue label should NOT be added when Linear integration is disabled"
        );
    }

    /// Verify Linear labels work for all agents (for PM server routing).
    #[test]
    fn test_linear_labels_for_all_agents() {
        let agents_and_expected_types = vec![
            ("5DLabs-Rex", "rex"),
            ("5DLabs-Blaze", "blaze"),
            ("5DLabs-Grizz", "grizz"),
            ("5DLabs-Nova", "nova"),
            ("5DLabs-Cleo", "cleo"),
            ("5DLabs-Cipher", "cipher"),
            ("5DLabs-Tess", "tess"),
            ("5DLabs-Atlas", "atlas"),
            ("5DLabs-Bolt", "bolt"),
            ("5DLabs-Morgan", "morgan"),
        ];

        for (agent, expected_type) in agents_and_expected_types {
            let code_run = create_test_code_run_with_linear(agent, CLIType::Claude, true);
            let labels = CodeResourceManager::create_task_labels(&code_run);

            assert!(
                labels.contains_key("linear-session"),
                "Agent {agent} should have linear-session label"
            );
            assert_eq!(
                labels.get("cto.5dlabs.io/agent-type"),
                Some(&expected_type.to_string()),
                "Agent {agent} should have agent-type={expected_type}"
            );
        }
    }

    /// End-to-end test: Verify the complete label set for a typical Play workflow CodeRun.
    /// This simulates what PM server will see when looking up pods.
    #[test]
    fn test_play_workflow_labels_end_to_end() {
        // Simulate a typical Play workflow CodeRun
        let code_run = create_test_code_run_with_linear("5DLabs-Rex", CLIType::Claude, true);
        let labels = CodeResourceManager::create_task_labels(&code_run);

        // These labels are required for PM server pod discovery:
        assert!(
            labels.contains_key("linear-session"),
            "PM server needs linear-session for routing"
        );
        assert!(
            labels.contains_key("cto.5dlabs.io/linear-issue"),
            "PM server needs linear-issue for issue-based lookup"
        );
        assert!(
            labels.contains_key("cto.5dlabs.io/agent-type"),
            "Observability needs agent-type"
        );

        // Standard labels should also be present
        assert_eq!(labels.get("app"), Some(&"controller".to_string()));
        assert_eq!(labels.get("component"), Some(&"code-runner".to_string()));
        assert_eq!(labels.get("job-type"), Some(&"code".to_string()));

        // CLI labels
        assert!(labels.contains_key("cli-type"));
        assert!(labels.contains_key("cli-container"));
    }
}
