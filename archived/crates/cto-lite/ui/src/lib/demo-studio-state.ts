import type { AgentUiConfig, ProjectRecord, StudioState } from '@/lib/tauri'

/** Sample PRD markdown for dev browser preview (split / preview modes). */
export const SAMPLE_PRD_MARKDOWN = `# Sigma 1 — CTO Lite PRD (sample)

## Product goal

Ship a **single** Morgan workspace where chat, voice, video, and project context stay aligned.

## User stories

- As a builder, I open **Projects** and edit a PRD with live markdown preview.
- As a user, I switch between Split, Markdown, and Preview without losing state.

## Technical notes

| Area        | Choice              |
|------------|---------------------|
| Shell      | Tauri + React       |
| Editor     | CodeMirror (GFM)    |
| Preview    | react-markdown      |

## Snippet

\`\`\`text
session → LiveKit room → Morgan + agents
\`\`\`

## Links

- [CTO repo](https://github.com/5dlabs/cto)

---

_This block is seeded for UI preview; replace with your real PRD._
`

function demoMorgan(): AgentUiConfig {
  return {
    id: 'morgan',
    displayName: 'Morgan',
    role: 'Intake & Orchestrator',
    summary: 'Primary conversational agent across chat, voice, and video.',
    avatarLabel: 'MO',
    enabled: true,
    skills: ['openclaw', 'acp', 'memory', 'workflow-routing'],
    capabilities: ['Conversation', 'Task decomposition', 'Delegation', 'Project context'],
    tools: ['codex', 'claude', 'github', 'linear'],
    systemPrompt:
      'You are Morgan, the crisp and practical orchestrator for CTO. Keep responses concise, collaborative, and execution-oriented.',
    heartbeatEvery: '5m',
    model: 'anthropic/claude-sonnet-4-20250514',
  }
}

function demoProject(id: string, name: string, prdTitle: string, prdContent: string): ProjectRecord {
  return {
    id,
    name,
    summary: 'Demo project for CTO Lite UI.',
    repository: null,
    prdTitle,
    prdContent,
    workflowSummary: 'Morgan intake → agent execution → review → apply.',
    workflowNotes: 'Track PRD decomposition and local runtime checkpoints here.',
    configNotes: 'Local OpenClaw-backed stack.',
  }
}

/** Used when \`studio_get_state\` is unavailable (e.g. Vite without Tauri). */
export function getDemoStudioState(): StudioState {
  return {
    selectedProjectId: 'cto-core',
    projects: [
      demoProject('cto-core', 'Sigma 1', 'Sigma 1 — sample PRD', SAMPLE_PRD_MARKDOWN),
      demoProject(
        'morgan-avatar',
        'Morgan Avatar',
        'Avatar layer',
        '## Morgan Avatar\n\nVoice and video over the shared session.\n'
      ),
    ],
    agents: [demoMorgan()],
  }
}
