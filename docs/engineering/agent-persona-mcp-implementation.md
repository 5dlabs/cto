# Agent Persona MCP Tool Implementation Guide

## MCP Tool Definition

### Tool Registration in `mcp/src/tools.rs`





```rust
Tool {
    name: "create_agent_persona",
    description: "Create a new AI agent persona with GitHub App, character generation, and full configuration",
    parameters: json!({
        "type": "object",
        "required": ["purpose"],
        "properties": {
            "purpose": {
                "type": "string",
                "description": "What the agent does (e.g., 'Security vulnerability detection and code auditing')"
            },
            "capabilities": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Specific capabilities (auto-inferred from purpose if not provided)"
            },
            "personality_hints": {
                "type": "object",
                "properties": {
                    "archetype": {
                        "type": "string",
                        "description": "Character archetype (e.g., 'detective', 'engineer', 'artist')"
                    },
                    "tone": {
                        "type": "string",
                        "description": "Communication tone (e.g., 'professional', 'friendly', 'analytical')"
                    },
                    "quirks": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Personality quirks (e.g., 'uses metaphors', 'explains with examples')"
                    }
                }
            },
            "github_org": {
                "type": "string",
                "description": "GitHub organization (defaults to config value)"
            },
            "permissions": {
                "type": "object",
                "description": "GitHub App permissions (auto-generated if not provided)"
            },
            "deploy": {
                "type": "boolean",
                "description": "Whether to deploy the agent immediately",
                "default": false
            },
            "namespace": {
                "type": "string",
                "description": "Kubernetes namespace",
                "default": "cto"
            }
        }
    })
}








```

## Implementation Flow

### 1. Character Generation Phase





```rust
async fn generate_agent_character(
    purpose: &str,
    hints: Option<PersonalityHints>
) -> Result<AgentPersona> {
    // Use local LLM or configured AI model
    let prompt = format!(
        r#"Create a unique AI agent persona for the following purpose:
        Purpose: {}

        Personality hints:
        - Archetype: {}
        - Tone: {}
        - Quirks: {:?}

        Generate:


        1. A human-like name (single first name)


        2. A detailed personality profile


        3. Communication style guidelines


        4. Three key personality traits


        5. A visual description for an avatar

        Format as JSON with fields: name, archetype, traits, communication_style, avatar_description"#,
        purpose,
        hints.archetype.unwrap_or("auto-generate"),
        hints.tone.unwrap_or("professional"),
        hints.quirks.unwrap_or_default()
    );

    let response = ai_client.generate(prompt).await?;
    let persona: AgentPersona = serde_json::from_str(&response)?;

    Ok(persona)
}








```

Example Generated Persona:




```json
{
  "name": "Cipher",
  "archetype": "Security Detective",
  "traits": [
    "Methodical and thorough in investigation",
    "Pattern recognition specialist",
    "Protective and vigilant"
  ],
  "communication_style": {
    "tone": "Analytical yet approachable",
    "patterns": [
      "Uses security metaphors",
      "Provides risk assessments",
      "Suggests preventive measures"
    ]
  },
  "avatar_description": "A sleek black cat with glowing green eyes, wearing a detective's magnifying glass as a monocle"
}








```

### 2. System Prompt Generation





```rust
async fn generate_system_prompts(
    persona: &AgentPersona,
    purpose: &str,
    capabilities: &[String]
) -> Result<SystemPrompts> {
    let base_prompt = format!(
        r#"You are {}, a {} specialized in {}.

Core Personality:
{}

Your approach to work:
{}

Communication style:
{}"#,
        persona.name,
        persona.archetype,
        purpose,
        persona.traits.join("\n"),
        persona.work_approach(),
        persona.communication_style.describe()
    );

    // Generate task-specific prompts
    let mut task_prompts = HashMap::new();

    if capabilities.contains(&"code_review".to_string()) {
        task_prompts.insert("code_review", format!(
            r#"When reviewing code, you:


- {}


- Focus on {}


- Communicate findings by {}"#,
            persona.review_approach(),
            persona.focus_areas(),
            persona.communication_pattern()
        ));
    }

    if capabilities.contains(&"problem_solving".to_string()) {
        task_prompts.insert("problem_solving", format!(
            r#"When solving problems, you:


- Start by {}


- Apply {} methodology


- Present solutions by {}"#,
            persona.problem_approach(),
            persona.methodology(),
            persona.solution_presentation()
        ));
    }

    Ok(SystemPrompts {
        base: base_prompt,
        task_specific: task_prompts,
    })
}








```

### 3. GitHub App Creation





```rust
async fn create_github_app_for_agent(
    persona: &AgentPersona,
    purpose: &str,
    org: &str,
    custom_permissions: Option<Permissions>
) -> Result<GitHubAppInfo> {
    // Generate app manifest
    let manifest = GitHubAppManifest {
        name: format!("{}-AI-Agent", persona.name),
        url: format!("https://github.com/{}/cto", org),
        description: format!("{}: {}", persona.archetype, purpose),
        hook_attributes: WebhookConfig {
            url: format!("https://webhooks.{}.com/agents/{}",
                org, persona.name.to_lowercase()),
            active: true,
            events: infer_events_from_purpose(purpose),
        },
        redirect_url: format!("https://platform.{}.com/agents/{}/callback",
            org, persona.name.to_lowercase()),
        public: false,
        default_permissions: custom_permissions
            .unwrap_or_else(|| infer_permissions_from_purpose(purpose)),
        default_events: infer_events_from_purpose(purpose),
    };

    // Option 1: Use management app to create
    if let Some(mgmt_app) = get_management_app() {
        return mgmt_app.create_child_app(manifest).await;
    }

    // Option 2: Generate creation URL for manual step
    let manifest_json = serde_json::to_string(&manifest)?;
    let encoded = urlencoding::encode(&manifest_json);
    let state = uuid::Uuid::new_v4().to_string();

    // Store state for callback
    store_creation_state(&state, persona, &manifest).await?;

    let creation_url = format!(
        "https://github.com/organizations/{}/settings/apps/new?state={}&manifest={}",
        org, state, encoded
    );

    // Return URL and wait for callback
    Ok(GitHubAppInfo::Pending {
        creation_url,
        state,
    })
}








```

### 4. Kubernetes Configuration





```rust
async fn create_k8s_resources(
    persona: &AgentPersona,
    app_info: &GitHubAppInfo,
    prompts: &SystemPrompts,
    namespace: &str
) -> Result<()> {
    let k8s_client = kube::Client::try_default().await?;

    // Create ConfigMap with agent configuration
    let config_map = ConfigMap {
        metadata: ObjectMeta {
            name: Some(format!("{}-config", persona.name.to_lowercase())),
            namespace: Some(namespace.to_string()),
            labels: Some(btreemap! {
                "app".to_string() => persona.name.to_lowercase(),
                "type".to_string() => "agent-config".to_string(),
                "created-by".to_string() => "persona-creator".to_string(),
            }),
            ..Default::default()
        },
        data: Some(btreemap! {
            "agent.yaml".to_string() => serde_yaml::to_string(&AgentConfig {
                name: persona.name.clone(),
                archetype: persona.archetype.clone(),
                purpose: purpose.to_string(),
                prompts: prompts.clone(),
                github_app_id: app_info.app_id,
                capabilities: persona.capabilities.clone(),
            })?,
            "persona.json".to_string() => serde_json::to_string_pretty(persona)?,
        }),
        ..Default::default()
    };

    let api: Api<ConfigMap> = Api::namespaced(k8s_client.clone(), namespace);
    api.create(&PostParams::default(), &config_map).await?;

    // Create Secret for GitHub App credentials
    let secret = Secret {
        metadata: ObjectMeta {
            name: Some(format!("{}-github-app", persona.name.to_lowercase())),
            namespace: Some(namespace.to_string()),
            labels: Some(btreemap! {
                "app".to_string() => persona.name.to_lowercase(),
                "type".to_string() => "github-app".to_string(),
            }),
            ..Default::default()
        },
        type_: Some("Opaque".to_string()),
        data: Some(btreemap! {
            "app-id".to_string() => base64::encode(&app_info.app_id),
            "private-key".to_string() => base64::encode(&app_info.private_key),
            "webhook-secret".to_string() => base64::encode(&app_info.webhook_secret),
            "client-id".to_string() => base64::encode(&app_info.client_id),
            "client-secret".to_string() => base64::encode(&app_info.client_secret),
        }),
        ..Default::default()
    };

    let secret_api: Api<Secret> = Api::namespaced(k8s_client, namespace);
    secret_api.create(&PostParams::default(), &secret).await?;

    Ok(())
}








```

### 5. Deployment (Optional)





```rust
async fn deploy_agent(
    persona: &AgentPersona,
    namespace: &str
) -> Result<()> {
    // Create CRDs for the agent
    let code_run = CodeRun {
        metadata: ObjectMeta {
            name: Some(format!("{}-deployment", persona.name.to_lowercase())),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: CodeRunSpec {
            agent: persona.name.to_lowercase(),
            model: "claude-3-5-sonnet".to_string(),
            repository: "https://github.com/5dlabs/cto".to_string(),
            service: format!("{}-service", persona.name.to_lowercase()),
            // ... other fields
        },
        ..Default::default()
    };

    // Submit to controller
    let api: Api<CodeRun> = Api::namespaced(k8s_client, namespace);
    api.create(&PostParams::default(), &code_run).await?;

    Ok(())
}








```

## Complete MCP Tool Handler





```rust
async fn handle_create_agent_persona(params: Value) -> Result<Value> {
    // Parse parameters
    let purpose = params["purpose"].as_str()
        .ok_or("Purpose is required")?;

    let hints = params.get("personality_hints")
        .map(|h| serde_json::from_value::<PersonalityHints>(h.clone()))
        .transpose()?;

    let capabilities = params.get("capabilities")
        .and_then(|c| c.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>())
        .unwrap_or_else(|| infer_capabilities_from_purpose(purpose));

    let github_org = params.get("github_org")
        .and_then(|o| o.as_str())
        .unwrap_or("5dlabs");

    let should_deploy = params.get("deploy")
        .and_then(|d| d.as_bool())
        .unwrap_or(false);

    let namespace = params.get("namespace")
        .and_then(|n| n.as_str())
        .unwrap_or("cto");

    // Step 1: Generate character
    println!("🎭 Generating agent persona...");
    let persona = generate_agent_character(purpose, hints).await?;
    println!("✅ Created persona: {} the {}", persona.name, persona.archetype);

    // Step 2: Generate prompts
    println!("📝 Generating system prompts...");
    let prompts = generate_system_prompts(&persona, purpose, &capabilities).await?;
    println!("✅ Generated {} task-specific prompts", prompts.task_specific.len());

    // Step 3: Create GitHub App
    println!("🐙 Creating GitHub App...");
    let app_info = create_github_app_for_agent(
        &persona,
        purpose,
        github_org,
        params.get("permissions").map(|p| serde_json::from_value(p.clone())).transpose()?
    ).await?;

    match &app_info {
        GitHubAppInfo::Created { app_id, .. } => {
            println!("✅ Created GitHub App with ID: {}", app_id);
        }
        GitHubAppInfo::Pending { creation_url, .. } => {
            println!("⏳ Manual step required. Visit: {}", creation_url);
            // Wait for callback or timeout
            let app_info = wait_for_app_creation(&app_info).await?;
        }
    }

    // Step 4: Create K8s resources
    println!("☸️ Creating Kubernetes resources...");
    create_k8s_resources(&persona, &app_info, &prompts, namespace).await?;
    println!("✅ Created ConfigMap and Secrets");

    // Step 5: Deploy (optional)
    if should_deploy {
        println!("🚀 Deploying agent...");
        deploy_agent(&persona, namespace).await?;
        println!("✅ Agent deployed successfully");
    }

    // Return complete information
    Ok(json!({
        "success": true,
        "agent": {
            "name": persona.name,
            "archetype": persona.archetype,
            "traits": persona.traits,
            "capabilities": capabilities,
        },
        "github_app": {
            "id": app_info.app_id,
            "name": format!("{}-AI-Agent", persona.name),
            "installation_url": format!("https://github.com/apps/{}-ai-agent/installations/new",
                persona.name.to_lowercase()),
        },
        "kubernetes": {
            "namespace": namespace,
            "config_map": format!("{}-config", persona.name.to_lowercase()),
            "secret": format!("{}-github-app", persona.name.to_lowercase()),
        },
        "deployment": {
            "deployed": should_deploy,
            "status": if should_deploy { "running" } else { "ready" },
        },
        "next_steps": if !should_deploy {
            vec![
                format!("Install the GitHub App: https://github.com/apps/{}-ai-agent/installations/new",
                    persona.name.to_lowercase()),
                format!("Deploy the agent: mcp_cto_deploy_agent --name={}", persona.name),
                format!("View agent config: kubectl get cm {}-config -n {} -o yaml",
                    persona.name.to_lowercase(), namespace),
            ]
        } else {
            vec![
                format!("View agent logs: kubectl logs -l app={} -n {}",
                    persona.name.to_lowercase(), namespace),
                format!("Check agent status: kubectl get coderun -n {} | grep {}",
                    namespace, persona.name.to_lowercase()),
            ]
        }
    }))
}








```



## Usage Examples

### Example 1: Create Security Analyst





```typescript
// MCP call from Cursor/IDE
await mcp.call("create_agent_persona", {
  purpose: "Security vulnerability detection, code auditing, and threat analysis",
  personality_hints: {
    archetype: "detective",
    tone: "analytical",
    quirks: ["uses security metaphors", "always considers threat vectors"]
  },
  deploy: true
});








```

Response:




```json
{
  "success": true,
  "agent": {
    "name": "Cipher",
    "archetype": "Security Detective",
    "traits": [
      "Methodical investigation approach",
      "Pattern recognition expert",
      "Proactive threat hunter"
    ],
    "capabilities": ["security_audit", "vulnerability_scan", "threat_analysis"]
  },
  "github_app": {
    "id": "987654",
    "name": "Cipher-AI-Agent",
    "installation_url": "https://github.com/apps/cipher-ai-agent/installations/new"
  },
  "kubernetes": {
    "namespace": "cto",
    "config_map": "cipher-config",
    "secret": "cipher-github-app"
  },
  "deployment": {
    "deployed": true,
    "status": "running"
  }
}








```

### Example 2: Create Documentation Expert





```typescript
await mcp.call("create_agent_persona", {
  purpose: "Technical documentation, API references, and developer guides",
  personality_hints: {
    archetype: "librarian",
    quirks: ["loves organization", "provides examples"]
  }
});








```

Generated Agent:
- Name: "Lexie"
- Archetype: "Knowledge Librarian"
- Traits: ["Organized", "Clear communicator", "Detail-oriented"]

### Example 3: Performance Optimizer





```typescript
await mcp.call("create_agent_persona", {
  purpose: "Performance optimization and bottleneck detection",
  capabilities: ["profiling", "optimization", "benchmarking"],
  github_org: "5dlabs",
  namespace: "cto"
});








```

Generated Agent:
- Name: "Blaze"
- Archetype: "Performance Engineer"
- Traits: ["Speed-focused", "Efficiency expert", "Metrics-driven"]

## Testing the Implementation

### Unit Tests





```rust


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persona_generation() {
        let persona = generate_agent_character(
            "Testing and quality assurance",
            Some(PersonalityHints {
                archetype: Some("engineer".to_string()),
                tone: Some("friendly".to_string()),
                quirks: Some(vec!["thorough".to_string()]),
            })
        ).await.unwrap();

        assert!(!persona.name.is_empty());
        assert!(persona.archetype.contains("engineer") ||
                persona.archetype.contains("Engineer"));
        assert!(!persona.traits.is_empty());
    }

    #[tokio::test]
    async fn test_permission_inference() {
        let perms = infer_permissions_from_purpose(
            "security vulnerability detection"
        );

        assert_eq!(perms.security_events, Some("write".to_string()));
        assert_eq!(perms.contents, Some("read".to_string()));
    }

    #[tokio::test]
    async fn test_prompt_generation() {
        let persona = AgentPersona {
            name: "TestBot".to_string(),
            archetype: "Tester".to_string(),
            traits: vec!["thorough".to_string()],
            communication_style: CommunicationStyle::default(),
            avatar_description: "A robot".to_string(),
        };

        let prompts = generate_system_prompts(
            &persona,
            "testing",
            &vec!["code_review".to_string()]
        ).await.unwrap();

        assert!(prompts.base.contains("TestBot"));
        assert!(prompts.task_specific.contains_key("code_review"));
    }
}








```

## Error Handling





```rust


#[derive(Debug, thiserror::Error)]
enum PersonaCreationError {
    #[error("Failed to generate persona: {0}")]
    GenerationFailed(String),

    #[error("GitHub App creation failed: {0}")]
    GitHubAppFailed(String),

    #[error("Kubernetes resource creation failed: {0}")]
    K8sResourceFailed(String),

    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),

    #[error("Timeout waiting for app creation")]
    CreationTimeout,
}








```

## Monitoring & Observability





```rust
// Add metrics for tracking
static PERSONA_CREATIONS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "agent_persona_creations_total",
        "Total number of agent personas created"
    ).unwrap()
});

static CREATION_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "agent_persona_creation_duration_seconds",
        "Time taken to create an agent persona"
    ).unwrap()
});








```

## Future Enhancements



1. **Avatar Generation Integration**
   ```rust
   async fn generate_avatar(description: &str) -> Result<Vec<u8>> {
       // Call DALL-E or Stable Diffusion API
       let image_bytes = image_generator
           .generate(description)
           .await?;
       Ok(image_bytes)
   }







```



2. **Persona Learning**
   ```rust
   async fn evolve_persona(
       persona: &mut AgentPersona,
       performance_metrics: &Metrics
   ) -> Result<()> {
       // Adjust traits based on success patterns
       if performance_metrics.accuracy > 0.9 {
           persona.traits.push("High accuracy achiever".to_string());
       }
       Ok(())
   }







```



3. **Team Composition**
   ```rust
   async fn suggest_team_member(
       existing_team: &[AgentPersona],
       need: &str
   ) -> Result<PersonalityHints> {
       // Analyze team gaps and suggest complementary persona
       let gaps = analyze_team_gaps(existing_team);
       let hints = generate_complementary_hints(gaps, need);
       Ok(hints)
   }







```
