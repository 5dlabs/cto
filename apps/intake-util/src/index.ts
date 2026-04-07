#!/usr/bin/env bun
/**
 * intake-util CLI - deterministic operations for the intake pipeline.
 *
 * Subcommands:
 *   write-files              --base-path <dir> --type <docs|prompts|workflows>
 *   tally                    --ballots-json <file>
 *   fan-out                  --prompt <path> --schema <path> --provider <p> --model <m>
 *   validate                 --type <type> [--task-ids <json>] [--strict]
 *   sync-linear init         --project-name <name> --team-id <id> --prd-content <file>
 *   sync-linear issues       --project-id <id> --prd-issue-id <id> --team-id <id>
 *   sync-linear rewrite-urls --project-id <id> --old-base-url <url> --new-base-url <url>
 *   sync-linear github-sync  --project-id <id> --repo <owner/repo> --branch <branch>
 *   parse-decision-points    (stdin: {content, speaker})
 *   bridge-notify            --from <agent> --to <agent> [--metadata <json>]
 *   bridge-elicitation       --session-id <id> --decision-id <id> --vote-result <json>
 *   tally-decision-votes     (stdin: DecisionVote[])
 *   generate-workflows       --config <path> [--scaffolds-json <file>] [--repository-url <url>]
 *   linear-activity          --session-id <id> --type <type> --body <text> [--ephemeral] [--signal select] [--signal-metadata <json>]
 *   linear-plan              --session-id <id> [--plan-json <json>|--step <step-id>|--plan <json>]
 *   linear-session update-urls --session-id <id> [--add-url "label=url"] [--set-urls <json>]
 *   register-run             --run-id <id> --agent <name> [--issue-id <id>] [--session-key <k>] [--linear-session-id <id>]
 *   deregister-run           --run-id <id>
 *   invoke-agent             --mode <subagent|a2a> --agent <name> [--prompt-file <path>]
 *   classify-output          --cli <claude|codex|openclaw> [--intermediate]
 *
 * All subcommands also accept JSON on stdin when file args are omitted.
 */

import { writeFiles } from './write-files';
import { tallyVotes } from './tally';
import { createProjectAndPrdIssue, syncTaskIssues, rewriteProjectUrls, syncGitHubIssues } from './sync-linear';
import { fanOut } from './fan-out';
import { validateDocs, validatePrompts, validateWorkflows, validateGeneric } from './validate';
import { parseDecisionPoints } from './parse-decision-points';
import { bridgeNotify } from './bridge-notify';
import { bridgeElicitation } from './bridge-elicitation';
import { tallyDecisionVotes } from './tally-decision-votes';
import { generateWorkflows } from './generate-workflow';
import { linearActivity } from './linear-activity';
import { linearPlan, resolvePlanSteps } from './linear-plan';
import { linearSessionUpdateUrls } from './linear-session';
import { registerRun, deregisterRun } from './run-registry-client';
import { invokeAgent } from './invoke-agent';
import { classifyCliOutput } from './classify-output';
import { designReview } from './design-review';
import * as fs from 'fs';

/** Read all of stdin as a string (works in both Bun and Node). */
async function readStdin(): Promise<string> {
  if (typeof globalThis.Bun !== 'undefined') {
    return new Response(Bun.stdin.stream()).text();
  }
  // Node.js fallback
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) {
    chunks.push(Buffer.from(chunk));
  }
  return Buffer.concat(chunks).toString('utf-8');
}

function usage(): never {
  console.error(`Usage: intake-util <subcommand> [options]

Subcommands:
  write-files      Write LLM-generated JSON output to disk
    --base-path <dir>    Output directory (required)
    --type <type>        File type: "docs", "prompts", or "workflows" (required)

  tally            Tally votes from model ballots
    --ballots-json <file> Path to VoteBallot[] JSON (or pipe via stdin)

  fan-out          Run parallel LLM invocations per task (stdin: tasks JSON array)
    --prompt <path>      System prompt file path (required)
    --schema <path>      Per-item output schema path (required)
    --context <json>     Shared context JSON (scaffolds, codebase, infra)
    --provider <p>       LLM provider (required)
    --model <m>          LLM model (required)
    --concurrency <n>    Max parallel invocations (default 4)

  validate         Validate pipeline output (stdin: data)
    --type <type>        What to validate (required): docs, prompts, workflows,
                         tasks, complexity, expanded-tasks, scaffolds, tally,
                         debate-turn, decision-points, decision-tally,
                         deliberation-result
    --task-ids <json>    Expected task IDs as JSON array (for docs/prompts/workflows)
    --strict             Fail on warnings too

  sync-linear init           Create Linear project and PRD issue
  sync-linear issues         Create Linear issues for tasks and subtasks
  sync-linear rewrite-urls   Bulk-replace base URL in project issue descriptions
  sync-linear github-sync    Create GitHub issues mirroring Linear project issues
    --project-id <id>        Linear project ID (required)
    --repo <owner/repo>      GitHub repository (required)
    --branch <branch>        Branch name for linking (required)
    --github-project <num>   GitHub Project number (optional)

  parse-decision-points  Extract DECISION_POINT blocks from debate text
    (stdin: {content: string, speaker: "optimist"|"pessimist"})

  bridge-notify      POST agent notification to both bridges
    --from <agent>       Sender agent name (required)
    --to <agent>         Recipient agent name (required)
    --metadata <json>    Optional metadata JSON
    (stdin: message text)

  bridge-elicitation POST elicitation to both bridges with resume token
    --session-id <id>    Session ID (required)
    --decision-id <id>   Decision point ID (required)
    --vote-result <json> Vote tally result JSON (required)
    --resume-token <tok> Lobster resume token
    --human-review-mode <mode>  full_auto | supervised | manual

  tally-decision-votes  Tally committee votes on a decision point
    (stdin: DecisionVote[] JSON)

  generate-workflows  Generate per-task implementation workflows
    --config <path>        Path to cto-config.json (for retry settings)
    --scaffolds-json <file> Path to scaffolds JSON (optional)
    --repository-url <url>  Repository URL (optional)
    (stdin: expanded tasks JSON array)

  linear-activity    Create Linear agent activity
    --session-id <id>    Linear session ID (required)
    --type <type>        Activity type: thought|action|elicitation|response|error
    --body <text>        Activity body text
    --ephemeral          Mark as ephemeral
    --signal <signal>    Signal type: "select" (for elicitation with options)
    --signal-metadata <json>  Raw signal metadata JSON (e.g. '{"options":[...]}')
    --options <json>     Shorthand for signal-metadata options array

  linear-plan        Update Linear session plan
    --session-id <id>    Linear session ID (required)
    --plan-json <json>   Plan steps JSON array (or "-" for stdin)
    --step <step-id>     Auto-build plan from canonical pipeline steps
    --plan <json>        (legacy) Plan steps JSON array

  linear-session update-urls  Update session external URLs (clickable links)
    --session-id <id>    Linear session ID (required)
    --add-url <label=url>  Add a URL (repeatable, label=url format)
    --set-urls <json>    Full replacement: JSON array of {label, url} objects

  register-run       Register pipeline run with linear-bridge
    --run-id <id>        Run ID (required)
    --agent <name>       Agent name (required)
    --issue-id <id>      Linear issue ID
    --session-key <k>    Optional Lobster/session key for the bridge
    --linear-session-id <id>  Linear Agent API session id (optional)

  deregister-run     Deregister pipeline run
    --run-id <id>        Run ID (required)

  invoke-agent       Invoke an agent (subagent or A2A mode)
    --mode <mode>        subagent | a2a (required, acp accepted as deprecated alias)
    --agent <name>       Agent name (required)
    --prompt-file <path> Prompt file path
    --session-key <key>  Session key for subagent mode
    --task-context <ctx> Task context for ACP mode

  classify-output    Classify CLI output into Linear activity type
    --cli <type>         CLI type: claude | codex | openclaw (required)
    --intermediate       Mark as intermediate output
    (stdin: CLI output text)

  design-review      Post design variants to bridges and wait for selection
    --variants <path>    Path to design-variants.json (required)
    --session-id <id>    Session ID (required)
    --output <path>      Output path for selections JSON (default: .intake/design/stitch/design-selections.json)
    --linear-issue-id <id>  Linear issue ID for posting
    --discord-channel-id <id>  Discord channel ID
    --run-id <id>        Pipeline run ID
`);
  process.exit(1);
}

function getArg(args: string[], flag: string): string | undefined {
  const idx = args.indexOf(flag);
  if (idx !== -1 && idx + 1 < args.length) {
    return args[idx + 1];
  }
  return undefined;
}

/** Collect all values for a repeated flag (e.g. --add-url "X" --add-url "Y"). */
function getAllArgs(args: string[], flag: string): string[] {
  const results: string[] = [];
  for (let i = 0; i < args.length; i++) {
    if (args[i] === flag && i + 1 < args.length) {
      results.push(args[i + 1]);
      i++; // skip value
    }
  }
  return results;
}

async function readJsonInput(filePath: string | undefined): Promise<unknown> {
  let raw: string;

  if (filePath) {
    raw = fs.readFileSync(filePath, 'utf-8');
  } else {
    // Read from stdin (works in both Bun and Node)
    raw = await readStdin();
  }

  if (!raw.trim()) {
    console.error('Error: Empty input');
    process.exit(1);
  }

  try {
    return JSON.parse(raw);
  } catch {
    const commandLabel = process.argv.slice(2, 4).filter(Boolean).join(' ');
    const preview = raw.replace(/\s+/g, ' ').slice(0, 240);
    console.error(`Error: Invalid JSON input${commandLabel ? ` (${commandLabel})` : ''}`);
    if (filePath) {
      console.error(`Source file: ${filePath}`);
    } else {
      console.error(`Stdin preview: ${preview || '<empty>'}`);
    }
    process.exit(1);
  }
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const subcommand = args[0];

  if (!subcommand || subcommand === '--help' || subcommand === '-h') {
    usage();
  }

  switch (subcommand) {
    case 'write-files': {
      const basePath = getArg(args, '--base-path');
      const type = getArg(args, '--type') as 'docs' | 'prompts' | 'workflows' | undefined;

      if (!basePath) {
        console.error('Error: --base-path is required');
        process.exit(1);
      }
      if (!type || !['docs', 'prompts', 'workflows'].includes(type)) {
        console.error('Error: --type must be "docs", "prompts", or "workflows"');
        process.exit(1);
      }

      const input = await readJsonInput(undefined);
      const result = await writeFiles(input, basePath, type);
      console.log(JSON.stringify(result, null, 2));
      process.exit(0);
      break;
    }

    case 'tally': {
      const ballotsJson = getArg(args, '--ballots-json');
      const ballots = await readJsonInput(ballotsJson);

      if (!Array.isArray(ballots)) {
        console.error('Error: Expected an array of VoteBallot objects');
        process.exit(1);
      }

      const result = tallyVotes(ballots);
      console.log(JSON.stringify(result, null, 2));
      process.exit(0);
      break;
    }

    case 'fan-out': {
      const promptPath = getArg(args, '--prompt');
      const schemaPath = getArg(args, '--schema');
      const contextArg = getArg(args, '--context');
      const provider = getArg(args, '--provider');
      const model = getArg(args, '--model');
      const concurrencyArg = getArg(args, '--concurrency');

      if (!promptPath || !schemaPath || !provider || !model) {
        console.error('Error: --prompt, --schema, --provider, and --model are required');
        process.exit(1);
      }

      const items = await readJsonInput(undefined);
      if (!Array.isArray(items)) {
        console.error('Error: Expected a JSON array of tasks on stdin');
        process.exit(1);
      }

      let context: Record<string, unknown> = {};
      if (contextArg) {
        try {
          context = JSON.parse(contextArg);
        } catch {
          console.error('Error: --context must be valid JSON');
          process.exit(1);
        }
      }

      const concurrency = concurrencyArg ? parseInt(concurrencyArg, 10) : 4;

      const result = await fanOut({
        items,
        promptPath,
        schemaPath,
        context,
        provider,
        model,
        concurrency,
      });

      if (result.failures.length > 0) {
        console.error(`Warning: ${result.failures.length} item(s) failed:`);
        for (const f of result.failures) {
          console.error(`  - index ${f.index}${f.task_id != null ? ` (task ${f.task_id})` : ''}: ${f.error}`);
        }
      }

      console.log(JSON.stringify(result.results, null, 2));
      process.exit(result.failures.length > 0 ? 1 : 0);
      break;
    }

    case 'validate': {
      const valType = getArg(args, '--type');
      const taskIdsArg = getArg(args, '--task-ids');
      const strict = args.includes('--strict');

      if (!valType) {
        console.error('Error: --type is required');
        process.exit(1);
      }

      // Legacy fan-out validation types (require --task-ids)
      if (['docs', 'prompts', 'workflows'].includes(valType)) {
        if (!taskIdsArg) {
          console.error('Error: --task-ids is required for docs/prompts/workflows');
          process.exit(1);
        }

        let taskIds: number[];
        try {
          taskIds = JSON.parse(taskIdsArg);
        } catch {
          console.error('Error: --task-ids must be valid JSON array');
          process.exit(1);
        }

        const merged = await readJsonInput(undefined);
        if (!Array.isArray(merged)) {
          console.error('Error: Expected a JSON array on stdin');
          process.exit(1);
        }

        let valResult;
        if (valType === 'docs') valResult = validateDocs(merged, taskIds);
        else if (valType === 'prompts') valResult = validatePrompts(merged, taskIds);
        else valResult = validateWorkflows(merged, taskIds);

        console.log(JSON.stringify(valResult, null, 2));
        process.exit(valResult.valid ? 0 : 1);
      } else {
        // Generic schema-based validation
        const input = await readStdin();
        const result = validateGeneric(valType, input, strict);
        console.log(JSON.stringify(result, null, 2));
        process.exit(result.valid ? 0 : 1);
      }
      break;
    }

    case 'sync-linear': {
      const subMode = args[1];
      const apiKey = process.env.LINEAR_API_KEY;
      if (!apiKey) {
        console.error('Error: LINEAR_API_KEY environment variable is required');
        process.exit(1);
      }

      if (subMode === 'init') {
        const projectName = getArg(args, '--project-name');
        const teamId = getArg(args, '--team-id');
        const prdContentFile = getArg(args, '--prd-content');

        if (!projectName || !teamId) {
          console.error('Error: --project-name and --team-id are required');
          process.exit(1);
        }

        let prdContent: string;
        if (prdContentFile) {
          prdContent = fs.readFileSync(prdContentFile, 'utf-8');
        } else {
          prdContent = await readStdin();
        }

        if (!prdContent.trim()) {
          console.error('Error: Empty PRD content');
          process.exit(1);
        }

        const result = await createProjectAndPrdIssue({
          projectName,
          teamId,
          prdContent,
          apiKey,
        });

        console.log(JSON.stringify(result, null, 2));
        process.exit(0);
      } else if (subMode === 'issues') {
        const taskJson = getArg(args, '--task-json');
        const projectId = getArg(args, '--project-id');
        const prdIssueId = getArg(args, '--prd-issue-id');
        const teamId = getArg(args, '--team-id');
        const baseUrl = getArg(args, '--base-url') || '';
        const prUrl = getArg(args, '--pr-url') || '';
        const agentMapArg = getArg(args, '--agent-map');
        const milestoneMapArg = getArg(args, '--milestone-map');
        const pmUrl = getArg(args, '--pm-url') || process.env.PM_URL || '';

        if (!projectId || !prdIssueId || !teamId) {
          console.error('Error: --project-id, --prd-issue-id, and --team-id are required');
          process.exit(1);
        }

        const tasks = await readJsonInput(taskJson);
        if (!Array.isArray(tasks)) {
          console.error('Error: Expected an array of GeneratedTask objects');
          process.exit(1);
        }

        let agentMap: Record<string, string> = {};
        if (agentMapArg) {
          try {
            agentMap = JSON.parse(agentMapArg);
            console.error(`sync-linear issues: parsed agentMap with ${Object.keys(agentMap).length} entries`);
          } catch {
            console.error(`Warning: Could not parse --agent-map JSON (${agentMapArg.length} chars, preview: ${agentMapArg.slice(0, 80)}), using empty map`);
          }
        } else {
          console.error('sync-linear issues: no --agent-map provided, using empty map');
        }

        let milestoneMap: Record<string, string> | undefined;
        if (milestoneMapArg) {
          try {
            milestoneMap = JSON.parse(milestoneMapArg);
            console.error(`sync-linear issues: parsed milestoneMap with ${Object.keys(milestoneMap!).length} entries`);
          } catch {
            console.error(`Warning: Could not parse --milestone-map JSON, issues will not be assigned to milestones`);
          }
        }

        if (pmUrl) {
          console.error(`sync-linear issues: PM server at ${pmUrl} — will use per-agent tokens for self-assignment`);
        }

        const personalApiKey = process.env.LINEAR_PERSONAL_API_KEY || undefined;
        if (personalApiKey) {
          console.error(`sync-linear issues: personal API key available for issue assignment`);
        }

        const result = await syncTaskIssues({
          tasks,
          projectId,
          prdIssueId,
          teamId,
          baseUrl,
          prUrl,
          agentMap,
          apiKey,
          personalApiKey,
          pmUrl: pmUrl || undefined,
          milestoneMap,
        });

        console.log(JSON.stringify(result, null, 2));
        process.exit(0);
      } else if (subMode === 'rewrite-urls') {
        const projectId = getArg(args, '--project-id');
        const oldBaseUrl = getArg(args, '--old-base-url');
        const newBaseUrl = getArg(args, '--new-base-url');

        if (!projectId || !oldBaseUrl || !newBaseUrl) {
          console.error('Error: --project-id, --old-base-url, and --new-base-url are required');
          process.exit(1);
        }

        const result = await rewriteProjectUrls({
          projectId,
          oldBaseUrl,
          newBaseUrl,
          apiKey,
        });

        console.log(JSON.stringify(result, null, 2));
        process.exit(0);
      } else if (subMode === 'github-sync') {
        const projectId = getArg(args, '--project-id');
        const projectName = getArg(args, '--project-name');
        const repo = getArg(args, '--repo');
        const branch = getArg(args, '--branch');
        const githubProjectArg = getArg(args, '--github-project');

        if (!projectId || !repo || !branch) {
          console.error('Error: --project-id, --repo, and --branch are required');
          process.exit(1);
        }

        const githubProject = githubProjectArg ? parseInt(githubProjectArg, 10) : undefined;
        if (githubProjectArg && (!githubProject || isNaN(githubProject))) {
          console.error(`Error: --github-project must be a number, got "${githubProjectArg}"`);
          process.exit(1);
        }

        const result = await syncGitHubIssues({
          projectId,
          projectName,
          repo,
          branch,
          apiKey,
          githubProject,
        });

        console.log(JSON.stringify(result, null, 2));
        process.exit(0);
      } else {
        console.error(`Error: Unknown sync-linear sub-mode "${subMode}". Use "init", "issues", "rewrite-urls", or "github-sync".`);
        process.exit(1);
      }
      break;
    }

    case 'parse-decision-points': {
      const input = await readJsonInput(undefined) as { content?: string; speaker?: string };
      if (!input.content || !input.speaker) {
        console.error('Error: Expected { content: string, speaker: "optimist"|"pessimist" }');
        process.exit(1);
      }
      if (input.speaker !== 'optimist' && input.speaker !== 'pessimist') {
        console.error('Error: speaker must be "optimist" or "pessimist"');
        process.exit(1);
      }

      const result = parseDecisionPoints(input.content, input.speaker);
      console.log(JSON.stringify(result, null, 2));
      process.exit(0);
      break;
    }

    case 'bridge-notify': {
      const from = getArg(args, '--from');
      const to = getArg(args, '--to');
      const metadataArg = getArg(args, '--metadata');

      if (!from || !to) {
        console.error('Error: --from and --to are required');
        process.exit(1);
      }

      const message = await readStdin();

      let metadata: Record<string, string> | undefined;
      if (metadataArg) {
        try {
          metadata = JSON.parse(metadataArg);
        } catch {
          console.error('Warning: Could not parse --metadata JSON');
        }
      }

      const result = await bridgeNotify({ from, to, message: message.trim(), metadata });
      console.log(JSON.stringify(result));
      process.exit(0);
      break;
    }

    case 'bridge-elicitation': {
      const sessionId = getArg(args, '--session-id');
      const decisionId = getArg(args, '--decision-id');
      const voteResultArg = getArg(args, '--vote-result');
      const humanReviewMode = getArg(args, '--human-review-mode');
      const resumeToken = getArg(args, '--resume-token');
      const linearSessionId = getArg(args, '--linear-session-id');
      const linearIssueId = getArg(args, '--linear-issue-id');
      const discordChannelId = getArg(args, '--discord-channel-id');
      const runId = getArg(args, '--run-id');

      if (!sessionId || !decisionId || !voteResultArg) {
        console.error('Error: --session-id, --decision-id, and --vote-result are required');
        process.exit(1);
      }

      let voteResult: Record<string, unknown>;
      try {
        voteResult = JSON.parse(voteResultArg);
      } catch {
        console.error('Error: --vote-result must be valid JSON');
        process.exit(1);
      }

      const result = await bridgeElicitation({
        sessionId,
        decisionId,
        voteResult,
        linearSessionId,
        resumeToken,
        humanReviewMode,
        linearIssueId,
        discordChannelId,
        runId,
      });
      console.log(JSON.stringify(result));
      process.exit(0);
      break;
    }

    case 'tally-decision-votes': {
      const input = await readJsonInput(undefined);
      if (!Array.isArray(input)) {
        console.error('Error: Expected an array of DecisionVote objects');
        process.exit(1);
      }

      const result = tallyDecisionVotes(input);
      console.log(JSON.stringify(result, null, 2));
      process.exit(0);
      break;
    }

    case 'generate-workflows': {
      const configPath = getArg(args, '--config');
      const scaffoldsFile = getArg(args, '--scaffolds-json');
      const repositoryUrl = getArg(args, '--repository-url') ?? '';

      const tasks = await readJsonInput(undefined);
      if (!Array.isArray(tasks)) {
        console.error('Error: Expected a JSON array of expanded tasks on stdin');
        process.exit(1);
      }

      let playConfig: { implementationMaxRetries?: number; qualityMaxRetries?: number; agentCommunication?: string } = {};
      if (configPath) {
        try {
          const raw = fs.readFileSync(configPath, 'utf-8');
          const config = JSON.parse(raw);
          playConfig = config?.defaults?.play ?? {};
        } catch {
          console.error('Warning: Could not read config, using defaults');
        }
      }

      let scaffolds: unknown[] = [];
      if (scaffoldsFile) {
        try {
          const raw = fs.readFileSync(scaffoldsFile, 'utf-8');
          const parsed = JSON.parse(raw);
          scaffolds = parsed.scaffolds ?? parsed;
        } catch {
          console.error('Warning: Could not read scaffolds, proceeding without');
        }
      }

      const result = generateWorkflows({
        expanded_tasks: tasks,
        scaffolds: scaffolds as import('./types').TaskScaffold[],
        config: playConfig,
        repository_url: repositoryUrl,
      });

      console.log(JSON.stringify(result, null, 2));
      process.exit(0);
      break;
    }

    case 'linear-activity': {
      const sessionId = getArg(args, '--session-id');
      const type = getArg(args, '--type') as 'thought' | 'plan' | 'action' | 'elicitation' | 'response' | 'error' | undefined;
      const body = getArg(args, '--body') ?? (await readStdin()).trim();
      const ephemeral = args.includes('--ephemeral');
      const action = getArg(args, '--action');
      const parameter = getArg(args, '--parameter');
      const resultStr = getArg(args, '--result');
      const signal = getArg(args, '--signal') as 'select' | undefined;
      const optionsArg = getArg(args, '--options');
      const signalMetadataArg = getArg(args, '--signal-metadata');

      if (!sessionId) {
        console.error('Warning: linear-activity skipped (missing --session-id)');
        console.log(JSON.stringify({ ok: true, skipped: true, reason: 'missing_session_id' }));
        process.exit(0);
      }

      if (!type) {
        console.error('Error: --type is required');
        process.exit(1);
      }

      let options: Array<{ label: string; value: string }> | undefined;
      if (optionsArg) {
        try {
          options = JSON.parse(optionsArg);
        } catch {
          console.error('Error: --options must be valid JSON');
          process.exit(1);
        }
      }

      let signalMetadata: Record<string, unknown> | undefined;
      if (signalMetadataArg) {
        try {
          signalMetadata = JSON.parse(signalMetadataArg);
        } catch {
          console.error('Error: --signal-metadata must be valid JSON');
          process.exit(1);
        }
      }

      const result = await linearActivity({
        sessionId,
        type,
        body,
        ephemeral,
        action,
        parameter,
        result: resultStr,
        signal,
        options,
        signalMetadata,
      });
      console.log(JSON.stringify(result));
      process.exit(0);
      break;
    }

    case 'linear-plan': {
      const sessionId = getArg(args, '--session-id');
      const planJsonArg = getArg(args, '--plan-json');
      const stepArg = getArg(args, '--step');
      const planArg = getArg(args, '--plan'); // legacy compat

      if (!sessionId) {
        console.error('Warning: linear-plan skipped (missing --session-id)');
        console.log(JSON.stringify({ ok: true, skipped: true, reason: 'missing_session_id' }));
        process.exit(0);
      }

      try {
        // Resolve plan from --plan-json, --step, or legacy --plan
        let stdinContent: string | undefined;
        if (planJsonArg === '-') {
          stdinContent = await readStdin();
        }
        const plan = resolvePlanSteps(planJsonArg ?? planArg, stepArg, stdinContent);
        if (!plan || plan.length === 0) {
          console.error('Error: one of --plan-json, --step, or --plan is required');
          process.exit(1);
        }

        await linearPlan({ sessionId, plan: plan as any });
        console.log(JSON.stringify({ updated: true }));
      } catch (err: any) {
        // Non-fatal — pipeline should continue if plan update fails
        console.error(`Warning: linear-plan failed (non-fatal): ${err.message || err}`);
        console.log(JSON.stringify({ updated: false, error: String(err.message || err) }));
      }
      process.exit(0);
      break;
    }

    case 'linear-session': {
      const subMode = args[1];
      if (subMode !== 'update-urls') {
        console.error(`Error: Unknown linear-session sub-mode "${subMode}". Use "update-urls".`);
        process.exit(1);
      }

      const sessionId = getArg(args, '--session-id');
      if (!sessionId) {
        console.warn('Warning: linear-session skipped (missing --session-id)');
        console.log(JSON.stringify({ ok: true, skipped: true, reason: 'missing_session_id' }));
        process.exit(0);
      }

      const setUrlsArg = getArg(args, '--set-urls');
      const addUrlArgs = getAllArgs(args, '--add-url');

      if (!setUrlsArg && addUrlArgs.length === 0) {
        console.error('Error: --add-url or --set-urls is required');
        process.exit(1);
      }

      let setUrls: Array<{ label: string; url: string }> | undefined;
      if (setUrlsArg) {
        try {
          setUrls = JSON.parse(setUrlsArg);
        } catch {
          console.error('Error: --set-urls must be valid JSON array of {label, url}');
          process.exit(1);
        }
      }

      // Parse --add-url "label=url" entries
      const addUrls: Array<{ label: string; url: string }> = [];
      for (const entry of addUrlArgs) {
        const eqIdx = entry.indexOf('=');
        if (eqIdx === -1) {
          console.error(`Error: --add-url value must be "label=url" format, got: ${entry}`);
          process.exit(1);
        }
        addUrls.push({ label: entry.slice(0, eqIdx), url: entry.slice(eqIdx + 1) });
      }

      const result = await linearSessionUpdateUrls({
        sessionId,
        externalUrls: setUrls,
        addUrls: addUrls.length > 0 ? addUrls : undefined,
      });
      console.log(JSON.stringify(result));
      process.exit(0);
      break;
    }

    case 'register-run': {
      const runId = getArg(args, '--run-id');
      const agent = getArg(args, '--agent');
      const issueId = getArg(args, '--issue-id') || undefined;
      const sessionKey = getArg(args, '--session-key') || undefined;
      const linearSessionId = getArg(args, '--linear-session-id') || undefined;

      if (!runId || !agent) {
        console.error('Error: --run-id and --agent are required');
        process.exit(1);
      }

      const ok = await registerRun({ runId, agent, issueId, sessionKey, linearSessionId });
      console.log(JSON.stringify({ registered: ok, run_id: runId }));
      process.exit(ok ? 0 : 1);
      break;
    }

    case 'deregister-run': {
      const runId = getArg(args, '--run-id');
      if (!runId) {
        console.error('Error: --run-id is required');
        process.exit(1);
      }

      const ok = await deregisterRun(runId);
      console.log(JSON.stringify({ deregistered: ok }));
      process.exit(ok ? 0 : 1);
      break;
    }

    case 'invoke-agent': {
      const mode = getArg(args, '--mode') as 'subagent' | 'a2a' | 'acp' | undefined;
      const agent = getArg(args, '--agent');
      const promptFile = getArg(args, '--prompt-file');
      const prompt = getArg(args, '--prompt');
      const sessionKey = getArg(args, '--session-key');
      const taskContext = getArg(args, '--task-context');

      if (!mode || !agent) {
        console.error('Error: --mode and --agent are required');
        process.exit(1);
      }

      if (!['subagent', 'a2a', 'acp'].includes(mode)) {
        console.error('Error: --mode must be "subagent" or "a2a" (or deprecated alias "acp")');
        process.exit(1);
      }

      const result = await invokeAgent({ mode, agent, promptFile, prompt, sessionKey, taskContext });
      console.log(JSON.stringify(result));
      process.exit(result.success ? 0 : 1);
      break;
    }

    case 'classify-output': {
      const cli = getArg(args, '--cli') as 'claude' | 'codex' | 'openclaw' | undefined;
      const isIntermediate = args.includes('--intermediate');

      if (!cli) {
        console.error('Error: --cli is required');
        process.exit(1);
      }

      const output = await readStdin();
      const result = classifyCliOutput(output, cli, isIntermediate);
      console.log(JSON.stringify(result, null, 2));
      process.exit(0);
      break;
    }

    case 'design-review': {
      const variantsPath = getArg(args, '--variants');
      const sessionId = getArg(args, '--session-id');
      const outputPath = getArg(args, '--output') ?? '.intake/design/stitch/design-selections.json';
      const linearIssueId = getArg(args, '--linear-issue-id') || undefined;
      const discordChannelId = getArg(args, '--discord-channel-id') || undefined;
      const runId = getArg(args, '--run-id') || undefined;

      if (!variantsPath || !sessionId) {
        console.error('Error: --variants and --session-id are required');
        process.exit(1);
      }

      const selections = await designReview({
        variantsPath,
        sessionId,
        outputPath,
        linearIssueId,
        discordChannelId,
        runId,
      });
      console.log(JSON.stringify(selections, null, 2));
      process.exit(0);
      break;
    }

    default:
      console.error(`Error: Unknown subcommand "${subcommand}"`);
      usage();
  }
}

main().catch((err) => {
  console.error('Fatal error:', err.message || err);
  process.exit(1);
});
