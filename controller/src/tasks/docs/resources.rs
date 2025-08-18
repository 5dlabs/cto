use crate::crds::DocsRun;
use crate::tasks::code::naming::ResourceNaming;
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::{github_app_secret_name, ssh_secret_name, Context, Result};
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, Service},
};
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

    pub async fn reconcile_create_or_update(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!(
            "🚀 RESOURCE_MANAGER: Starting reconcile_create_or_update for: {}",
            name
        );

        // Don't cleanup resources at start - let idempotent creation handle it
        info!("🔄 RESOURCE_MANAGER: Using idempotent resource creation (no aggressive cleanup)");

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = self.generate_configmap_name(docs_run);
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

        // Ensure headless Service exists for input bridge discovery when enabled
        if self.config.agent.input_bridge.enabled {
            let job_name = self.generate_job_name(docs_run);
            self.ensure_headless_service_exists(docs_run, &job_name)
                .await?;
        }

        Ok(Action::await_change())
    }

    async fn ensure_headless_service_exists(
        &self,
        docs_run: &DocsRun,
        job_name: &str,
    ) -> Result<()> {
        let namespace = docs_run
            .metadata
            .namespace
            .as_deref()
            .unwrap_or(&self.ctx.namespace);
        let services: Api<Service> = Api::namespaced(self.ctx.client.clone(), namespace);

        let svc_name = ResourceNaming::headless_service_name(job_name);

        let mut meta_labels = BTreeMap::new();
        meta_labels.insert("agents.platform/jobType".to_string(), "docs".to_string());
        meta_labels.insert("agents.platform/name".to_string(), docs_run.name_any());
        meta_labels.insert("agents.platform/input".to_string(), "bridge".to_string());
        meta_labels.insert("agents.platform/owner".to_string(), "DocsRun".to_string());

        if let Some(user) = docs_run
            .spec
            .github_app
            .as_deref()
            .or(docs_run.spec.github_user.as_deref())
        {
            meta_labels.insert(
                "agents.platform/user".to_string(),
                self.sanitize_label_value(user),
            );
        }

        // Respect configured input bridge port
        let port = self.config.agent.input_bridge.port;
        let svc_json = json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": { "name": svc_name, "labels": meta_labels },
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
            Ok(_) => Ok(()),
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                // Fetch existing to preserve resourceVersion on replace
                let mut existing = services.get(&svc_name).await?;
                if let Some(spec) = existing.spec.as_mut() {
                    spec.cluster_ip = Some("None".to_string());
                    spec.selector = Some(std::collections::BTreeMap::from([(
                        "job-name".to_string(),
                        job_name.to_string(),
                    )]));
                    spec.ports = Some(vec![k8s_openapi::api::core::v1::ServicePort {
                        name: Some("http".to_string()),
                        port: port as i32,
                        target_port: Some(
                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                                port as i32,
                            ),
                        ),
                        ..Default::default()
                    }]);
                }
                let mut updated: k8s_openapi::api::core::v1::Service =
                    serde_json::from_value(svc_json)?;
                updated.metadata.resource_version = existing.metadata.resource_version.take();
                services
                    .replace(&svc_name, &PostParams::default(), &updated)
                    .await?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn cleanup_resources(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!("Cleaning up docs resources for: {}", name);

        // Clean up any remaining jobs and configmaps
        self.cleanup_old_jobs(docs_run).await?;
        self.cleanup_old_configmaps(docs_run).await?;

        Ok(Action::await_change())
    }

    fn generate_configmap_name(&self, docs_run: &DocsRun) -> String {
        // Generate unique ConfigMap name per DocsRun to prevent conflicts between sequential jobs
        let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = docs_run
            .metadata
            .uid
            .as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");
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
        let labels = self.create_task_labels(docs_run);
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
        let job_name = self.generate_job_name(docs_run);

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
                } else {
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
        let job_name = self.generate_job_name(docs_run);

        // Ensure PVC exists before creating job
        self.ensure_workspace_pvc(docs_run).await?;

        let job = self.build_job_spec(docs_run, &job_name, cm_name)?;

        let created_job = self.jobs.create(&PostParams::default(), &job).await?;

        error!("✅ RESOURCE_MANAGER: Created docs job: {}", job_name);

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

    fn generate_job_name(&self, docs_run: &DocsRun) -> String {
        // Use deterministic naming based on the DocsRun's actual name and UID
        // This ensures the same DocsRun always generates the same Job name
        let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = docs_run
            .metadata
            .uid
            .as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");

        format!("docs-{namespace}-{name}-{uid_suffix}")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    fn build_job_spec(&self, docs_run: &DocsRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = self.create_task_labels(docs_run);

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

        // Mount settings.json as managed-settings.json for enterprise compatibility
        volume_mounts.push(json!({
            "name": "task-files",
            "mountPath": "/etc/claude-code/managed-settings.json",
            "subPath": "settings.json"
        }));

        // Persistent workspace volume for docs to prevent data loss
        // Create a PVC name based on the working directory for reuse across jobs
        let pvc_name = format!(
            "docs-workspace-{}",
            docs_run
                .spec
                .working_directory
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                })
                .collect::<String>()
                .trim_matches('-')
                .to_lowercase()
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
        let ssh_volumes = self.generate_ssh_volumes(docs_run);
        volumes.extend(ssh_volumes.volumes);
        volume_mounts.extend(ssh_volumes.volume_mounts);

        let image = format!(
            "{}:{}",
            self.config.agent.image.repository, self.config.agent.image.tag
        );

        // Build primary docs container spec
        let mut containers = vec![json!({
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

        // Optionally add sidecar when enabled (input bridge and future tools)
        if self.config.agent.input_bridge.enabled {
            let input_bridge_image = format!(
                "{}:{}",
                self.config.agent.input_bridge.image.repository,
                self.config.agent.input_bridge.image.tag
            );
            containers.push(json!({
                "name": "sidecar",
                "image": input_bridge_image,
                "imagePullPolicy": "Always",
                "env": [
                    {"name": "FIFO_PATH", "value": "/workspace/agent-input.jsonl"},
                    {"name": "PORT", "value": self.config.agent.input_bridge.port.to_string()}
                ],
                "ports": [{"name": "http", "containerPort": self.config.agent.input_bridge.port}],
                "volumeMounts": [ {"name": "workspace", "mountPath": "/workspace"} ],
                "lifecycle": {
                    "preStop": {
                        "exec": {"command": ["/bin/sh", "-lc", "curl -fsS -X POST http://127.0.0.1:8080/shutdown || true"]}
                    }
                },
                "resources": {
                    "requests": { "cpu": "50m", "memory": "32Mi" },
                    "limits":   { "cpu": "100m", "memory": "64Mi" }
                }
            }));
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

    fn create_task_labels(&self, docs_run: &DocsRun) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();

        // Update legacy orchestrator label to controller
        labels.insert("app".to_string(), "controller".to_string());
        labels.insert("component".to_string(), "docs-generator".to_string());

        // Project identification labels
        labels.insert("job-type".to_string(), "docs".to_string());

        // Use working_directory as project name (it's the most meaningful identifier)
        labels.insert(
            "project-name".to_string(),
            self.sanitize_label_value(&docs_run.spec.working_directory),
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
            self.sanitize_label_value(github_identity),
        );
        labels.insert("context-version".to_string(), "1".to_string()); // Docs always version 1

        // Docs-specific labels
        labels.insert("task-type".to_string(), "docs".to_string());
        labels.insert(
            "repository".to_string(),
            self.sanitize_label_value(&docs_run.spec.repository_url),
        );

        labels
    }

    fn generate_ssh_volumes(&self, docs_run: &DocsRun) -> SshVolumes {
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
            self.sanitize_label_value(github_identity)
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
        let current_cm_name = self.generate_configmap_name(docs_run);

        let github_identity = docs_run
            .spec
            .github_app
            .as_deref()
            .or(docs_run.spec.github_user.as_deref())
            .unwrap_or("");
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=docs-generator,github-identity={}",
            self.sanitize_label_value(github_identity)
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

                info!("Deleting old docs ConfigMap: {}", cm_name);
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

    async fn ensure_workspace_pvc(&self, docs_run: &DocsRun) -> Result<()> {
        let pvc_name = format!(
            "docs-workspace-{}",
            docs_run
                .spec
                .working_directory
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                })
                .collect::<String>()
                .trim_matches('-')
                .to_lowercase()
        );

        // Check if PVC already exists
        let pvcs: Api<k8s_openapi::api::core::v1::PersistentVolumeClaim> =
            Api::namespaced(self.ctx.client.clone(), &self.ctx.namespace);

        match pvcs.get(&pvc_name).await {
            Ok(_) => {
                error!("✅ PVC {} already exists", pvc_name);
                return Ok(());
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                // PVC doesn't exist, create it
                error!("📦 Creating PVC: {}", pvc_name);
            }
            Err(e) => return Err(e.into()),
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
                        self.sanitize_label_value(&docs_run.spec.working_directory),
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

        match pvcs.create(&kube::api::PostParams::default(), &pvc).await {
            Ok(_) => {
                error!("✅ Created PVC: {}", pvc_name);
                Ok(())
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                error!("✅ PVC {} already exists (created concurrently)", pvc_name);
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to create PVC {}: {:?}", pvc_name, e);
                Err(e.into())
            }
        }
    }
}

struct SshVolumes {
    volumes: Vec<serde_json::Value>,
    volume_mounts: Vec<serde_json::Value>,
}
