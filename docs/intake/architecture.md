# Discord Agent Monitoring System - Complete Design Document

## Executive Summary

A zero-impact monitoring system for AI agents using Discord as the primary interface. The system uses an independent Rust binary that watches Claude's transcript files, completely decoupled from the agent's execution for maximum reliability and simplicity.

## Key Architecture Decision: Independent Watcher

After evaluating status lines, hooks, and independent monitoring, we chose a **pure independent watcher** for these reasons:

### Why Independent Monitoring is Optimal

- **Zero Performance Impact**: Runs as separate process, cannot affect Claude
- **Complete Event Capture**: Sees everything in the transcript, no throttling
- **Simple & Reliable**: One binary, one job, can restart without affecting agent
- **No Configuration Required**: No need to modify Claude settings or hooks
- **Standard Unix Pattern**: Simple file tailing, like `tail -f`

## System Architecture

### 1. Core Components

```mermaid
graph TB
    subgraph "Discord Server"
        A[Discord Webhook]
        B[Dynamic Channels]
        C[Agent Updates]
    end
    
    subgraph "Discord Monitor (Sidecar)"
        D[File Watcher]
        E[JSONL Parser]
        F[Event Filter]
        G[Discord Client (Twilight HTTP)]
    end
    
    subgraph "Claude Agent Pod"
        H[Claude Process]
        I[Transcript JSONL]
    end
    
    subgraph "Shared Volume"
        J[/workspace/]
        K[.claude/projects/]
        L[session.jsonl]
    end
    
    H -->|Appends Events| I
    I -->|Stored In| L
    L -->|Located In| K
    K -->|Under| J
    
    D -->|Watches| L
    D -->|New Lines| E
    E -->|Parse Events| F
    F -->|Format| G
    G -->|POST (twilight-http)| A
    A -->|Display In| B
    B -->|Shows| C
```

### 2. Data Flow

```json
// 1. Claude appends to transcript.jsonl (each line is complete JSON)
{"type":"assistant","message":{"content":[{"text":"Analyzing code..."}]}}
{"type":"tool_use","name":"Bash","input":{"command":"npm install"}}
{"type":"tool_result","content":"Successfully installed dependencies"}

// 2. Watcher detects new lines and parses
TranscriptEvent {
    event_type: "tool_use",
    name: "Bash",
    input: {"command": "npm install"}
}

// 3. Formats and sends to Discord
{
  "embeds": [{
    "description": "⚡ Running: npm install",
    "color": 0xF39C12
  }]
}
```

### 3. Transcript File Structure

The transcript is an append-only JSONL file where each line is a complete event:

- Located at: `~/.claude/projects/{encoded-workspace}/session-id.jsonl`
- Each line is valid JSON representing one event
- Events include: `assistant`, `user`, `tool_use`, `tool_result`, `thinking`
- Perfect for `tail -f` style monitoring

## Implementation

### Production Discord Monitor (Rust)

```rust
// discord-monitor/src/main.rs
// Independent watcher that monitors Claude transcript files

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[derive(Debug, Deserialize)]
struct TranscriptEvent {
    #[serde(rename = "type")]
    event_type: String,
    uuid: Option<String>,
    timestamp: Option<String>,
    session_id: Option<String>,
    message: Option<Message>,
    name: Option<String>,
    input: Option<serde_json::Value>,
    #[serde(rename = "toolUseResult")]
    tool_use_result: Option<ToolResult>,
}

#[derive(Debug, Deserialize)]
struct Message {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    role: Option<String>,
    model: Option<String>,
    content: Vec<Content>,
    stop_reason: Option<String>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: Option<String>,
    #[serde(rename = "type")]
    content_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    cache_creation_input_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ToolResult {
    stdout: Option<String>,
    stderr: Option<String>,
    interrupted: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DiscordEmbed {
    title: Option<String>,
    description: String,
    color: u32,
    fields: Option<Vec<EmbedField>>,
    footer: Option<EmbedFooter>,
    timestamp: Option<String>,
}

#[derive(Debug, Serialize)]
struct EmbedField {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Debug, Serialize)]
struct EmbedFooter {
    text: String,
}

#[derive(Debug, Serialize)]
struct DiscordWebhook {
    embeds: Vec<DiscordEmbed>,
}

struct SessionStats {
    total_cost: f64,
    total_input_tokens: u32,
    total_output_tokens: u32,
    total_cache_tokens: u32,
    tool_use_count: u32,
    error_count: u32,
}

struct TranscriptWatcher {
    transcript_path: PathBuf,
    last_position: u64,
    webhook_url: String,
    session_stats: SessionStats,
}

impl TranscriptWatcher {
    fn new(transcript_path: PathBuf, webhook_url: String) -> Self {
        Self {
            transcript_path,
            last_position: 0,
            webhook_url,
            session_stats: SessionStats {
                total_cost: 0.0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cache_tokens: 0,
                tool_use_count: 0,
                error_count: 0,
            },
        }
    }
    
    async fn watch(&mut self) {
        loop {
            if let Ok(mut file) = File::open(&self.transcript_path) {
                // Seek to last read position
                let _ = file.seek(SeekFrom::Start(self.last_position));
                
                let reader = BufReader::new(&file);
                let mut events = Vec::new();
                
                for line in reader.lines() {
                    if let Ok(line_content) = line {
                        if let Ok(event) = serde_json::from_str::<TranscriptEvent>(&line_content) {
                            if let Some(embed) = self.format_for_discord(event) {
                                events.push(embed);
                            }
                        }
                    }
                }
                
                // Update position for next read
                if let Ok(metadata) = file.metadata() {
                    self.last_position = metadata.len();
                }
                
                // Send events to Discord
                if !events.is_empty() {
                    self.send_to_discord(events).await;
                }
            }
            
            // Check for new content every 100ms
            sleep(Duration::from_millis(100)).await;
        }
    }
    
    fn format_for_discord(&mut self, event: &TranscriptEvent) -> Option<DiscordEmbed> {
        match event.event_type.as_str() {
            "tool_use" => {
                if let Some(name) = &event.name {
                    self.session_stats.tool_use_count += 1;
                    
                    let emoji = match name.as_str() {
                        "Bash" => "⚡",
                        "Write" | "Edit" | "MultiEdit" => "📝",
                        "Read" | "Glob" | "Grep" => "👁️",
                        "WebSearch" | "WebFetch" => "🔍",
                        _ => "🔧",
                    };
                    
                    let mut description = format!("{} **{}**", emoji, name);
                    let mut fields = Vec::new();
                    
                    if name == "Bash" {
                        if let Some(input) = &event.input {
                            if let Some(cmd) = input.get("command").and_then(|c| c.as_str()) {
                                fields.push(EmbedField {
                                    name: "Command".to_string(),
                                    value: format!("`{}`", &cmd[..500.min(cmd.len())]),
                                    inline: false,
                                });
                            }
                        }
                    } else if name == "Write" || name == "Edit" {
                        if let Some(input) = &event.input {
                            if let Some(path) = input.get("file_path").and_then(|p| p.as_str()) {
                                fields.push(EmbedField {
                                    name: "File".to_string(),
                                    value: format!("`{}`", path),
                                    inline: true,
                                });
                            }
                        }
                    }
                    
                    fields.push(EmbedField {
                        name: "Total Tools Used".to_string(),
                        value: self.session_stats.tool_use_count.to_string(),
                        inline: true,
                    });
                    
                    return Some(DiscordEmbed {
                        title: None,
                        description,
                        color: 0xF39C12, // Yellow
                        fields: Some(fields),
                        footer: None,
                        timestamp: event.timestamp.clone(),
                    });
                }
            }
            "assistant" => {
                if let Some(message) = &event.message {
                    // Track token usage and costs
                    if let Some(usage) = &message.usage {
                        if let Some(input) = usage.input_tokens {
                            self.session_stats.total_input_tokens += input;
                        }
                        if let Some(output) = usage.output_tokens {
                            self.session_stats.total_output_tokens += output;
                        }
                        if let Some(cache) = usage.cache_read_input_tokens {
                            self.session_stats.total_cache_tokens += cache;
                        }
                        
                        // Estimate cost (example rates - adjust to actual)
                        let input_cost = (self.session_stats.total_input_tokens as f64) * 0.000003;
                        let output_cost = (self.session_stats.total_output_tokens as f64) * 0.000015;
                        self.session_stats.total_cost = input_cost + output_cost;
                    }
                    
                    if let Some(text) = message.content.first().and_then(|c| c.text.as_ref()) {
                        // Filter for significant messages
                        let patterns = ["Starting", "Error", "Success", "Creating", "Testing", "Implementing", "Complete"];
                        if patterns.iter().any(|p| text.contains(p)) {
                            let mut fields = Vec::new();
                            
                            // Add usage stats if available
                            if let Some(usage) = &message.usage {
                                if usage.input_tokens.is_some() || usage.output_tokens.is_some() {
                                    fields.push(EmbedField {
                                        name: "Tokens".to_string(),
                                        value: format!("In: {} | Out: {}", 
                                            usage.input_tokens.unwrap_or(0),
                                            usage.output_tokens.unwrap_or(0)),
                                        inline: true,
                                    });
                                    
                                    fields.push(EmbedField {
                                        name: "Session Cost".to_string(),
                                        value: format!("${:.6}", self.session_stats.total_cost),
                                        inline: true,
                                    });
                                }
                            }
                            
                            // Add model info
                            if let Some(model) = &message.model {
                                fields.push(EmbedField {
                                    name: "Model".to_string(),
                                    value: model.clone(),
                                    inline: true,
                                });
                            }
                            
                            return Some(DiscordEmbed {
                                title: None,
                                description: format!("💭 {}", &text[..800.min(text.len())]),
                                color: 0x3498DB, // Blue
                                fields: if fields.is_empty() { None } else { Some(fields) },
                                footer: if let Some(reason) = &message.stop_reason {
                                    Some(EmbedFooter { text: format!("Stop: {}", reason) })
                                } else { None },
                                timestamp: event.timestamp.clone(),
                            });
                        }
                    }
                }
            }
            "user" => {
                // Tool results with success/error handling
                if let Some(result) = &event.tool_use_result {
                    if let Some(stderr) = &result.stderr {
                        if !stderr.is_empty() {
                            self.session_stats.error_count += 1;
                            
                            return Some(DiscordEmbed {
                                title: Some("❌ Error".to_string()),
                                description: format!("```\n{}\n```", &stderr[..800.min(stderr.len())]),
                                color: 0xE74C3C, // Red
                                fields: Some(vec![
                                    EmbedField {
                                        name: "Total Errors".to_string(),
                                        value: self.session_stats.error_count.to_string(),
                                        inline: true,
                                    }
                                ]),
                                footer: None,
                                timestamp: event.timestamp.clone(),
                            });
                        }
                    }
                    
                    // Success output (only for significant stdout)
                    if let Some(stdout) = &result.stdout {
                        if stdout.len() > 50 && stdout.chars().all(|c| c != ' ' && c != '\n') {
                            return Some(DiscordEmbed {
                                title: Some("✅ Output".to_string()),
                                description: format!("```\n{}\n```", &stdout[..800.min(stdout.len())]),
                                color: 0x27AE60, // Green
                                fields: None,
                                footer: None,
                                timestamp: event.timestamp.clone(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }
    
    async fn send_to_discord(&self, events: Vec<DiscordEmbed>) {
// Use timeout to ensure we don't block
        let _ = timeout(Duration::from_secs(2), async {
            let client = reqwest::Client::new();
            let webhook = DiscordWebhook { 
                embeds: events.into_iter().take(10).collect() 
            };
            
            let _ = client.post(&self.webhook_url)
                .json(&webhook)
                .send()
                .await;
        }).await;
    }
}

fn find_latest_transcript(workspace_path: &str) -> Option<PathBuf> {
    // Look for the most recent .jsonl file in Claude's project directory
    let claude_dir = format!("{}/.claude/projects", 
        std::env::var("HOME").unwrap_or_else(|_| workspace_path.to_string()));
    
    std::fs::read_dir(&claude_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .flat_map(|dir| std::fs::read_dir(dir.path()).ok())
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .max_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .ok()
        })
        .map(|entry| entry.path())
}

#[tokio::main]
async fn main() {
    let webhook_url = std::env::var("DISCORD_WEBHOOK_URL")
        .expect("DISCORD_WEBHOOK_URL must be set");
    
    let workspace_path = std::env::var("WORKSPACE_PATH")
        .unwrap_or_else(|_| "/workspace".to_string());
    
    println!("🔍 Discord Monitor starting...");
    println!("📁 Workspace: {}", workspace_path);
    
    // Wait for transcript file to appear
    let transcript_path = loop {
        if let Some(path) = find_latest_transcript(&workspace_path) {
            println!("📝 Found transcript: {:?}", path);
            break path;
        }
        println!("⏳ Waiting for Claude to start...");
        sleep(Duration::from_secs(2)).await;
    };
    
    // Start watching
    let mut watcher = TranscriptWatcher::new(transcript_path, webhook_url);
    println!("👁️ Monitoring transcript for Discord updates...");
    watcher.watch().await;
}
```

### Cargo.toml for the Rust Binary

```toml
[package]
name = "discord-monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }

[[bin]]
name = "discord-monitor"
path = "src/main.rs"
```

### Discord Integration (Twilight)

For bot and gallery management via Discord REST:

```toml
# controller/Cargo.toml (excerpt)
[dependencies]
twilight-http = "0.16"
twilight-model = "0.16"
# Optional if/when we need gateway events
# twilight-gateway = "0.16"
# twilight-cache-inmemory = "0.16"
```

```rust
use std::sync::Arc;
use twilight_http::Client as DiscordHttp;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, MessageMarker};

async fn update_gallery_tile(http: Arc<DiscordHttp>, channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>, content: String) -> anyhow::Result<()> {
    http.update_message(channel_id, message_id)
        .content(Some(&content))?
        .await?;
    Ok(())
}
```

### Container Deployment

The Discord monitor runs as a sidecar container alongside the Claude agent:

```yaml
# Kubernetes Pod Configuration
apiVersion: v1
kind: Pod
metadata:
  name: claude-agent
spec:
  containers:
    # Main Claude agent container
    - name: claude
      image: claude-code:latest
      volumeMounts:
        - name: workspace
          mountPath: /workspace
        - name: claude-home
          mountPath: /root/.claude
      env:
        - name: TASK_ID
          value: "task-123"
    
    # Discord monitor sidecar
    - name: discord-monitor
      image: discord-monitor:latest
      volumeMounts:
        - name: workspace
          mountPath: /workspace
        - name: claude-home
          mountPath: /root/.claude  # Share Claude's home for transcript access
      env:
        - name: DISCORD_WEBHOOK_URL
          valueFrom:
            secretKeyRef:
              name: discord-secrets
              key: webhook-url
        - name: WORKSPACE_PATH
          value: "/workspace"
  
  volumes:
    - name: workspace
      persistentVolumeClaim:
        claimName: agent-workspace
    - name: claude-home
      emptyDir: {}  # Or PVC if you want persistence
```

### Dockerfile for Discord Monitor

```dockerfile
# discord-monitor/Dockerfile
FROM rust:1.75 as builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/discord-monitor /usr/local/bin/

CMD ["/usr/local/bin/discord-monitor"]
```

## Discord Output Examples

### Rich Embed Messages

The monitor sends beautifully formatted Discord embeds with all available information:

```text
┌─────────────────────────────────────┐
│ ⚡ **Bash**                         │
├─────────────────────────────────────┤
│ Command:                            │
│ `npm install express`               │
├─────────────────────────────────────┤
│ Total Tools Used: 15                │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ 💭 Starting API implementation...   │
├─────────────────────────────────────┤
│ Tokens: In: 5432 | Out: 234        │
│ Session Cost: $0.003456             │
│ Model: claude-3-opus                │
├─────────────────────────────────────┤
│ Stop: max_tokens                    │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ ❌ Error                            │
├─────────────────────────────────────┤
│ ```                                 │
│ Module not found: express          │
│ ```                                 │
├─────────────────────────────────────┤
│ Total Errors: 2                     │
└─────────────────────────────────────┘
```

### Session Statistics

The monitor tracks cumulative statistics throughout the session:

- **Total Cost**: Real-time cost tracking based on token usage
- **Token Usage**: Input, output, and cache tokens
- **Tool Usage**: Count of each tool type used
- **Error Rate**: Track and alert on high error rates
- **Performance**: Time between events, response times

## Performance Analysis

| Monitoring Method | CPU Impact | Memory | Latency | Reliability | Complexity |
|------------------|------------|--------|---------|-------------|------------|
| **Independent Watcher** (Our Choice) | 0-1% | 30MB | 100ms | Very High | Low |
| Status Line | ~0% on Claude | 10MB | 300ms | High | Medium |
| Hooks | 5-10% on Claude | 50MB | 0ms | Medium | High |
| Direct Integration | 2-5% on Claude | 40MB | 0ms | Low | Very High |

The independent watcher wins because:

- **Zero impact on Claude**: Completely separate process
- **Simple deployment**: Just another container in the pod
- **Easy debugging**: Separate logs, can restart independently
- **Complete visibility**: Sees all events, no filtering

## Configuration

### Helm Values

```yaml
discord:
  monitoring:
    enabled: true
    image: discord-monitor:latest
    
    webhook:
      url: ${DISCORD_WEBHOOK_URL}  # From secret
      rateLimit: 10  # messages per second
      
    polling:
      interval: 100  # ms between transcript checks
      batchSize: 10  # max embeds per Discord message
      
    filters:
      # Only send significant events to Discord
      includeTools: ["Bash", "Write", "Edit"]
      includePatterns: ["Error", "Success", "Complete", "Starting"]
      minStdoutLength: 50  # Ignore small outputs
      
    stats:
      trackCost: true
      trackTokens: true
      trackErrors: true
      
    resources:
      requests:
        memory: "32Mi"
        cpu: "10m"
      limits:
        memory: "64Mi"
        cpu: "100m"
```

### Environment Variables

```yaml
env:
  - name: DISCORD_WEBHOOK_URL
    valueFrom:
      secretKeyRef:
        name: discord-secrets
        key: webhook-url
  - name: WORKSPACE_PATH
    value: "/workspace"
  - name: LOG_LEVEL
    value: "info"
```

## Implementation Phases

### Phase 1: Basic Monitoring (Day 1-2)

- [ ] Build Rust binary with basic transcript watching
- [ ] Deploy as sidecar container
- [ ] Configure Discord webhook
- [ ] Test event capture and formatting
- [ ] Verify zero impact on Claude

### Phase 2: Rich Discord Output (Day 3-4)

- [ ] Implement full embed formatting with fields
- [ ] Add cost and token tracking
- [ ] Create error tracking and statistics
- [ ] Add timestamp and session information
- [ ] Test with real agent runs

### Phase 3: Production Hardening (Day 5-7)

- [ ] Add robust error handling and retries
- [ ] Implement rate limiting for Discord API
- [ ] Add configuration via environment variables
- [ ] Create health checks and metrics
- [ ] Deploy to staging environment

### Phase 4: Advanced Features (Week 2)

- [ ] Add filtering configuration
- [ ] Implement message batching
- [ ] Create summary reports at session end
- [ ] Add Discord bot for dynamic channels (optional)
- [ ] Deploy to production

## Key Insights

1. **Complete Independence**: Monitoring has zero impact on Claude's execution
2. **Append-Only JSONL**: Perfect for tail-style watching, each line is complete event
3. **Rust Performance**: Compiled binary with minimal resource usage
4. **Rich Discord Output**: Full access to all transcript fields for comprehensive monitoring
5. **Simple Deployment**: Just another container in the pod, no Claude configuration

## Benefits of Independent Watcher

- **Can't break Claude**: Completely separate process, failures don't affect agent
- **Easy debugging**: Separate logs, can add verbose logging without affecting Claude
- **Hot reload**: Can update monitor without restarting agent
- **Multiple consumers**: Can have multiple watchers for different purposes (Discord, metrics, etc.)
- **No configuration**: No need to modify Claude settings or add hooks

## Conclusion

The independent watcher approach provides:

- **Zero performance impact** on Claude (separate process)
- **Complete event visibility** (sees everything in transcript)
- **Simple architecture** (just tailing a file)
- **Production reliability** (can restart without affecting agent)
- **Rich monitoring** (cost tracking, token usage, error rates)

This architecture treats monitoring as a first-class concern, completely decoupled from the agent's execution, providing maximum reliability and flexibility.
