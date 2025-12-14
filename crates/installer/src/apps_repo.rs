//! ArgoCD Apps Repository setup for customer workloads.
//!
//! This module handles the setup and configuration of the customer's ArgoCD apps repository,
//! where Bolt deploys Application manifests for preview and production environments.
//!
//! The apps repository follows the "App of Apps" pattern:
//! - `app-of-apps.yaml` - Root application that watches the repository
//! - `preview/` - Preview deployment manifests (task-{id}-preview.yaml)
//! - `production/` - Production deployment manifests (task-{id}-prod.yaml)
//! - `templates/` - Templates for Bolt to use when creating new applications

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

use crate::ui;

/// Template for the app-of-apps.yaml manifest.
/// This is applied to the customer's apps repository to bootstrap ArgoCD.
const APP_OF_APPS_TEMPLATE: &str = r"---
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: cto-apps
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: platform
  source:
    repoURL: {{APPS_REPO_URL}}
    targetRevision: {{APPS_REPO_BRANCH}}
    path: .
    directory:
      recurse: true
      exclude: |
        templates/*
        README.md
        app-of-apps.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: argocd
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=false
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
";

/// Template for preview application manifests.
const PREVIEW_APP_TEMPLATE: &str = r#"---
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: {{APP_NAME}}
  namespace: argocd
  labels:
    task-id: "{{TASK_ID}}"
    environment: preview
    managed-by: bolt
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: platform
  source:
    repoURL: {{REPO_URL}}
    targetRevision: {{BRANCH}}
    path: {{ARGOCD_PATH}}
  destination:
    server: https://kubernetes.default.svc
    namespace: {{NAMESPACE}}
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
    retry:
      limit: 3
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 1m
"#;

/// Template for production application manifests.
const PRODUCTION_APP_TEMPLATE: &str = r#"---
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: {{APP_NAME}}
  namespace: argocd
  labels:
    task-id: "{{TASK_ID}}"
    environment: production
    managed-by: bolt
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: platform
  source:
    repoURL: {{REPO_URL}}
    targetRevision: main
    path: {{ARGOCD_PATH}}
  destination:
    server: https://kubernetes.default.svc
    namespace: {{NAMESPACE}}
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
"#;

/// README template for the apps repository.
const README_TEMPLATE: &str = r"# CTO Apps - ArgoCD Application Manifests

This repository contains ArgoCD Application manifests for workloads deployed by the CTO platform.

## Structure

```
cto-apps/
├── app-of-apps.yaml          # Main ArgoCD app that watches this repo
├── preview/                   # Preview deployments (task-{id}-preview)
│   └── .gitkeep
├── production/                # Production deployments (task-{id}-prod)
│   └── .gitkeep
└── templates/                 # Templates for Bolt to use
    ├── preview-app.yaml.template
    └── production-app.yaml.template
```

## How It Works

### 1. Bolt Creates/Updates Application Manifests

When Bolt deploys:
- **Preview**: Creates `preview/task-{id}-preview.yaml`
- **Production**: Creates `production/task-{id}-prod.yaml`

### 2. ArgoCD Syncs Automatically

The `app-of-apps.yaml` application watches this repository and automatically
syncs any new or updated application manifests.

### 3. Applications Deploy

ArgoCD deploys the applications to their respective namespaces:
- Preview: `agent-platform-preview-task-{id}`
- Production: `agent-platform-prod-task-{id}`

## Managed by CTO Platform

This repository is managed by the [CTO Platform](https://github.com/5dlabs/cto).
Application manifests are automatically created and updated by the Bolt agent.
";

/// Apps repository manager for setting up customer ArgoCD apps.
pub struct AppsRepoManager {
    /// Apps repository URL (e.g., "https://github.com/myorg/cto-apps").
    repo_url: String,
    /// Branch for the apps repository.
    branch: String,
}

impl AppsRepoManager {
    /// Create a new apps repository manager.
    #[must_use]
    pub fn new(repo_url: String, branch: String) -> Self {
        Self { repo_url, branch }
    }

    /// Check if the repository exists and is accessible.
    ///
    /// Uses `gh repo view` to check if the repository exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the `gh` CLI is not available or the check fails.
    pub fn check_repo_exists(&self) -> Result<bool> {
        let repo_path = self.extract_repo_path()?;

        debug!(repo = %repo_path, "Checking if repository exists");

        let output = Command::new("gh")
            .args(["repo", "view", &repo_path, "--json", "name"])
            .output()
            .context("Failed to run 'gh repo view'. Is the GitHub CLI installed?")?;

        Ok(output.status.success())
    }

    /// Check if the repository has the required structure for CTO apps.
    ///
    /// Checks for:
    /// - `app-of-apps.yaml` file
    /// - `preview/` directory
    /// - `production/` directory
    /// - `templates/` directory
    ///
    /// # Errors
    ///
    /// Returns an error if the check fails.
    pub fn check_repo_structure(&self) -> Result<RepoStructureStatus> {
        let repo_path = self.extract_repo_path()?;

        debug!(repo = %repo_path, "Checking repository structure");

        // Check for app-of-apps.yaml
        let has_app_of_apps = self.file_exists_in_repo(&repo_path, "app-of-apps.yaml")?;

        // Check for directories (by checking for .gitkeep or any file)
        let has_preview = self.directory_exists_in_repo(&repo_path, "preview")?;
        let has_production = self.directory_exists_in_repo(&repo_path, "production")?;
        let has_templates = self.directory_exists_in_repo(&repo_path, "templates")?;

        if has_app_of_apps && has_preview && has_production && has_templates {
            Ok(RepoStructureStatus::Valid)
        } else if !has_app_of_apps && !has_preview && !has_production && !has_templates {
            Ok(RepoStructureStatus::Empty)
        } else {
            Ok(RepoStructureStatus::Partial {
                has_app_of_apps,
                has_preview,
                has_production,
                has_templates,
            })
        }
    }

    /// Initialize the repository with the required structure.
    ///
    /// Creates:
    /// - `app-of-apps.yaml`
    /// - `preview/.gitkeep`
    /// - `production/.gitkeep`
    /// - `templates/preview-app.yaml.template`
    /// - `templates/production-app.yaml.template`
    /// - `README.md`
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub fn initialize_repo(&self, temp_dir: &Path) -> Result<()> {
        let repo_path = self.extract_repo_path()?;

        info!(repo = %repo_path, "Initializing apps repository structure");

        // Clone the repo to temp directory
        let clone_dir = temp_dir.join("cto-apps");
        ui::print_info(&format!("Cloning {repo_path} to initialize..."));

        let output = Command::new("gh")
            .args([
                "repo",
                "clone",
                &repo_path,
                clone_dir.to_str().unwrap_or_default(),
            ])
            .output()
            .context("Failed to clone repository")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to clone repository: {}", stderr.trim());
        }

        // Create directory structure
        std::fs::create_dir_all(clone_dir.join("preview"))
            .context("Failed to create preview directory")?;
        std::fs::create_dir_all(clone_dir.join("production"))
            .context("Failed to create production directory")?;
        std::fs::create_dir_all(clone_dir.join("templates"))
            .context("Failed to create templates directory")?;

        // Create .gitkeep files
        std::fs::write(clone_dir.join("preview/.gitkeep"), "")
            .context("Failed to create preview/.gitkeep")?;
        std::fs::write(clone_dir.join("production/.gitkeep"), "")
            .context("Failed to create production/.gitkeep")?;

        // Create app-of-apps.yaml
        let app_of_apps = APP_OF_APPS_TEMPLATE
            .replace("{{APPS_REPO_URL}}", &self.repo_url)
            .replace("{{APPS_REPO_BRANCH}}", &self.branch);
        std::fs::write(clone_dir.join("app-of-apps.yaml"), app_of_apps)
            .context("Failed to create app-of-apps.yaml")?;

        // Create templates
        std::fs::write(
            clone_dir.join("templates/preview-app.yaml.template"),
            PREVIEW_APP_TEMPLATE,
        )
        .context("Failed to create preview template")?;
        std::fs::write(
            clone_dir.join("templates/production-app.yaml.template"),
            PRODUCTION_APP_TEMPLATE,
        )
        .context("Failed to create production template")?;

        // Create README
        std::fs::write(clone_dir.join("README.md"), README_TEMPLATE)
            .context("Failed to create README.md")?;

        // Commit and push
        ui::print_info("Committing initial structure...");

        // git add
        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(&clone_dir)
            .output()
            .context("Failed to run git add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git add failed: {}", stderr.trim());
        }

        // git commit
        let output = Command::new("git")
            .args([
                "commit",
                "-m",
                "Initialize CTO apps repository structure\n\nCreated by CTO Platform installer.",
            ])
            .current_dir(&clone_dir)
            .output()
            .context("Failed to run git commit")?;

        // Commit may fail if nothing to commit (already initialized)
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("nothing to commit") {
                info!("Repository already has content, skipping commit");
            } else {
                anyhow::bail!("git commit failed: {}", stderr.trim());
            }
        }

        // git push
        ui::print_info("Pushing to remote...");
        let output = Command::new("git")
            .args(["push", "origin", &self.branch])
            .current_dir(&clone_dir)
            .output()
            .context("Failed to run git push")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Check if it's because nothing to push
            if stderr.contains("Everything up-to-date") {
                info!("Repository already up to date");
            } else {
                anyhow::bail!("git push failed: {}", stderr.trim());
            }
        }

        ui::print_success("Apps repository initialized successfully");
        Ok(())
    }

    /// Apply the app-of-apps manifest to ArgoCD.
    ///
    /// This creates the root ArgoCD Application that watches the apps repository.
    ///
    /// # Errors
    ///
    /// Returns an error if applying the manifest fails.
    pub fn apply_app_of_apps(&self, kubeconfig: &Path) -> Result<()> {
        info!(repo = %self.repo_url, "Applying apps app-of-apps manifest");

        let manifest = APP_OF_APPS_TEMPLATE
            .replace("{{APPS_REPO_URL}}", &self.repo_url)
            .replace("{{APPS_REPO_BRANCH}}", &self.branch);

        let mut child = Command::new("kubectl")
            .args([
                "--kubeconfig",
                kubeconfig.to_str().unwrap_or_default(),
                "apply",
                "-f",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(manifest.as_bytes())
                .context("Failed to write manifest to kubectl stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for kubectl")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kubectl apply failed: {}", stderr.trim());
        }

        info!("Apps app-of-apps manifest applied successfully");
        Ok(())
    }

    /// Extract the repository path (owner/repo) from the URL.
    fn extract_repo_path(&self) -> Result<String> {
        // Handle various URL formats:
        // - https://github.com/owner/repo
        // - https://github.com/owner/repo.git
        // - git@github.com:owner/repo.git
        let url = self.repo_url.trim_end_matches(".git");

        if url.contains("github.com/") {
            let parts: Vec<&str> = url.split("github.com/").collect();
            if parts.len() >= 2 {
                return Ok(parts[1].to_string());
            }
        } else if url.contains("github.com:") {
            let parts: Vec<&str> = url.split("github.com:").collect();
            if parts.len() >= 2 {
                return Ok(parts[1].to_string());
            }
        }

        anyhow::bail!(
            "Could not extract repository path from URL: {}",
            self.repo_url
        )
    }

    /// Check if a file exists in the repository.
    fn file_exists_in_repo(&self, repo_path: &str, file_path: &str) -> Result<bool> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/contents/{}?ref={}",
                    repo_path, file_path, self.branch
                ),
            ])
            .output()
            .context("Failed to check file existence")?;

        Ok(output.status.success())
    }

    /// Check if a directory exists in the repository.
    fn directory_exists_in_repo(&self, repo_path: &str, dir_path: &str) -> Result<bool> {
        // GitHub API returns directory contents if it exists
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/contents/{}?ref={}",
                    repo_path, dir_path, self.branch
                ),
            ])
            .output()
            .context("Failed to check directory existence")?;

        Ok(output.status.success())
    }
}

/// Status of the apps repository structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoStructureStatus {
    /// Repository has all required files and directories.
    Valid,
    /// Repository is empty or has no CTO-related files.
    Empty,
    /// Repository has some but not all required structure.
    Partial {
        /// Whether app-of-apps.yaml exists.
        has_app_of_apps: bool,
        /// Whether preview/ directory exists.
        has_preview: bool,
        /// Whether production/ directory exists.
        has_production: bool,
        /// Whether templates/ directory exists.
        has_templates: bool,
    },
}

/// Configure the apps repository.
///
/// This is the main entry point for apps repository setup:
/// 1. Check if the repository exists
/// 2. Check if it has the required structure
/// 3. Initialize if needed
/// 4. Apply the app-of-apps manifest to ArgoCD
///
/// # Errors
///
/// Returns an error if any step fails.
pub async fn configure_apps_repo(
    apps_repo: &str,
    apps_repo_branch: &str,
    kubeconfig: &Path,
    temp_dir: &Path,
) -> Result<()> {
    let manager = AppsRepoManager::new(apps_repo.to_string(), apps_repo_branch.to_string());

    // Step 1: Check if repository exists
    ui::print_info(&format!("Checking apps repository: {apps_repo}"));
    if !manager.check_repo_exists()? {
        anyhow::bail!(
            "Apps repository does not exist or is not accessible: {apps_repo}\nPlease create the repository first or check your GitHub authentication."
        );
    }
    ui::print_success("Apps repository exists");

    // Step 2: Check repository structure
    ui::print_info("Checking repository structure...");
    let status = manager.check_repo_structure()?;

    match status {
        RepoStructureStatus::Valid => {
            ui::print_success("Apps repository has valid structure");
        }
        RepoStructureStatus::Empty => {
            ui::print_info("Apps repository is empty, initializing...");
            manager.initialize_repo(temp_dir)?;
        }
        RepoStructureStatus::Partial {
            has_app_of_apps,
            has_preview,
            has_production,
            has_templates,
        } => {
            warn!(
                "Apps repository has partial structure: app-of-apps={}, preview={}, production={}, templates={}",
                has_app_of_apps, has_preview, has_production, has_templates
            );
            ui::print_warning("Apps repository has partial structure");
            ui::print_info("Attempting to complete initialization...");
            manager.initialize_repo(temp_dir)?;
        }
    }

    // Step 3: Apply app-of-apps manifest to ArgoCD
    ui::print_info("Applying apps app-of-apps manifest to ArgoCD...");
    manager.apply_app_of_apps(kubeconfig)?;
    ui::print_success("Apps repository configured successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_path_https() {
        let manager =
            AppsRepoManager::new("https://github.com/myorg/cto-apps".into(), "main".into());
        assert_eq!(manager.extract_repo_path().unwrap(), "myorg/cto-apps");
    }

    #[test]
    fn test_extract_repo_path_https_with_git() {
        let manager = AppsRepoManager::new(
            "https://github.com/myorg/cto-apps.git".into(),
            "main".into(),
        );
        assert_eq!(manager.extract_repo_path().unwrap(), "myorg/cto-apps");
    }

    #[test]
    fn test_extract_repo_path_ssh() {
        let manager =
            AppsRepoManager::new("git@github.com:myorg/cto-apps.git".into(), "main".into());
        assert_eq!(manager.extract_repo_path().unwrap(), "myorg/cto-apps");
    }

    #[test]
    fn test_app_of_apps_template_substitution() {
        let manifest = APP_OF_APPS_TEMPLATE
            .replace("{{APPS_REPO_URL}}", "https://github.com/myorg/cto-apps")
            .replace("{{APPS_REPO_BRANCH}}", "main");

        assert!(manifest.contains("repoURL: https://github.com/myorg/cto-apps"));
        assert!(manifest.contains("targetRevision: main"));
    }
}
