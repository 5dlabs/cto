#!/usr/bin/env bun
/**
 * Expand all AlertHub tasks with subtasks.
 * This generates the full real-world structure.
 */

import { spawn } from 'child_process';
import { readFileSync, writeFileSync, mkdirSync, existsSync, rmSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const INTAKE_AGENT = join(__dirname, '../dist/intake-agent');
const TASKS_FILE = join(__dirname, '../../../tests/intake/alerthub-e2e-test/.tasks/tasks/tasks.json');
const OUTPUT_DIR = join(__dirname, '../tests/alerthub-full/.tasks');

interface Subtask {
  id: number;
  title: string;
  description: string;
  status: string;
  dependencies: number[];
  details?: string;
  testStrategy?: string;
  subagentType?: string;
  parallelizable?: boolean;
}

interface Task {
  id: number;
  title: string;
  description: string;
  status: string;
  dependencies: number[];
  priority: string;
  details?: string;
  testStrategy?: string;
  decisionPoints?: any[];
  subtasks?: Subtask[];
}

async function callIntakeAgent(request: object): Promise<any> {
  return new Promise((resolve, reject) => {
    const proc = spawn(INTAKE_AGENT, [], { stdio: ['pipe', 'pipe', 'pipe'] });
    
    let stdout = '';
    proc.stdout.on('data', (data) => { stdout += data; });
    proc.on('close', () => {
      const lines = stdout.split('\n');
      const jsonLine = lines.find(l => l.startsWith('{'));
      if (jsonLine) {
        try { resolve(JSON.parse(jsonLine)); }
        catch (e) { reject(new Error(`Parse error`)); }
      } else {
        reject(new Error(`No JSON in output`));
      }
    });
    
    proc.stdin.write(JSON.stringify(request));
    proc.stdin.end();
  });
}

async function expandTask(task: Task): Promise<Subtask[]> {
  const result = await callIntakeAgent({
    operation: 'expand_task',
    payload: {
      task: {
        id: String(task.id),
        title: task.title,
        description: task.description,
        details: task.details || '',
        test_strategy: task.testStrategy || '',
        status: task.status,
        dependencies: task.dependencies.map(String)
      },
      subtask_count: 4,
      next_subtask_id: 1,
      enable_subagents: true
    }
  });
  
  if (result.success) {
    return result.data.subtasks;
  }
  return [];
}

function generateSubtaskPrompt(subtask: Subtask, parentTaskId: number): string {
  return `# Subtask ${parentTaskId}.${subtask.id}: ${subtask.title}

## Parent Task
Task ${parentTaskId}

## Subagent Type
${subtask.subagentType || 'implementer'}

## Parallelizable
${subtask.parallelizable ? 'Yes - can run concurrently' : 'No - must wait for dependencies'}

## Description
${subtask.description}

## Dependencies
${subtask.dependencies.length > 0 ? subtask.dependencies.map(d => `- Subtask ${parentTaskId}.${d}`).join('\n') : 'None'}

## Implementation Details
${subtask.details || 'See parent task.'}

## Test Strategy
${subtask.testStrategy || 'See parent task acceptance criteria.'}
`;
}

function generateTaskPrompt(task: Task): string {
  const cleanTitle = task.title.replace(/\s*\([^)]+\)\s*$/, '').trim();
  return `# Task ${task.id}: ${cleanTitle}

## Priority
${task.priority || 'medium'}

## Description
${task.description}

## Dependencies
${task.dependencies.length > 0 ? task.dependencies.map(d => `- Task ${d}`).join('\n') : 'None'}

## Implementation Details
${task.details || 'See subtasks.'}

## Acceptance Criteria
${task.testStrategy || 'See subtasks.'}

## Decision Points
${task.decisionPoints?.map(d => `- **${d.id}** [${d.category}]: ${d.description}`).join('\n') || 'None'}

## Subtasks
${task.subtasks?.map(s => `- ${s.id}. ${s.title} [${s.subagentType}]`).join('\n') || 'Pending expansion'}
`;
}

async function main() {
  // Load tasks
  const tasks: Task[] = JSON.parse(readFileSync(TASKS_FILE, 'utf-8'));
  console.log(`📦 Loaded ${tasks.length} tasks from AlertHub PRD\n`);
  
  // Clean output
  if (existsSync(OUTPUT_DIR)) {
    rmSync(OUTPUT_DIR, { recursive: true });
  }
  mkdirSync(join(OUTPUT_DIR, 'tasks'), { recursive: true });
  mkdirSync(join(OUTPUT_DIR, 'docs'), { recursive: true });
  
  let totalSubtasks = 0;
  let failedTasks = 0;
  
  // Expand each task
  for (const task of tasks) {
    process.stdout.write(`🔧 Task ${task.id}/${tasks.length}: ${task.title.slice(0, 40)}... `);
    
    try {
      const subtasks = await expandTask(task);
      task.subtasks = subtasks;
      
      if (subtasks.length > 0) {
        totalSubtasks += subtasks.length;
        console.log(`✅ ${subtasks.length} subtasks`);
        
        // Create task folder
        const taskDir = join(OUTPUT_DIR, 'docs', `task-${task.id}`);
        mkdirSync(taskDir, { recursive: true });
        
        // Write task prompt
        writeFileSync(join(taskDir, 'prompt.md'), generateTaskPrompt(task));
        
        // Create subtask folders
        const subtasksDir = join(taskDir, 'subtasks');
        mkdirSync(subtasksDir, { recursive: true });
        
        for (const subtask of subtasks) {
          const subtaskDir = join(subtasksDir, `task-${task.id}.${subtask.id}`);
          mkdirSync(subtaskDir, { recursive: true });
          writeFileSync(join(subtaskDir, 'prompt.md'), generateSubtaskPrompt(subtask, task.id));
        }
      } else {
        console.log(`⚠️ No subtasks`);
        failedTasks++;
      }
    } catch (e) {
      console.log(`❌ Failed`);
      failedTasks++;
    }
  }
  
  // Save expanded tasks.json
  writeFileSync(join(OUTPUT_DIR, 'tasks', 'tasks.json'), JSON.stringify(tasks, null, 2));
  
  console.log(`\n📊 Summary:`);
  console.log(`   Tasks: ${tasks.length}`);
  console.log(`   Subtasks: ${totalSubtasks}`);
  console.log(`   Failed: ${failedTasks}`);
  console.log(`   Output: ${OUTPUT_DIR}`);
}

main().catch(console.error);
