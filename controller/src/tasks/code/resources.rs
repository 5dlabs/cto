use super::agent::AgentClassifier;
use super::naming::ResourceNaming;
use crate::cli::types::CLIType;
use crate::crds::{CLIConfig, CodeRun};
use crate::tasks::config::{ControllerConfig, ResolvedSecretBinding};
use crate::tasks::types::{github_app_secret_name, Context, Error, Result};
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim, Service},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::ResourceExt;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tracing::{error, info};

pub struct CodeResourceManager<'a> {
    pub jobs: &'a Api<Job>,
    pub configmaps: &'a Api<ConfigMap>,
    pub pvcs: &'a Api<PersistentVolumeClaim>,
    pub config: &'a Arc<ControllerConfig>,
    pub ctx: &'a Arc<Context>,
}

impl<'a> CodeResourceManager<'a> {
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

    pub async fn reconcile_create_or_update(&self, code_run: &Arc<CodeRun>) -> Result<Action> {
        let name = code_run.name_any();
        info!("🚀 Creating/updating code resources for: {}", name);

        // STEP: Auto-populate CLI config based on agent (if not already specified)
        let code_run = self.populate_cli_config_if_needed(code_run).await?;
        let code_run_ref = &*code_run;

        // Determine PVC name based on agent classification
        let service_name = &code_run_ref.spec.service;
        let classifier = AgentClassifier::new();

        // Get the appropriate PVC name based on agent type
        let pvc_name = if let Some(github_app) = &code_run_ref.spec.github_app {
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

        info!("📦 Ensuring PVC exists: {}", pvc_name);
        self.ensure_pvc_exists(
            &pvc_name,
            service_name,
            code_run_ref.spec.github_app.as_deref(),
        )
        .await?;
        info!("✅ PVC check completed");

        // Don't cleanup resources at start - let idempotent creation handle it
        info!("🔄 Using idempotent resource creation (no aggressive cleanup)");

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = self.generate_configmap_name(code_run_ref);
        info!("📄 Generated ConfigMap name: {}", cm_name);

        info!("🔧 Creating ConfigMap template data...");
        let configmap = self.create_configmap(code_run_ref, &cm_name, None)?;
        info!("✅ ConfigMap template created successfully");

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

        // Ensure headless Service exists for input bridge discovery
        let cli_type = Self::code_run_cli_type(code_run_ref);

        if self.config.agent.input_bridge.enabled
            && cli_type != CLIType::Codex
            && cli_type != CLIType::Cursor
            && cli_type != CLIType::Factory
        {
            let job_name = self.generate_job_name(code_run_ref);
            self.ensure_headless_service_exists(code_run_ref, &job_name)
                .await?;
        }

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

        Ok(Action::await_change())
    }

    async fn ensure_headless_service_exists(
        &self,
        code_run: &CodeRun,
        job_name: &str,
    ) -> Result<()> {
        let namespace = code_run
            .metadata
            .namespace
            .as_deref()
            .unwrap_or(&self.ctx.namespace);
        let services: Api<Service> = Api::namespaced(self.ctx.client.clone(), namespace);

        let svc_name = ResourceNaming::headless_service_name(job_name);

        // Build labels for metadata and selector
        let mut meta_labels = BTreeMap::new();
        meta_labels.insert("agents.platform/jobType".to_string(), "code".to_string());
        meta_labels.insert("agents.platform/name".to_string(), code_run.name_any());
        meta_labels.insert("agents.platform/input".to_string(), "bridge".to_string());
        meta_labels.insert("agents.platform/owner".to_string(), "CodeRun".to_string());

        // Prefer github_user/app as user label if present
        if let Some(user) = code_run
            .spec
            .github_app
            .as_deref()
            .or(code_run.spec.github_user.as_deref())
        {
            meta_labels.insert(
                "agents.platform/user".to_string(),
                self.sanitize_label_value(user),
            );
        }

        let port = self.config.agent.input_bridge.port;

        let svc_json = json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": svc_name,
                "labels": meta_labels
            },
            "spec": {
                "clusterIP": "None",
                "ports": [{ "name": "http", "port": port, "targetPort": port }],
                "selector": { "job-name": job_name }
            }
        });

        match services
            .create(
                &PostParams::default(),
                &serde_json::from_value(svc_json.clone())?,
            )
            .await
        {
            Ok(_) => {
                info!("✅ Created headless Service: {}", svc_name);
                Ok(())
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                // Exists: fetch to preserve resourceVersion, then replace
                let existing = services.get(&svc_name).await?;
                let mut updated: k8s_openapi::api::core::v1::Service =
                    serde_json::from_value(svc_json)?;
                updated.metadata.resource_version = existing.metadata.resource_version;

                services
                    .replace(&svc_name, &PostParams::default(), &updated)
                    .await?;
                info!("🔄 Updated headless Service: {}", svc_name);
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
    ) -> Result<()> {
        match self.pvcs.get(pvc_name).await {
            Ok(_) => {
                info!("PVC {} already exists", pvc_name);
                Ok(())
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                info!("Creating PVC: {}", pvc_name);
                let pvc = self.build_pvc_spec(pvc_name, service_name, github_app);
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

    fn generate_configmap_name(&self, code_run: &CodeRun) -> String {
        // Generate unique ConfigMap name per CodeRun to prevent conflicts between sequential jobs
        let namespace = code_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = code_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = code_run
            .metadata
            .uid
            .as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");
        let task_id = code_run.spec.task_id;
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
            super::templates::CodeTemplateGenerator::generate_all_templates(code_run, self.config)?;
        for (filename, content) in templates {
            data.insert(filename, content);
        }

        let labels = self.create_task_labels(code_run);
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
        let job_name = self.generate_job_name(code_run);

        // Try to get existing job first (idempotent check)
        match self.jobs.get(&job_name).await {
            Ok(existing_job) => {
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

                if !pod_list.items.is_empty() {
                    info!(
                        "Found {} existing pod(s) for job {}, using existing job",
                        pod_list.items.len(),
                        job_name
                    );
                } else {
                    info!(
                        "Job {} exists but has no pods, will let Job controller handle it",
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
            }
            Err(_) => {
                // Job doesn't exist, create it
                info!("Job {} doesn't exist, creating it", job_name);
                self.create_job(code_run, cm_name).await
            }
        }
    }

    async fn create_job(
        &self,
        code_run: &CodeRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = self.generate_job_name(code_run);
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

    fn generate_job_name(&self, code_run: &CodeRun) -> String {
        // Use unified naming system to ensure consistency with controller lookups
        ResourceNaming::job_name(code_run)
    }

    fn build_job_spec(&self, code_run: &CodeRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = self.create_task_labels(code_run);

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

        // Agents ConfigMap volume for system prompts
        let agents_cm_name = "controller-agents".to_string();
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

        let cli_type = Self::code_run_cli_type(code_run);

        if cli_type == CLIType::Claude {
            volume_mounts.push(json!({
                "name": "task-files",
                "mountPath": "/etc/claude-code/managed-settings.json",
                "subPath": "settings.json"
            }));
        }

        // PVC workspace volume for code (persistent across sessions)
        // Use conditional naming based on agent classification
        let classifier = AgentClassifier::new();
        let pvc_name = if let Some(github_app) = &code_run.spec.github_app {
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

        // Docker-in-Docker volumes (disabled by default, can be enabled by setting enableDocker: true)
        let enable_docker = code_run.spec.enable_docker.unwrap_or(false);
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

        // GitHub App authentication only - no SSH volumes needed
        let github_app = code_run.spec.github_app.as_ref().ok_or_else(|| {
            tracing::error!("GitHub App is required for CodeRun authentication");
            crate::tasks::types::Error::ConfigError(
                "GitHub App is required for CodeRun authentication".to_string(),
            )
        })?;

        tracing::info!(
            "Using GitHub App authentication for CodeRun: {}",
            github_app
        );

        // Select image based on CLI type (if specified) or fallback to default
        let image = self.select_image_for_cli(code_run)?;

        // Resolve CLI-specific API key binding (env var + secret reference)
        let provider = self
            .config
            .agent
            .cli_providers
            .get(&cli_type.to_string().to_lowercase())
            .map(|value| value.as_str());

        let api_key_binding = self.config.secrets.resolve_cli_binding(&cli_type, provider);
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
        let (mut final_env_vars, env_from) = self.process_task_requirements(code_run, env_vars)?;

        // Critical system variables that must not be overridden
        // Add these AFTER requirements processing to ensure they take precedence
        let critical_env_vars = vec![
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
        ];

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
        if enable_docker {
            final_env_vars.push(json!({
                "name": "DOCKER_HOST",
                "value": "unix:///var/run/docker.sock"
            }));
        }

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

        // Build the job spec with environment configuration
        let mut container_spec = json!({
            "name": "claude-code",
            "image": image,
            "env": final_env_vars,
            "command": ["/bin/bash"],
            "args": ["/task-files/container.sh"],
            "workingDir": "/workspace",
            "volumeMounts": volume_mounts
        });

        // Add envFrom if we have secrets to mount
        if !env_from.is_empty() {
            container_spec["envFrom"] = json!(env_from);
        }

        // Build containers array
        let mut containers = vec![container_spec];

        // Add sidecar for live JSONL input via HTTP and future tools (if enabled)
        if self.config.agent.input_bridge.enabled
            && cli_type != CLIType::Codex
            && cli_type != CLIType::Cursor
            && cli_type != CLIType::Factory
        {
            let input_bridge_image = format!(
                "{}:{}",
                self.config.agent.input_bridge.image.repository,
                self.config.agent.input_bridge.image.tag
            );
            let input_bridge = json!({
                "name": "sidecar",
                "image": input_bridge_image,
                "imagePullPolicy": "Always",
                "env": [
                    {"name": "FIFO_PATH", "value": "/workspace/agent-input.jsonl"},
                    {"name": "PORT", "value": self.config.agent.input_bridge.port.to_string()}
                ],
                "ports": [{"name": "http", "containerPort": self.config.agent.input_bridge.port}],
                "volumeMounts": [
                    {"name": "workspace", "mountPath": "/workspace"}
                ],
                "lifecycle": {
                    "preStop": {
                        "exec": {
                            "command": ["/bin/sh", "-lc", "curl -fsS -X POST http://127.0.0.1:8080/shutdown || true"]
                        }
                    }
                },
                "resources": {
                    "requests": {
                        "cpu": "50m",
                        "memory": "32Mi"
                    },
                    "limits": {
                        "cpu": "100m",
                        "memory": "64Mi"
                    }
                }
            });
            containers.push(input_bridge);
        }

        // Add Docker daemon if enabled (kept as-is for DIND workflows)
        if enable_docker {
            let docker_daemon_spec = json!({
                "name": "docker-daemon",
                "image": "docker:dind",
                "securityContext": {
                    "privileged": true
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
                        "mountPath": "/data"
                    }
                ],
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

        // Build Pod spec and set ServiceAccountName (required by CRD)
        let mut pod_spec = json!({
                        "shareProcessNamespace": true,
                        "restartPolicy": "Never",
                        "securityContext": {
                            "runAsUser": 1000,
                            "runAsGroup": 1000,
                            "fsGroup": 1000,
                            "fsGroupChangePolicy": "OnRootMismatch"
                        },
                        "initContainers": [{
                            "name": "fix-workspace-perms",
                            "image": "busybox:1.36",
                            "command": ["/bin/sh", "-lc", "chown -R 1000:1000 /workspace && chmod -R ug+rwX /workspace || true"],
                            "securityContext": {
                                "runAsUser": 0,
                                "runAsGroup": 0,
                                "allowPrivilegeEscalation": false
                            },
                            "volumeMounts": [ {"name": "workspace", "mountPath": "/workspace"} ]
                        }],
                        "containers": containers,
                        "volumes": volumes
        });

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

        let job_spec = json!({
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
                "template": {
                    "metadata": { "labels": labels },
                    "spec": pod_spec
                }
            }
        });

        Ok(serde_json::from_value(job_spec)?)
    }

    fn process_task_requirements(
        &self,
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
            .map(|r| !r.trim().is_empty())
            .unwrap_or(false);

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

    fn create_task_labels(&self, code_run: &CodeRun) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();

        // Update legacy orchestrator label to controller
        labels.insert("app".to_string(), "controller".to_string());
        labels.insert("component".to_string(), "code-runner".to_string());

        // Project identification labels
        labels.insert("job-type".to_string(), "code".to_string());

        // Use service as project name for code tasks
        labels.insert(
            "project-name".to_string(),
            self.sanitize_label_value(&code_run.spec.service),
        );

        let github_identifier = code_run
            .spec
            .github_app
            .as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");
        labels.insert(
            "github-user".to_string(),
            self.sanitize_label_value(github_identifier),
        );
        labels.insert(
            "context-version".to_string(),
            code_run.spec.context_version.to_string(),
        );

        // Code-specific labels
        labels.insert("task-type".to_string(), "code".to_string());
        labels.insert("task-id".to_string(), code_run.spec.task_id.to_string());
        labels.insert(
            "service".to_string(),
            self.sanitize_label_value(&code_run.spec.service),
        );

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
        let github_identifier = code_run
            .spec
            .github_app
            .as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");
        let list_params = ListParams::default().labels(&format!(
            "app=controller,component=code-runner,github-user={},service={}",
            self.sanitize_label_value(github_identifier),
            self.sanitize_label_value(&code_run.spec.service)
        ));

        let jobs = self.jobs.list(&list_params).await?;

        for job in jobs {
            if let Some(job_name) = job.metadata.name {
                info!("Deleting old code job: {}", job_name);
                let _ = self.jobs.delete(&job_name, &DeleteParams::default()).await;
            }
        }

        Ok(())
    }

    async fn cleanup_old_configmaps(&self, code_run: &CodeRun) -> Result<()> {
        // Generate current ConfigMap name to avoid deleting it
        let current_cm_name = self.generate_configmap_name(code_run);

        let github_identifier = code_run
            .spec
            .github_app
            .as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");
        let list_params = ListParams::default().labels(&format!(
            "app=controller,component=code-runner,github-user={},service={}",
            self.sanitize_label_value(github_identifier),
            self.sanitize_label_value(&code_run.spec.service)
        ));

        let configmaps = self.configmaps.list(&list_params).await?;

        for cm in configmaps {
            if let Some(cm_name) = cm.metadata.name {
                // Skip deleting the current ConfigMap - this prevents deletion of active job's ConfigMap
                if cm_name == current_cm_name {
                    info!("Skipping deletion of current ConfigMap: {}", cm_name);
                    continue;
                }

                // Check if ConfigMap has an owner reference to a Job that's still running
                let has_active_job = cm
                    .metadata
                    .owner_references
                    .as_ref()
                    .map(|owners| {
                        owners.iter().any(|owner| {
                            owner.kind == "Job" && owner.api_version.starts_with("batch/")
                        })
                    })
                    .unwrap_or(false);

                if has_active_job {
                    // If ConfigMap is owned by a Job, let Kubernetes handle cleanup when Job completes
                    info!(
                        "Skipping cleanup of ConfigMap with active Job owner: {}",
                        cm_name
                    );
                    continue;
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

    fn sanitize_label_value(&self, input: &str) -> String {
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

    fn code_run_cli_type(code_run: &CodeRun) -> CLIType {
        code_run
            .spec
            .cli_config
            .as_ref()
            .map(|cfg| cfg.cli_type)
            .unwrap_or(CLIType::Claude)
    }

    /// Select the appropriate Docker image based on the CLI type specified in the CodeRun
    /// Auto-populate CLI config based on agent GitHub app (if not already specified)
    async fn populate_cli_config_if_needed(&self, code_run: &Arc<CodeRun>) -> Result<Arc<CodeRun>> {
        // If we have no GitHub app context, we cannot enrich the CLI config
        let Some(github_app) = &code_run.spec.github_app else {
            if code_run.spec.cli_config.is_none() {
                info!("No CLI config or GitHub app specified, using defaults");
            }
            return Ok(code_run.clone());
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
            return Ok(code_run.clone());
        };

        let mut new_code_run = (**code_run).clone();

        match new_code_run.spec.cli_config.as_mut() {
            Some(existing) => {
                Self::merge_cli_config(existing, agent_cli_config);
                self.apply_cli_provider(existing);
            }
            None => {
                info!(
                    "🔧 Auto-populating CLI config for agent {}: {} ({})",
                    github_app, agent_cli_config.cli_type, agent_cli_config.model
                );
                new_code_run.spec.cli_config = Some(agent_cli_config.clone());
                if let Some(existing) = new_code_run.spec.cli_config.as_mut() {
                    self.apply_cli_provider(existing);
                }
            }
        }

        Ok(Arc::new(new_code_run))
    }

    fn merge_cli_config(existing: &mut CLIConfig, defaults: &CLIConfig) {
        if existing.model.trim().is_empty() {
            existing.model = defaults.model.clone();
        }

        if existing.max_tokens.is_none() {
            existing.max_tokens = defaults.max_tokens;
        }

        if existing.temperature.is_none() {
            existing.temperature = defaults.temperature;
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
        if let Some(provider) = self.config.agent.cli_providers.get(&cli_key) {
            existing
                .settings
                .entry("provider".to_string())
                .or_insert_with(|| serde_json::Value::String(provider.clone()));
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

        // No CLI config specified - use default image (backward compatibility)
        if self.config.agent.image.is_configured() {
            return Ok(format!(
                "{}:{}",
                self.config.agent.image.repository, self.config.agent.image.tag
            ));
        }

        Err(Error::ConfigError(
            "No CLI configuration provided and agent.image fallback is not set.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn cli_config_with_settings(settings: HashMap<String, serde_json::Value>) -> CLIConfig {
        CLIConfig {
            cli_type: CLIType::Codex,
            model: "".to_string(),
            settings,
            max_tokens: None,
            temperature: None,
        }
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
        };

        let mut defaults_settings = HashMap::new();
        defaults_settings.insert("reasoningEffort".to_string(), json!("high"));

        let defaults = CLIConfig {
            cli_type: CLIType::Codex,
            model: "gpt-5-codex".to_string(),
            settings: defaults_settings,
            max_tokens: Some(16_000),
            temperature: Some(0.9_f32),
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
}
