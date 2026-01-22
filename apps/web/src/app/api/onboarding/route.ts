import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';
import { headers } from 'next/headers';

// Onboarding state types (matching Rust implementation)
interface OnboardingState {
  state: string;
  [key: string]: unknown;
}

interface OnboardingAction {
  action: string;
  [key: string]: unknown;
}

interface OnboardingResponse {
  state: OnboardingState;
  message: string;
  actions: Array<{
    action_type: string;
    label: string;
    value: string;
    options?: Array<{ label: string; value: string }>;
  }>;
  progress: number;
}

// Simple in-memory state store (would be replaced with DB in production)
const stateStore = new Map<string, OnboardingState>();

export async function GET() {
  const session = await auth.api.getSession({
    headers: await headers(),
  });

  if (!session) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const state = stateStore.get(session.user.id) || { state: 'welcome' };

  return NextResponse.json({ state });
}

export async function POST(request: NextRequest) {
  const session = await auth.api.getSession({
    headers: await headers(),
  });

  if (!session) {
    return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
  }

  const action: OnboardingAction = await request.json();
  const currentState = stateStore.get(session.user.id) || { state: 'welcome' };

  // Process state transition
  const response = processTransition(currentState, action, session.user);

  // Store new state
  stateStore.set(session.user.id, response.state);

  return NextResponse.json(response);
}

function processTransition(
  currentState: OnboardingState,
  action: OnboardingAction,
  user: { id: string; name: string; email: string }
): OnboardingResponse {
  switch (currentState.state) {
    case 'welcome':
      if (action.action === 'start') {
        return {
          state: { state: 'github_connect' },
          message:
            "Great! Let's connect your GitHub account so our agents can access your repositories.",
          actions: [
            {
              action_type: 'button',
              label: 'Connect GitHub',
              value: 'connect_github',
            },
          ],
          progress: 10,
        };
      }
      break;

    case 'github_connect':
      if (action.action === 'github_connected') {
        // Mock repos - in production, fetch from GitHub API
        const repos = [
          {
            label: 'acme/api (Rust)',
            value: 'acme/api',
          },
          {
            label: 'acme/web (React/Next.js)',
            value: 'acme/web',
          },
          {
            label: 'acme/mobile (React Native)',
            value: 'acme/mobile',
          },
        ];

        return {
          state: {
            state: 'repo_selection',
            installation_id: action.installation_id,
            available_repos: repos,
          },
          message: 'GitHub connected! Which repositories should our agents work on?',
          actions: [
            {
              action_type: 'select',
              label: 'Select Repositories',
              value: 'select_repos',
              options: repos,
            },
          ],
          progress: 30,
        };
      }
      break;

    case 'repo_selection':
      if (action.action === 'select_repos') {
        return {
          state: {
            state: 'api_key_entry',
            selected_repos: action.repos,
            provider: 'anthropic',
          },
          message:
            'Now I need an API key to power our AI agents. We recommend Anthropic (Claude) for the best experience.\n\nYour API key is stored securely and never logged or exposed.',
          actions: [
            {
              action_type: 'input',
              label: 'Enter Anthropic API Key',
              value: 'anthropic_key',
            },
          ],
          progress: 50,
        };
      }
      break;

    case 'api_key_entry':
      if (action.action === 'submit_key') {
        // Detect stack from repos (mocked)
        const recommendedAgents = ['Rex', 'Cleo', 'Tess', 'Bolt'];

        return {
          state: {
            state: 'agent_selection',
            selected_repos: currentState.selected_repos,
            api_key_configured: true,
            detected_stack: {
              primary_language: 'Rust',
              frameworks: ['axum', 'tokio', 'sqlx'],
              recommended_agents: recommendedAgents,
            },
          },
          message: `API key validated! Based on your Rust codebase, I recommend:\n\n**Recommended Squad:**\n• ${recommendedAgents.join('\n• ')}`,
          actions: [
            {
              action_type: 'button',
              label: 'Use Recommended Squad',
              value: 'use_recommended',
            },
            {
              action_type: 'button',
              label: 'Customize Squad',
              value: 'customize',
            },
          ],
          progress: 70,
        };
      }
      break;

    case 'agent_selection':
      if (action.action === 'select_agents' || action.action === 'use_recommended') {
        return {
          state: {
            state: 'provisioning',
            step: 'creating_namespace',
          },
          message: 'Setting up your workspace... Creating isolated environment.',
          actions: [],
          progress: 85,
        };
      }
      break;

    case 'provisioning':
      // This would be triggered by backend when provisioning completes
      return {
        state: {
          state: 'complete',
          tenant_id: `tenant-${user.id.slice(0, 8)}`,
          namespace: `tenant-${user.id.slice(0, 8)}`,
        },
        message: `Your workspace is ready! Here's what I've set up:\n\n✓ Namespace: tenant-${user.id.slice(0, 8)}\n✓ Agents: Rex, Cleo, Tess, Bolt\n✓ GitHub App installed\n✓ Secrets configured\n\nYou're all set! Head to the dashboard to create your first project.`,
        actions: [
          {
            action_type: 'button',
            label: 'Go to Dashboard',
            value: 'dashboard',
          },
          {
            action_type: 'button',
            label: 'Create First Project',
            value: 'create_project',
          },
        ],
        progress: 100,
      };
  }

  // Default response
  return {
    state: currentState,
    message: "I didn't understand that. Could you try again?",
    actions: [],
    progress: 0,
  };
}
