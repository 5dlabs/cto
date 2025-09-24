# Task 5: Create Template Management System

## Overview
Implement dynamic template selection and rendering system for CLI-specific prompts, configurations, and scripts with fallback mechanisms. This system enables the platform to generate optimized content for each CLI while maintaining consistency and supporting graceful degradation.

## Context
Each CLI in the Multi-CLI Agent Platform requires different configuration formats, prompt styles, and container scripts. The template management system provides the intelligent rendering layer that adapts content for each CLI type while maintaining DRY principles through inheritance and fallback mechanisms.

## Technical Specification

### 1. Template Types and Organization
```
templates/
├── claude/
│   ├── system_prompt.hbs
│   ├── config_file.hbs
│   ├── entrypoint_script.hbs
│   └── memory_file.hbs
├── codex/
│   ├── system_prompt.hbs
│   ├── config_file.toml.hbs
│   ├── entrypoint_script.hbs
│   └── memory_file.hbs
├── generic/
│   ├── system_prompt.hbs
│   ├── config_file.hbs
│   └── entrypoint_script.hbs
└── defaults/
    └── fallback_template.hbs
```

### 2. TemplateManager Architecture
```rust
pub struct TemplateManager {
    registry: HashMap<TemplateKey, CompiledTemplate>,
    handlebars: Handlebars<'static>,
    fallback_chain: FallbackChain,
    hot_reloader: Option<HotReloader>,
    performance_cache: Arc<RwLock<LruCache<String, RenderedOutput>>>,
    metrics: TemplateMetrics,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TemplateKey {
    cli_type: CLIType,
    template_type: TemplateType,
    version: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateType {
    SystemPrompt,
    ConfigFile,
    EntrypointScript,
    MemoryFile,
    ErrorMessage,
    HealthCheck,
}
```

### 3. Template Discovery and Loading
```rust
impl TemplateManager {
    pub async fn new(template_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            registry: HashMap::new(),
            handlebars: Self::setup_handlebars()?,
            fallback_chain: FallbackChain::default(),
            hot_reloader: None,
            performance_cache: Arc::new(RwLock::new(LruCache::new(1000))),
            metrics: TemplateMetrics::new(),
        };

        manager.discover_and_load_templates(&template_dir).await?;
        Ok(manager)
    }

    async fn discover_and_load_templates(&mut self, template_dir: &Path) -> Result<()> {
        let discovery_walker = WalkDir::new(template_dir);

        for entry in discovery_walker {
            let entry = entry?;
            if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("hbs")) {
                let template_key = self.parse_template_path(entry.path())?;
                let template_content = tokio::fs::read_to_string(entry.path()).await?;

                // Compile template for performance
                let compiled = self.handlebars.compile_template(&template_content)?;
                self.registry.insert(template_key, CompiledTemplate {
                    template: compiled,
                    source_path: entry.path().to_path_buf(),
                    last_modified: entry.metadata()?.modified()?,
                    checksum: self.calculate_checksum(&template_content),
                });
            }
        }

        info!("Loaded {} templates", self.registry.len());
        Ok(())
    }
}
```

### 4. Intelligent Template Selection with Fallback
```rust
#[derive(Debug, Clone)]
pub struct FallbackChain {
    strategies: Vec<FallbackStrategy>,
}

#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    CliSpecific(CLIType),    // templates/claude/system_prompt.hbs
    Generic,                 // templates/generic/system_prompt.hbs
    Default,                 // templates/defaults/fallback_template.hbs
    Inherited(CLIType),      // Inherit from similar CLI (e.g., Qwen -> Gemini)
}

impl TemplateManager {
    pub async fn render_template(&self, cli_type: CLIType, template_type: TemplateType, context: &TemplateContext) -> Result<String> {
        let template_key = TemplateKey {
            cli_type,
            template_type,
            version: context.template_version.clone(),
        };

        // Try cache first
        let cache_key = format!("{:?}:{}", template_key, context.checksum());
        if let Some(cached) = self.performance_cache.read().await.get(&cache_key) {
            self.metrics.record_cache_hit();
            return Ok(cached.content.clone());
        }

        // Apply fallback chain
        let template = self.select_template_with_fallback(&template_key).await?;

        // Render template
        let start = Instant::now();
        let rendered = template.render(&self.handlebars, context).await?;
        let render_duration = start.elapsed();

        // Cache result
        self.performance_cache.write().await.put(cache_key, RenderedOutput {
            content: rendered.clone(),
            rendered_at: Utc::now(),
            render_duration,
        });

        // Record metrics
        self.metrics.record_render(cli_type, template_type, render_duration, true);

        Ok(rendered)
    }

    async fn select_template_with_fallback(&self, key: &TemplateKey) -> Result<&CompiledTemplate> {
        let fallback_chain = self.fallback_chain.build_for(key.cli_type);

        for strategy in fallback_chain {
            let candidate_key = self.apply_fallback_strategy(key, &strategy);
            if let Some(template) = self.registry.get(&candidate_key) {
                debug!("Selected template using strategy: {:?}", strategy);
                return Ok(template);
            }
        }

        Err(anyhow!("No template found for {:?} after trying all fallback strategies", key))
    }
}
```

### 5. Advanced Handlebars Helpers
```rust
impl TemplateManager {
    fn setup_handlebars() -> Result<Handlebars<'static>> {
        let mut hb = Handlebars::new();

        // Custom helpers for CLI-specific logic
        hb.register_helper("cli_config_format", Box::new(cli_config_format_helper));
        hb.register_helper("model_context_window", Box::new(model_context_window_helper));
        hb.register_helper("escape_shell", Box::new(escape_shell_helper));
        hb.register_helper("json_stringify", Box::new(json_stringify_helper));
        hb.register_helper("toml_serialize", Box::new(toml_serialize_helper));
        hb.register_helper("if_cli_supports", Box::new(if_cli_supports_helper));

        // Conditional rendering helpers
        hb.register_helper("if_streaming", Box::new(|h: &handlebars::Helper, _: &Handlebars, ctx: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
            let cli_type = ctx.data().get("cli_type").and_then(|v| v.as_str()).unwrap_or("");
            let supports_streaming = match cli_type {
                "claude" | "opencode" | "gemini" => true,
                "codex" | "cursor" | "openhands" => false,
                _ => false,
            };

            if supports_streaming {
                let t = h.template().unwrap();
                t.render(&handlebars::Handlebars::new(), ctx, &mut handlebars::RenderContext::new(None), out)?;
            }

            Ok(())
        }));

        Ok(hb)
    }
}

// Custom helper implementations
fn cli_config_format_helper(h: &handlebars::Helper, _: &Handlebars, ctx: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let cli_type = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("claude");
    let config_format = match cli_type {
        "codex" => "toml",
        "claude" | "opencode" | "gemini" => "json",
        "cursor" | "openhands" => "yaml",
        _ => "json",
    };
    out.write(config_format)?;
    Ok(())
}
```

### 6. Template Context System
```rust
#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    // Agent information
    pub agent_name: String,
    pub github_app: String,
    pub repository: RepositoryInfo,

    // CLI configuration
    pub cli_type: CLIType,
    pub cli_config: serde_json::Value,
    pub model: String,
    pub capabilities: CliCapabilities,

    // Runtime context
    pub workspace_path: PathBuf,
    pub environment: HashMap<String, String>,
    pub container_info: ContainerInfo,

    // Template metadata
    pub template_version: Option<String>,
    pub render_timestamp: DateTime<Utc>,

    // Custom variables
    pub custom: HashMap<String, serde_json::Value>,
}

impl TemplateContext {
    pub fn for_agent(agent_config: &AgentConfig, runtime_info: &RuntimeInfo) -> Result<Self> {
        Ok(Self {
            agent_name: agent_config.name.clone(),
            github_app: agent_config.github_app.clone(),
            repository: runtime_info.repository.clone(),
            cli_type: agent_config.cli_type,
            cli_config: agent_config.cli_config.clone(),
            model: agent_config.model.clone(),
            capabilities: agent_config.capabilities.clone(),
            workspace_path: runtime_info.workspace_path.clone(),
            environment: runtime_info.environment.clone(),
            container_info: runtime_info.container.clone(),
            template_version: None,
            render_timestamp: Utc::now(),
            custom: HashMap::new(),
        })
    }

    pub fn checksum(&self) -> String {
        let mut hasher = DefaultHasher::new();
        // Hash significant fields that affect rendering
        self.agent_name.hash(&mut hasher);
        self.cli_type.hash(&mut hasher);
        self.model.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
```

### 7. Hot Reloading for Development
```rust
pub struct HotReloader {
    watcher: RecommendedWatcher,
    template_manager: Arc<RwLock<TemplateManager>>,
    reload_debouncer: Debouncer<RecommendedWatcher>,
}

impl HotReloader {
    pub async fn new(template_dir: PathBuf, manager: Arc<RwLock<TemplateManager>>) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel(1000);

        let watcher = notify::Watcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = tx.try_send(event);
                }
            },
            notify::Config::default(),
        )?;

        let debouncer = Debouncer::new(Duration::from_millis(500), move |events: Vec<notify::Event>| {
            tokio::spawn(async move {
                for event in events {
                    if event.kind.is_modify() || event.kind.is_create() {
                        Self::handle_template_change(&manager, event.paths).await;
                    }
                }
            });
        });

        let mut hot_reloader = Self {
            watcher,
            template_manager: manager.clone(),
            reload_debouncer: debouncer,
        };

        hot_reloader.watcher.watch(&template_dir, RecursiveMode::Recursive)?;

        // Handle reload events
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                hot_reloader.reload_debouncer.send(event);
            }
        });

        Ok(hot_reloader)
    }

    async fn handle_template_change(manager: &Arc<RwLock<TemplateManager>>, paths: Vec<PathBuf>) {
        let mut manager = manager.write().await;

        for path in paths {
            if path.extension() == Some(OsStr::new("hbs")) {
                info!("Reloading template: {:?}", path);
                if let Err(e) = manager.reload_template(&path).await {
                    error!("Failed to reload template {:?}: {}", path, e);
                }
            }
        }
    }
}
```

### 8. Template Versioning System
```rust
pub struct TemplateVersionManager {
    versions: HashMap<String, VersionedTemplateSet>,
    rollout_config: RolloutConfig,
}

#[derive(Debug, Clone)]
pub struct VersionedTemplateSet {
    version: String,
    templates: HashMap<TemplateKey, CompiledTemplate>,
    rollout_percentage: f64,
    created_at: DateTime<Utc>,
    deprecated: bool,
}

impl TemplateVersionManager {
    pub async fn select_version_for_agent(&self, agent_name: &str) -> String {
        // Use consistent hashing for gradual rollout
        let agent_hash = self.hash_agent_name(agent_name);
        let rollout_threshold = agent_hash % 100;

        for (version, template_set) in &self.versions {
            if !template_set.deprecated && rollout_threshold < (template_set.rollout_percentage * 100.0) as u32 {
                return version.clone();
            }
        }

        "stable".to_string() // Fallback to stable version
    }

    pub async fn gradual_rollout(&mut self, new_version: String, target_percentage: f64, duration: Duration) -> Result<()> {
        let steps = 10; // Rollout in 10% increments
        let step_duration = duration / steps;
        let step_increment = target_percentage / steps as f64;

        for step in 1..=steps {
            let current_percentage = step_increment * step as f64;

            if let Some(template_set) = self.versions.get_mut(&new_version) {
                template_set.rollout_percentage = current_percentage;
                info!("Template version {} rollout: {:.1}%", new_version, current_percentage);
            }

            tokio::time::sleep(step_duration).await;
        }

        Ok(())
    }
}
```

### 9. Performance Optimization and Metrics
```rust
pub struct TemplateMetrics {
    render_histogram: Histogram,
    cache_hits: Counter,
    cache_misses: Counter,
    template_errors: Counter,
    active_templates: Gauge,
}

impl TemplateMetrics {
    pub fn record_render(&self, cli_type: CLIType, template_type: TemplateType, duration: Duration, success: bool) {
        let labels = &[
            ("cli_type", cli_type.as_str()),
            ("template_type", template_type.as_str()),
            ("success", success.to_string().as_str()),
        ];

        self.render_histogram.observe_with(duration.as_secs_f64(), labels);

        if !success {
            self.template_errors.inc_by_with(1, labels);
        }
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }
}

// Performance cache with intelligent eviction
pub struct TemplatePerformanceCache {
    cache: LruCache<String, CachedRender>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct CachedRender {
    content: String,
    rendered_at: DateTime<Utc>,
    access_count: AtomicU64,
    last_accessed: DateTime<Utc>,
}
```

## Implementation Steps

### Phase 1: Core Template System
1. Create TemplateManager with registry and discovery
2. Implement Handlebars setup with custom helpers
3. Build template key system and organization
4. Add basic fallback chain logic

### Phase 2: Advanced Template Features
1. Implement intelligent template selection
2. Add template context system
3. Create performance caching layer
4. Build template validation and syntax checking

### Phase 3: Development Tools
1. Add hot reloading for development workflow
2. Implement template versioning system
3. Create gradual rollout mechanisms
4. Build template debugging tools

### Phase 4: Performance and Metrics
1. Optimize template rendering performance
2. Add comprehensive metrics collection
3. Implement intelligent cache eviction
4. Build performance monitoring dashboards

## Success Criteria
- Dynamic template selection with fallback chains
- Sub-10ms template rendering for typical templates
- 1000+ templates per second rendering capacity
- Hot reloading without service disruption
- Template versioning with gradual rollout
- <2MB memory usage for 100 compiled templates
- Comprehensive metrics and monitoring

## Dependencies
- Task 3: CLI adapter trait system for capabilities
- Task 4: Configuration resolution for context data
- Handlebars templating engine
- File system watcher for hot reloading
- Metrics collection infrastructure

## Files Created
```
controller/src/templates/
├── manager.rs (main template management)
├── discovery.rs (template discovery and loading)
├── fallback.rs (fallback chain logic)
├── context.rs (template context system)
├── helpers.rs (custom Handlebars helpers)
├── versioning.rs (template version management)
├── hot_reload.rs (development hot reloading)
├── cache.rs (performance caching)
└── metrics.rs (template metrics)

templates/
├── claude/ (Claude-specific templates)
├── codex/ (Codex-specific templates)
├── opencode/ (Opencode templates)
├── gemini/ (Gemini templates)
├── generic/ (Generic fallback templates)
└── defaults/ (Default fallback templates)
```

## Risk Mitigation

### Template Security
- Sandboxed template execution
- Input validation and sanitization
- No arbitrary code execution in templates
- Secure helper functions only

### Performance Impact
- Compiled template caching
- Intelligent cache eviction
- Metrics-driven optimization
- Resource usage monitoring

### Development Workflow
- Hot reloading for rapid iteration
- Template validation and testing
- Version control integration
- Rollback capabilities

## Testing Strategy
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_template_fallback_chain() {
        let manager = TemplateManager::new().await.unwrap();

        // Test missing CLI-specific template falls back to generic
        let result = manager.render_template(
            CLIType::NewCLI,
            TemplateType::SystemPrompt,
            &create_test_context()
        ).await;

        assert!(result.is_ok());
        // Should have used generic fallback
    }

    #[tokio::test]
    async fn test_performance_requirements() {
        let manager = TemplateManager::new().await.unwrap();

        let start = Instant::now();
        for _ in 0..1000 {
            manager.render_template(
                CLIType::Claude,
                TemplateType::SystemPrompt,
                &create_test_context()
            ).await.unwrap();
        }

        assert!(start.elapsed() < Duration::from_secs(1)); // 1000 renders in <1s
    }
}
```

This template management system provides the intelligent content generation layer that adapts to each CLI's unique requirements while maintaining consistency and performance.