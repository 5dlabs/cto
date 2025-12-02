//! Template rendering for heal prompts using Handlebars.
//!
//! This module provides DRY template composition by:
//! - Loading shared partials (`_preamble`, `_issue-create`, etc.)
//! - Rendering alert-specific templates with context
//! - Supporting both `.hbs` and legacy `.md` templates

use anyhow::{Context, Result};
use handlebars::{
    Context as HbsContext, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

/// Helper function to concatenate strings in templates.
/// Usage: `{{concat "prefix" variable "suffix"}}`
fn concat_helper(
    h: &Helper,
    _: &Handlebars,
    _: &HbsContext,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let mut result = String::new();
    for param in h.params() {
        if let Some(s) = param.value().as_str() {
            result.push_str(s);
        } else {
            // Handle non-string values by converting to string
            result.push_str(param.value().to_string().trim_matches('"'));
        }
    }
    out.write(&result)?;
    Ok(())
}

/// Template engine for heal prompts
pub struct TemplateEngine<'a> {
    handlebars: Handlebars<'a>,
    prompts_dir: String,
}

/// Context for rendering alert templates
#[derive(Debug, Serialize)]
pub struct AlertContext {
    pub alert_id: String,
    pub pod_name: String,
    pub namespace: String,
    pub phase: String,
    pub task_id: String,
    pub agent: String,
    pub logs: String,
    pub expected_behaviors: String,
    pub duration: String,
    /// Additional alert-specific variables
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl TemplateEngine<'_> {
    /// Create a new template engine, loading partials from the prompts directory
    pub fn new(prompts_dir: &str) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Configure handlebars
        handlebars.set_strict_mode(false); // Allow missing variables

        // Register custom helpers
        handlebars.register_helper("concat", Box::new(concat_helper));

        // Load partials from the partials directory
        let partials_dir = Path::new(prompts_dir).join("partials");
        if partials_dir.exists() {
            Self::load_partials(&mut handlebars, &partials_dir)?;
        }

        Ok(Self {
            handlebars,
            prompts_dir: prompts_dir.to_string(),
        })
    }

    /// Load all partial templates from a directory
    fn load_partials(handlebars: &mut Handlebars, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "hbs") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .context("Invalid partial filename")?;

                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read partial: {}", path.display()))?;

                handlebars
                    .register_partial(name, &content)
                    .with_context(|| format!("Failed to register partial: {name}"))?;

                tracing::debug!("Loaded partial: {name}");
            }
        }
        Ok(())
    }

    /// Render an alert template with the given context
    pub fn render_alert(&self, alert_id: &str, context: &AlertContext) -> Result<String> {
        // Try .hbs first, fall back to .md
        let hbs_path = Path::new(&self.prompts_dir)
            .join("alerts")
            .join(format!("{alert_id}.hbs"));
        let md_path = Path::new(&self.prompts_dir).join(format!("{alert_id}.md"));

        let template_path = if hbs_path.exists() {
            hbs_path
        } else if md_path.exists() {
            md_path
        } else {
            anyhow::bail!("No template found for alert: {alert_id}");
        };

        let template = std::fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to read template: {}", template_path.display()))?;

        // For .hbs files, use full Handlebars rendering
        // For .md files, use simple string replacement (legacy compatibility)
        if template_path.extension().is_some_and(|ext| ext == "hbs") {
            self.handlebars
                .render_template(&template, context)
                .with_context(|| format!("Failed to render template: {alert_id}"))
        } else {
            // Legacy: simple string replacement
            Ok(Self::render_legacy(&template, context))
        }
    }

    /// Legacy rendering using simple string replacement
    fn render_legacy(template: &str, context: &AlertContext) -> String {
        let mut rendered = template
            .replace("{{pod_name}}", &context.pod_name)
            .replace("{{namespace}}", &context.namespace)
            .replace("{{phase}}", &context.phase)
            .replace("{{task_id}}", &context.task_id)
            .replace("{{agent}}", &context.agent)
            .replace("{{logs}}", &context.logs)
            .replace("{{expected_behaviors}}", &context.expected_behaviors)
            .replace("{{duration}}", &context.duration);

        // Replace extra context variables
        for (key, value) in &context.extra {
            let pattern = format!("{{{{{key}}}}}");
            rendered = rendered.replace(&pattern, value);
        }

        rendered
    }

    /// Map alert shorthand to template filename
    #[must_use]
    pub fn alert_to_filename(alert_id: &str) -> String {
        match alert_id {
            "a1" => "a1-comment-order".to_string(),
            "a2" => "a2-silent-failure".to_string(),
            "a3" => "a3-stale-progress".to_string(),
            "a4" => "a4-approval-loop".to_string(),
            "a5" => "a5-post-tess-ci".to_string(),
            "a7" => "a7-pod-failure".to_string(),
            "a8" => "a8-step-timeout".to_string(),
            "a9" => "a9-stuck-coderun".to_string(),
            "completion" => "success-completion".to_string(),
            _ => alert_id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_rendering() {
        let context = AlertContext {
            alert_id: "a2".to_string(),
            pod_name: "test-pod".to_string(),
            namespace: "cto".to_string(),
            phase: "Running".to_string(),
            task_id: "task-123".to_string(),
            agent: "Rex".to_string(),
            logs: "Some logs".to_string(),
            expected_behaviors: String::new(),
            duration: "5m".to_string(),
            extra: HashMap::new(),
        };

        let template = "Pod {{pod_name}} in {{namespace}} is {{phase}}";
        let rendered = TemplateEngine::render_legacy(template, &context);

        assert_eq!(rendered, "Pod test-pod in cto is Running");
    }

    #[test]
    fn test_concat_helper() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("concat", Box::new(concat_helper));

        let context = AlertContext {
            alert_id: "a2".to_string(),
            pod_name: "my-pod".to_string(),
            namespace: "cto".to_string(),
            phase: "Running".to_string(),
            task_id: "task-123".to_string(),
            agent: "Rex".to_string(),
            logs: String::new(),
            expected_behaviors: String::new(),
            duration: "5m".to_string(),
            extra: HashMap::new(),
        };

        // Test basic concat
        let template = "{{concat \"Pod Failure: \" pod_name}}";
        let result = handlebars.render_template(template, &context).unwrap();
        assert_eq!(result, "Pod Failure: my-pod");

        // Test multiple args
        let template = "{{concat \"[\" agent \"] \" pod_name}}";
        let result = handlebars.render_template(template, &context).unwrap();
        assert_eq!(result, "[Rex] my-pod");
    }
}

