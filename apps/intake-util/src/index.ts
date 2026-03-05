#!/usr/bin/env bun
/**
 * intake-util CLI - deterministic operations for the intake pipeline.
 *
 * Subcommands:
 *   write-files     --base-path <dir> --type <docs|prompts>
 *   tally           --ballots-json <file>
 *   sync-linear init    --project-name <name> --team-id <id> --prd-content <file>
 *   sync-linear issues  --project-id <id> --prd-issue-id <id> --team-id <id> --base-url <url>
 *
 * All subcommands also accept JSON on stdin when --task-json / --ballots-json is omitted.
 */

import { writeFiles } from './write-files';
import { tallyVotes } from './tally';
import { createProjectAndPrdIssue, syncTaskIssues } from './sync-linear';
import { fanOut } from './fan-out';
import { validateDocs, validatePrompts } from './validate';
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
    --type <type>        File type: "docs" or "prompts" (required)

  tally            Tally votes from model ballots
    --ballots-json <file> Path to VoteBallot[] JSON (or pipe via stdin)

  fan-out          Run parallel LLM invocations per task (stdin: tasks JSON array)
    --prompt <path>      System prompt file path (required)
    --schema <path>      Per-item output schema path (required)
    --context <json>     Shared context JSON (scaffolds, codebase, infra)
    --provider <p>       LLM provider (required)
    --model <m>          LLM model (required)
    --concurrency <n>    Max parallel invocations (default 4)

  validate         Validate merged fan-out output (stdin: merged JSON array)
    --type <type>        What to validate: "docs" or "prompts" (required)
    --task-ids <json>    Expected task IDs as JSON array (required)

  sync-linear init    Create Linear project and PRD issue
    --project-name <n>   Project name (required)
    --team-id <id>       Linear team ID (required)
    --prd-content <file> Path to PRD content file (or pipe via stdin)

  sync-linear issues  Create Linear issues for tasks and subtasks
    --task-json <file>   Path to GeneratedTask[] JSON (or pipe via stdin)
    --project-id <id>    Linear project ID (required)
    --prd-issue-id <id>  Linear PRD issue ID (required)
    --team-id <id>       Linear team ID (required)
    --base-url <url>     Repository URL for prompt links
    --agent-map <json>   JSON map of agent name → Linear user ID
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
    console.error('Error: Invalid JSON input');
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
      const type = getArg(args, '--type') as 'docs' | 'prompts' | undefined;

      if (!basePath) {
        console.error('Error: --base-path is required');
        process.exit(1);
      }
      if (!type || (type !== 'docs' && type !== 'prompts')) {
        console.error('Error: --type must be "docs" or "prompts"');
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
      const valType = getArg(args, '--type') as 'docs' | 'prompts' | undefined;
      const taskIdsArg = getArg(args, '--task-ids');

      if (!valType || (valType !== 'docs' && valType !== 'prompts')) {
        console.error('Error: --type must be "docs" or "prompts"');
        process.exit(1);
      }
      if (!taskIdsArg) {
        console.error('Error: --task-ids is required');
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

      const valResult = valType === 'docs'
        ? validateDocs(merged, taskIds)
        : validatePrompts(merged, taskIds);

      console.log(JSON.stringify(valResult, null, 2));
      process.exit(valResult.valid ? 0 : 1);
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
        const agentMapArg = getArg(args, '--agent-map');

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
          } catch {
            console.error('Warning: Could not parse --agent-map JSON, using empty map');
          }
        }

        const result = await syncTaskIssues({
          tasks,
          projectId,
          prdIssueId,
          teamId,
          baseUrl,
          agentMap,
          apiKey,
        });

        console.log(JSON.stringify(result, null, 2));
        process.exit(0);
      } else {
        console.error(`Error: Unknown sync-linear sub-mode "${subMode}". Use "init" or "issues".`);
        process.exit(1);
      }
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
