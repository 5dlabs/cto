#!/usr/bin/env bun
/**
 * intake-util CLI - deterministic operations for the intake pipeline.
 *
 * Subcommands:
 *   generate-docs    --task-json <file> --base-path <dir>
 *   generate-prompts --task-json <file> --output-dir <dir> --project-name <name>
 *   tally            --ballots-json <file>
 *
 * All subcommands also accept JSON on stdin when --task-json / --ballots-json is omitted.
 */

import { generateDocs } from './generate-docs';
import { generatePrompts } from './generate-prompts';
import { tallyVotes } from './tally';
import * as fs from 'fs';

function usage(): never {
  console.error(`Usage: intake-util <subcommand> [options]

Subcommands:
  generate-docs    Generate task documentation files
    --task-json <file>   Path to GeneratedTask[] JSON (or pipe via stdin)
    --base-path <dir>    Output directory (required)

  generate-prompts Generate prompt/acceptance files
    --task-json <file>   Path to GeneratedTask[] JSON (or pipe via stdin)
    --output-dir <dir>   Output directory (required)
    --project-name <n>   Project name (default: "project")

  tally            Tally votes from model ballots
    --ballots-json <file> Path to VoteBallot[] JSON (or pipe via stdin)
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
    // Read from stdin
    raw = await new Response(Bun.stdin.stream()).text();
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
    case 'generate-docs': {
      const taskJson = getArg(args, '--task-json');
      const basePath = getArg(args, '--base-path');

      if (!basePath) {
        console.error('Error: --base-path is required');
        process.exit(1);
      }

      const tasks = await readJsonInput(taskJson);
      const result = await generateDocs({
        tasks: tasks as any[],
        base_path: basePath,
        project_root: basePath,
      });

      console.log(JSON.stringify(result, null, 2));
      process.exit(result.success ? 0 : 1);
      break;
    }

    case 'generate-prompts': {
      const taskJson = getArg(args, '--task-json');
      const outputDir = getArg(args, '--output-dir');
      const projectName = getArg(args, '--project-name') || 'project';

      if (!outputDir) {
        console.error('Error: --output-dir is required');
        process.exit(1);
      }

      const tasks = await readJsonInput(taskJson);
      const result = await generatePrompts({
        tasks: tasks as any[],
        output_dir: outputDir,
        project_name: projectName,
      });

      console.log(JSON.stringify(result, null, 2));
      process.exit(result.success ? 0 : 1);
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

    default:
      console.error(`Error: Unknown subcommand "${subcommand}"`);
      usage();
  }
}

main().catch((err) => {
  console.error('Fatal error:', err.message || err);
  process.exit(1);
});
