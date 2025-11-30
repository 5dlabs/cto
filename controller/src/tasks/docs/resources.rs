// Allow deprecated DocsRun usage for backwards compatibility
#![allow(deprecated)]

use crate::crds::DocsRun;
use crate::tasks::cleanup::{
    LABEL_CLEANUP_KIND, LABEL_CLEANUP_RUN, LABEL_CLEANUP_SCOPE, SCOPE_RUN,
};
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::{github_app_secret_name, ssh_secret_name, Context, Result};
use k8s_openapi::api::{batch::v1::Job, core::v1::ConfigMap};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::ResourceExt;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{error, info};

pub struct DocsResourceManager<'a> {
    pub jobs: &'a Api<Job>,
    pub configmaps: &'a Api<ConfigMap>,
    pub config: &'a Arc<ControllerConfig>,
    pub ctx: &'a Arc<Context>,
}

impl<'a> DocsResourceManager<'a> {
    #[must_use]
    pub fn new(
        jobs: &'a Api<Job>,
        configmaps: &'a Api<ConfigMap>,
        config: &'a Arc<ControllerConfig>,
        ctx: &'a Arc<Context>,
    ) -> Self {
        Self {
            jobs,
            configmaps,
            config,
            ctx,
        }
    }

    /// Sanitize a directory name for use in Kubernetes resource names.
    /// Returns "default" if the input is empty or becomes empty after sanitization.
    fn sanitize_directory_name(directory: &str) -> String {
        if directory.is_empty() {
            return "default".to_string();
        }

        let sanitized = directory
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_lowercase();

        if sanitized.is_empty() {
            "default".to_string()
        } else {
            sanitized
        }
    }

    pub async fn reconcile_create_or_update(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!(
            "🚀 RESOURCE_MANAGER: Starting reconcile_create_or_update for: {}",
            name
        );

        // Don't cleanup resources at start - let idempotent creation handle it
        info!("🔄 RESOURCE_MANAGER: Using idempotent resource creation (no aggressive cleanup)");

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = Self::generate_configmap_name(docs_run);
        info!("📝 RESOURCE_MANAGER: Generated ConfigMap name: {}", cm_name);

        info!("🏗️ RESOURCE_MANAGER: Creating ConfigMap object");
        let configmap = match self.create_configmap(docs_run, &cm_name, None) {
            Ok(cm) => {
                info!("✅ RESOURCE_MANAGER: ConfigMap object created successfully");
                cm
            }
            Err(e) => {
                error!(
                    "❌ RESOURCE_MANAGER: Failed to create ConfigMap object: {:?}",
                    e
                );
                error!(
                    "❌ RESOURCE_MANAGER: Error type: {}",
                    std::any::type_name_of_val(&e)
                );
                return Err(e);
            }
        };

        // Always create or update ConfigMap to ensure latest template content
        info!(
            "🔄 RESOURCE_MANAGER: Attempting to create ConfigMap: {}",
            cm_name
        );
        error!(
            "📝 RESOURCE_MANAGER: Attempting to create ConfigMap: {}",
            cm_name
        );
        match self
            .configmaps
            .create(&PostParams::default(), &configmap)
            .await
        {
            Ok(_) => {
                error!(
                    "✅ RESOURCE_MANAGER: Successfully created ConfigMap: {}",
                    cm_name
                );
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                // ConfigMap exists, update it with latest content
                error!("🔄 RESOURCE_MANAGER: ConfigMap {} already exists (409), attempting to update with latest content", cm_name);

                // First get the existing ConfigMap to preserve resourceVersion
                match self.configmaps.get(&cm_name).await {
                    Ok(existing_cm) => {
                        let mut updated_configmap = configmap;
                        updated_configmap.metadata.resource_version =
                            existing_cm.metadata.resource_version;

                        match self
                            .configmaps
                            .replace(&cm_name, &PostParams::default(), &updated_configmap)
                            .await
                        {
                            Ok(_) => {
                                error!("✅ RESOURCE_MANAGER: Successfully updated existing ConfigMap: {}", cm_name);
                            }
                            Err(e) => {
                                error!("❌ RESOURCE_MANAGER: Failed to replace existing ConfigMap {}: {:?}", cm_name, e);
                                error!(
                                    "❌ RESOURCE_MANAGER: Replace error type: {}",
                                    std::any::type_name_of_val(&e)
                                );

                                // Fall back to creating a new one with a different name
                                error!("🔄 RESOURCE_MANAGER: Replace failed, falling back to create-only approach");
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ RESOURCE_MANAGER: Failed to get existing ConfigMap {} for update: {:?}", cm_name, e);
                        error!(
                            "🔄 RESOURCE_MANAGER: Get failed, falling back to create-only approach"
                        );
                    }
                }
            }
            Err(e) => {
                error!(
                    "❌ RESOURCE_MANAGER: Failed to create ConfigMap {}: {:?}",
                    cm_name, e
                );
                error!(
                    "❌ RESOURCE_MANAGER: Kubernetes error type: {}",
                    std::any::type_name_of_val(&e)
                );
                return Err(e.into());
            }
        }

        // Create Job using idempotent creation (now it can successfully mount the existing ConfigMap)
        let job_ref = self.create_or_get_job(docs_run, &cm_name).await?;

        // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
        if let Some(owner_ref) = job_ref {
            self.update_configmap_owner(docs_run, &cm_name, owner_ref)
                .await?;
        }

        Ok(Action::await_change())
    }

    pub async fn cleanup_resources(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!("Cleaning up docs resources for: {}", name);

        // Clean up any remaining jobs and configmaps
        self.cleanup_old_jobs(docs_run).await?;
        self.cleanup_old_configmaps(docs_run).await?;

        Ok(Action::await_change())
    }

    fn generate_configmap_name(docs_run: &DocsRun) -> String {
        // Generate unique ConfigMap name per DocsRun to prevent conflicts between sequential jobs
        let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = docs_run
            .metadata
            .uid
            .as_deref()
            .map_or("nouid", |uid| &uid[..8]);
        let context_version = 1; // Docs don't have context versions, always 1

        // Use deterministic naming based on DocsRun UID for stable references
        format!("docs-{namespace}-{name}-{uid_suffix}-v{context_version}-files")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    fn create_configmap(
        &self,
        docs_run: &DocsRun,
        name: &str,
        owner_ref: Option<OwnerReference>,
    ) -> Result<ConfigMap> {
        let mut data = BTreeMap::new();

        // Generate all templates for docs
        error!(
            "🔧 RESOURCE_MANAGER: Generating templates for ConfigMap: {}",
            name
        );
        let templates = match super::templates::DocsTemplateGenerator::generate_all_templates(
            docs_run,
            self.config,
        ) {
            Ok(tmpl) => {
                error!(
                    "✅ RESOURCE_MANAGER: Successfully generated {} templates",
                    tmpl.len()
                );
                for filename in tmpl.keys() {
                    error!("📄 RESOURCE_MANAGER: Generated template file: {}", filename);
                }
                tmpl
            }
            Err(e) => {
                error!("❌ RESOURCE_MANAGER: Failed to generate templates: {:?}", e);
                error!(
                    "❌ RESOURCE_MANAGER: Template error type: {}",
                    std::any::type_name_of_val(&e)
                );
                error!("❌ RESOURCE_MANAGER: Template error details: {}", e);
                return Err(e);
            }
        };

        for (filename, content) in templates {
            data.insert(filename, content);
        }

        error!(
            "🏷️ RESOURCE_MANAGER: Creating labels for ConfigMap: {}",
            name
        );
        let labels = Self::create_task_labels(docs_run);
        error!("✅ RESOURCE_MANAGER: Created {} labels", labels.len());

        error!("📝 RESOURCE_MANAGER: Building ConfigMap metadata");
        let mut metadata = ObjectMeta {
            name: Some(name.to_string()),
            labels: Some(labels),
            ..Default::default()
        };

        if let Some(owner) = owner_ref {
            error!("👤 RESOURCE_MANAGER: Adding owner reference to ConfigMap");
            metadata.owner_references = Some(vec![owner]);
        }

        error!(
            "🏗️ RESOURCE_MANAGER: Constructing final ConfigMap object with {} data entries",
            data.len()
        );
        let configmap = ConfigMap {
            metadata,
            data: Some(data),
            ..Default::default()
        };

        error!("✅ RESOURCE_MANAGER: ConfigMap object created successfully");
        Ok(configmap)
    }

    /// Optimistic job creation: create job directly, handle conflicts gracefully
    async fn create_or_get_job(
        &self,
        docs_run: &DocsRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = Self::generate_job_name(docs_run);

        // FIRST: Check if the job already exists
        match self.jobs.get(&job_name).await {
            Ok(existing_job) => {
                error!(
                    "🔍 RESOURCE_MANAGER: Job {} already exists, checking for active pods",
                    job_name
                );

                // Check if there are any pods for this job (regardless of controller UID)
                // This prevents duplicate pods when controller restarts
                let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(
                    self.ctx.client.clone(),
                    docs_run.metadata.namespace.as_deref().unwrap_or("default"),
                );

                let pod_list = pods
                    .list(&ListParams::default().labels(&format!("job-name={job_name}")))
                    .await?;

                if !pod_list.items.is_empty() {
                    error!(
                        "✅ RESOURCE_MANAGER: Found {} existing pod(s) for job {}, skipping job creation",
                        pod_list.items.len(),
                        job_name
                    );

                    // Job exists with pods, return its owner reference
                    return Ok(Some(OwnerReference {
                        api_version: "batch/v1".to_string(),
                        kind: "Job".to_string(),
                        name: job_name.clone(),
                        uid: existing_job.metadata.uid.unwrap_or_default(),
                        controller: Some(false),
                        block_owner_deletion: Some(true),
                    }));
                }
                error!(
                    "⚠️ RESOURCE_MANAGER: Job {} exists but has no pods, will let Job controller handle it",
                    job_name
                );

                // Job exists but no pods - the Job controller will create them
                return Ok(Some(OwnerReference {
                    api_version: "batch/v1".to_string(),
                    kind: "Job".to_string(),
                    name: job_name.clone(),
                    uid: existing_job.metadata.uid.unwrap_or_default(),
                    controller: Some(false),
                    block_owner_deletion: Some(true),
                }));
            }
            Err(_) => {
                // Job doesn't exist, proceed with creation
                error!(
                    "🎯 RESOURCE_MANAGER: Job {} does not exist, creating new job",
                    job_name
                );
            }
        }

        // OPTIMISTIC APPROACH: Try to create job directly
        match self.create_job(docs_run, cm_name).await {
            Ok(owner_ref) => {
                error!(
                    "✅ RESOURCE_MANAGER: Successfully created new job: {}",
                    job_name
                );
                Ok(owner_ref)
            }
            Err(crate::tasks::types::Error::KubeError(kube::Error::Api(ae))) if ae.code == 409 => {
                // Job was created by another reconciliation loop, get the existing one
                error!("🔄 RESOURCE_MANAGER: Job {} was created concurrently (409 conflict), getting existing job", job_name);
                match self.jobs.get(&job_name).await {
                    Ok(existing_job) => {
                        error!("✅ RESOURCE_MANAGER: Retrieved existing job: {}", job_name);
                        Ok(Some(OwnerReference {
                            api_version: "batch/v1".to_string(),
                            kind: "Job".to_string(),
                            name: job_name,
                            uid: existing_job.metadata.uid.unwrap_or_default(),
                            controller: Some(false),
                            block_owner_deletion: Some(true),
                        }))
                    }
                    Err(e) => {
                        error!("❌ RESOURCE_MANAGER: Failed to get existing job after 409 conflict: {:?}", e);
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                error!(
                    "❌ RESOURCE_MANAGER: Job creation failed with non-conflict error: {:?}",
                    e
                );
                Err(e)
            }
        }
    }

    async fn create_job(
        &self,
        docs_run: &DocsRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = Self::generate_job_name(docs_run);

        // Ensure PVC exists before creating job
        info!(
            "🔧 DocsRun {}: Ensuring PVC exists for job {}",
            docs_run.name_any(),
            job_name
        );
        self.ensure_workspace_pvc(docs_run).await?;
        info!(
            "✅ DocsRun {}: PVC ready for job {}",
            docs_run.name_any(),
            job_name
        );

        info!(
            "🔧 DocsRun {}: Building job spec for {}",
            docs_run.name_any(),
            job_name
        );
        let job = self.build_job_spec(docs_run, &job_name, cm_name)?;
        info!(
            "✅ DocsRun {}: Job spec built for {}",
            docs_run.name_any(),
            job_name
        );

        info!(
            "🔧 DocsRun {}: Creating job {}",
            docs_run.name_any(),
            job_name
        );
        let created_job = self.jobs.create(&PostParams::default(), &job).await?;
        info!(
            "✅ DocsRun {}: Job created successfully: {}",
            docs_run.name_any(),
            job_name
        );

        info!("✅ RESOURCE_MANAGER: Created docs job: {}", job_name);

        // Update status using legacy status manager if needed
        if let Err(e) = super::status::DocsStatusManager::update_job_started(
            &Arc::new(docs_run.clone()),
            self.ctx,
            &job_name,
            cm_name,
        )
        .await
        {
            error!(
                "⚠️ RESOURCE_MANAGER: Failed to update job started status: {:?}",
                e
            );
            // Continue anyway, status will be updated by main controller
        }

        // Return owner reference for the created job
        if let (Some(uid), Some(name)) = (created_job.metadata.uid, created_job.metadata.name) {
            Ok(Some(OwnerReference {
                api_version: "batch/v1".to_string(),
                kind: "Job".to_string(),
                name,
                uid,
                controller: Some(true),
                block_owner_deletion: Some(true),
            }))
        } else {
            error!("⚠️ RESOURCE_MANAGER: Created job missing UID or name metadata");
            Ok(None)
        }
    }

    fn generate_job_name(docs_run: &DocsRun) -> String {
        // Use deterministic naming based on the DocsRun's actual name and UID
        // This ensures the same DocsRun always generates the same Job name
        let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = docs_run
            .metadata
            .uid
            .as_deref()
            .map_or("nouid", |uid| &uid[..8]);

        format!("docs-{namespace}-{name}-{uid_suffix}")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    #[allow(clippy::too_many_lines)]
    fn build_job_spec(&self, docs_run: &DocsRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = Self::create_task_labels(docs_run);

        // Create owner reference to DocsRun for proper event handling
        let owner_ref = OwnerReference {
            api_version: "agents.platform/v1".to_string(),
            kind: "DocsRun".to_string(),
            name: docs_run.name_any(),
            uid: docs_run.metadata.uid.clone().unwrap_or_default(),
            controller: Some(true),
            block_owner_deletion: Some(true),
        };

        // Build volumes for docs (emptyDir, no PVCs)
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

        // Agent templates volume for Claude docs templates
        let agent_templates_cm_name = format!("{cm_prefix}-agent-templates-claude-docs");
        volumes.push(json!({
            "name": "agent-templates",
            "configMap": {
                "name": agent_templates_cm_name
            }
        }));
        volume_mounts.push(json!({
            "name": "agent-templates",
            "mountPath": "/agent-templates"
        }));

        // Mount settings.json as managed-settings.json for enterprise compatibility
        volume_mounts.push(json!({
            "name": "task-files",
            "mountPath": "/etc/claude-code/managed-settings.json",
            "subPath": "settings.json"
        }));

        // Persistent workspace volume for docs to prevent data loss
        // Create a project-specific PVC name to avoid cross-project data contamination
        let repo_slug = docs_run
            .spec
            .repository_url
            .trim_start_matches("https://github.com/")
            .trim_end_matches(".git")
            .replace('/', "-");

        let pvc_name = format!(
            "docs-workspace-{}-{}",
            repo_slug
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                })
                .collect::<String>()
                .trim_matches('-')
                .to_lowercase(),
            Self::sanitize_directory_name(&docs_run.spec.working_directory)
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

        // SSH volumes
        let ssh_volumes = Self::generate_ssh_volumes(docs_run);
        volumes.extend(ssh_volumes.volumes);
        volume_mounts.extend(ssh_volumes.volume_mounts);

        let image = format!(
            "{}:{}",
            self.config.agent.image.repository, self.config.agent.image.tag
        );

        // Build primary docs container spec
        let containers = vec![json!({
            "name": "claude-docs",
            "image": image,
            "env": [
                {
                    "name": "GITHUB_APP_PRIVATE_KEY",
                    "valueFrom": { "secretKeyRef": { "name": github_app_secret_name(docs_run.spec.github_app.as_deref().or(docs_run.spec.github_user.as_deref()).unwrap_or("")), "key": "private-key" } }
                },
                {
                    "name": "GITHUB_APP_ID",
                    "valueFrom": { "secretKeyRef": { "name": github_app_secret_name(docs_run.spec.github_app.as_deref().or(docs_run.spec.github_user.as_deref()).unwrap_or("")), "key": "app-id" } }
                },
                {
                    "name": "ANTHROPIC_API_KEY",
                    "valueFrom": { "secretKeyRef": { "name": self.config.secrets.api_key_secret_name, "key": self.config.secrets.api_key_secret_key } }
                }
            ],
            "command": ["/bin/bash"],
            "args": ["/task-files/container.sh"],
            "workingDir": "/workspace",
            "volumeMounts": volume_mounts
        })];

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
                    "metadata": {
                        "labels": labels
                    },
                    "spec": {
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
                    }
                }
            }
        });

        Ok(serde_json::from_value(job_spec)?)
    }

    fn create_task_labels(docs_run: &DocsRun) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();

        // Update legacy orchestrator label to controller
        labels.insert("app".to_string(), "controller".to_string());
        labels.insert("component".to_string(), "docs-generator".to_string());

        labels.insert(LABEL_CLEANUP_SCOPE.to_string(), SCOPE_RUN.to_string());
        labels.insert(LABEL_CLEANUP_KIND.to_string(), "docsrun".to_string());
        if let Some(name) = docs_run.metadata.name.as_deref() {
            labels.insert(
                LABEL_CLEANUP_RUN.to_string(),
                Self::sanitize_label_value(name),
            );
        }

        // Project identification labels
        labels.insert("job-type".to_string(), "docs".to_string());

        // Use working_directory as project name (it's the most meaningful identifier)
        labels.insert(
            "project-name".to_string(),
            Self::sanitize_label_value(&docs_run.spec.working_directory),
        );

        // Use github_app if available, fallback to github_user for backward compatibility
        let github_identity = docs_run
            .spec
            .github_app
            .as_deref()
            .or(docs_run.spec.github_user.as_deref())
            .unwrap_or("");
        labels.insert(
            "github-identity".to_string(),
            Self::sanitize_label_value(github_identity),
        );
        labels.insert("context-version".to_string(), "1".to_string()); // Docs always version 1

        // Docs-specific labels
        labels.insert("task-type".to_string(), "docs".to_string());
        labels.insert(
            "repository".to_string(),
            Self::sanitize_label_value(&docs_run.spec.repository_url),
        );

        labels
    }

    fn generate_ssh_volumes(docs_run: &DocsRun) -> SshVolumes {
        // Only mount SSH keys when using github_user authentication (not GitHub Apps)
        if docs_run.spec.github_app.is_some() || docs_run.spec.github_user.is_none() {
            // GitHub App authentication doesn't need SSH keys
            return SshVolumes {
                volumes: vec![],
                volume_mounts: vec![],
            };
        }

        let ssh_secret = ssh_secret_name(docs_run.spec.github_user.as_deref().unwrap_or(""));

        let volumes = vec![json!({
            "name": "ssh-key",
            "secret": {
                "secretName": ssh_secret,
                "defaultMode": 0o644,
                "items": [{
                    "key": "ssh-privatekey",
                    "path": "id_ed25519"
                }]
            }
        })];

        let volume_mounts = vec![json!({
            "name": "ssh-key",
            "mountPath": "/workspace/.ssh",
            "readOnly": true
        })];

        SshVolumes {
            volumes,
            volume_mounts,
        }
    }

    async fn update_configmap_owner(
        &self,
        _docs_run: &DocsRun,
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
    async fn cleanup_old_jobs(&self, docs_run: &DocsRun) -> Result<()> {
        let github_identity = docs_run
            .spec
            .github_app
            .as_deref()
            .or(docs_run.spec.github_user.as_deref())
            .unwrap_or("");
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=docs-generator,github-identity={}",
            Self::sanitize_label_value(github_identity)
        ));

        let jobs = self.jobs.list(&list_params).await?;

        for job in jobs {
            if let Some(job_name) = job.metadata.name {
                info!("Deleting old docs job: {}", job_name);
                let _ = self.jobs.delete(&job_name, &DeleteParams::default()).await;
            }
        }

        Ok(())
    }

    async fn cleanup_old_configmaps(&self, docs_run: &DocsRun) -> Result<()> {
        // Generate current ConfigMap name to avoid deleting it
        let current_cm_name = Self::generate_configmap_name(docs_run);

        let github_identity = docs_run
            .spec
            .github_app
            .as_deref()
            .or(docs_run.spec.github_user.as_deref())
            .unwrap_or("");
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=docs-generator,github-identity={}",
            Self::sanitize_label_value(github_identity)
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
                let has_active_job = cm.metadata.owner_references.as_ref().is_some_and(|owners| {
                    owners
                        .iter()
                        .any(|owner| owner.kind == "Job" && owner.api_version.starts_with("batch/"))
                });

                if has_active_job {
                    // If ConfigMap is owned by a Job, let Kubernetes handle cleanup when Job completes
                    info!(
                        "Skipping cleanup of ConfigMap with active Job owner: {}",
                        cm_name
                    );
                    continue;
                }

                info!("Deleting old docs ConfigMap: {}", cm_name);
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

    #[allow(clippy::too_many_lines)]
    async fn ensure_workspace_pvc(&self, docs_run: &DocsRun) -> Result<()> {
        // Create a project-specific PVC name to avoid cross-project data contamination
        let repo_slug = docs_run
            .spec
            .repository_url
            .trim_start_matches("https://github.com/")
            .trim_end_matches(".git")
            .replace('/', "-");

        let pvc_name = format!(
            "docs-workspace-{}-{}",
            repo_slug
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                })
                .collect::<String>()
                .trim_matches('-')
                .to_lowercase(),
            Self::sanitize_directory_name(&docs_run.spec.working_directory)
        );

        // Check if PVC already exists
        let pvcs: Api<k8s_openapi::api::core::v1::PersistentVolumeClaim> =
            Api::namespaced(self.ctx.client.clone(), &self.ctx.namespace);

        info!(
            "🔍 DocsRun {}: Checking if PVC {} exists",
            docs_run.name_any(),
            pvc_name
        );
        match pvcs.get(&pvc_name).await {
            Ok(_) => {
                info!(
                    "✅ DocsRun {}: PVC {} already exists",
                    docs_run.name_any(),
                    pvc_name
                );
                return Ok(());
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                // PVC doesn't exist, create it
                info!(
                    "📦 DocsRun {}: Creating PVC: {}",
                    docs_run.name_any(),
                    pvc_name
                );
            }
            Err(e) => {
                error!(
                    "❌ DocsRun {}: Failed to check PVC {}: {}",
                    docs_run.name_any(),
                    pvc_name,
                    e
                );
                return Err(e.into());
            }
        }

        // Create PVC
        let pvc = k8s_openapi::api::core::v1::PersistentVolumeClaim {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(pvc_name.clone()),
                namespace: Some(self.ctx.namespace.clone()),
                labels: Some({
                    let mut labels = std::collections::BTreeMap::new();
                    labels.insert("app".to_string(), "controller".to_string());
                    labels.insert("component".to_string(), "docs-workspace".to_string());
                    labels.insert(
                        "working-directory".to_string(),
                        Self::sanitize_label_value(&docs_run.spec.working_directory),
                    );
                    labels
                }),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                resources: Some(k8s_openapi::api::core::v1::VolumeResourceRequirements {
                    requests: Some({
                        let mut requests = std::collections::BTreeMap::new();
                        requests.insert(
                            "storage".to_string(),
                            k8s_openapi::apimachinery::pkg::api::resource::Quantity(
                                "5Gi".to_string(),
                            ),
                        );
                        requests
                    }),
                    ..Default::default()
                }),
                storage_class_name: Some("local-path".to_string()), // Talos default
                ..Default::default()
            }),
            ..Default::default()
        };

        info!(
            "🔧 DocsRun {}: Attempting to create PVC {}",
            docs_run.name_any(),
            pvc_name
        );
        match pvcs.create(&kube::api::PostParams::default(), &pvc).await {
            Ok(_) => {
                info!(
                    "✅ DocsRun {}: Created PVC: {}",
                    docs_run.name_any(),
                    pvc_name
                );
                Ok(())
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                info!(
                    "✅ DocsRun {}: PVC {} already exists (created concurrently)",
                    docs_run.name_any(),
                    pvc_name
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "❌ DocsRun {}: Failed to create PVC {}: {:?}",
                    docs_run.name_any(),
                    pvc_name,
                    e
                );
                Err(e.into())
            }
        }
    }
}

struct SshVolumes {
    volumes: Vec<serde_json::Value>,
    volume_mounts: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_directory_name() {
        // Test empty string
        assert_eq!(DocsResourceManager::sanitize_directory_name(""), "default");

        // Test normal directory name
        assert_eq!(DocsResourceManager::sanitize_directory_name("docs"), "docs");

        // Test directory with special characters
        assert_eq!(
            DocsResourceManager::sanitize_directory_name("my/project"),
            "my-project"
        );

        // Test directory that becomes empty after sanitization
        assert_eq!(
            DocsResourceManager::sanitize_directory_name("///"),
            "default"
        );

        // Test directory with mixed characters
        assert_eq!(
            DocsResourceManager::sanitize_directory_name("docs@2024!"),
            "docs-2024"
        );
    }

    #[test]
    fn test_backward_compatibility_scenarios() {
        // Test that new function returns "default" for empty string
        assert_eq!(DocsResourceManager::sanitize_directory_name(""), "default");

        // Simulate what the old function would have returned for empty string
        let legacy_result = ""
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_lowercase();
        assert_eq!(legacy_result, "");

        // Test that both give same result for non-empty strings
        assert_eq!(DocsResourceManager::sanitize_directory_name("docs"), "docs");

        let legacy_result_docs = "docs"
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_lowercase();
        assert_eq!(legacy_result_docs, "docs");

        // Test that this demonstrates the backward compatibility issue
        // Old: empty working_directory would result in PVC name ending with "-" (empty suffix)
        // New: empty working_directory results in PVC name ending with "-default"
        let old_pvc_name = format!("docs-workspace-repo-{legacy_result}");
        let new_pvc_name = format!(
            "docs-workspace-repo-{}",
            DocsResourceManager::sanitize_directory_name("")
        );
        assert_eq!(old_pvc_name, "docs-workspace-repo-");
        assert_eq!(new_pvc_name, "docs-workspace-repo-default");
        assert_ne!(old_pvc_name, new_pvc_name);
    }
}
