#!/usr/bin/env bun
/**
 * Generate complete task structure with subtask folders.
 * 
 * 1. Parse PRD → Tasks with decisionPoints
 * 2. Expand each task → Subtasks with subagentType
 * 3. Generate prompt folders including subtasks/
 */

import { spawn } from 'child_process';
import { writeFileSync, mkdirSync, existsSync, rmSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const INTAKE_AGENT = join(__dirname, '../dist/intake-agent');

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
        catch (e) { reject(new Error(`Parse error: ${jsonLine.slice(0, 200)}`)); }
      } else {
        reject(new Error(`No JSON in output`));
      }
    });
    
    proc.stdin.write(JSON.stringify(request));
    proc.stdin.end();
  });
}

async function main() {
  const outputDir = join(__dirname, '../tests/e2e-full-structure/.tasks');
  
  // Clean and create output dir
  if (existsSync(outputDir)) {
    rmSync(outputDir, { recursive: true });
  }
  mkdirSync(join(outputDir, 'tasks'), { recursive: true });
  mkdirSync(join(outputDir, 'docs'), { recursive: true });
  
  const prd = `# Alert Management Service PRD

## Overview
Build an alert management system with multi-channel notifications, escalation policies, and on-call scheduling.

## Features
1. Alert ingestion via webhooks and API
2. Notification routing based on severity and tags
3. Escalation policies with timeout-based escalation
4. On-call schedule management
5. Alert acknowledgment and resolution tracking

## Technical Stack
- PostgreSQL for persistence
- Redis for real-time state
- Go backend service
- React admin dashboard`;

  console.log('🚀 Generating full task structure with subtask folders\n');
  
  // Step 1: Parse PRD
  console.log('📄 Step 1: Parsing PRD...');
  const parseResult = await callIntakeAgent({
    operation: 'parse_prd',
    payload: { prd_content: prd, prd_path: 'prd.md', num_tasks: 4, next_id: 1 }
  });
  
  if (!parseResult.success) throw new Error(`parse_prd failed: ${parseResult.error}`);
  const tasks: Task[] = parseResult.data.tasks;
  console.log(`   ✅ ${tasks.length} tasks generated\n`);
  
  // Step 2: Expand each task
  console.log('📦 Step 2: Expanding tasks into subtasks...');
  for (const task of tasks) {
    console.log(`   🔧 Expanding task ${task.id}...`);
    const expandResult = await callIntakeAgent({
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
    
    if (expandResult.success) {
      task.subtasks = expandResult.data.subtasks;
      console.log(`      ✅ ${task.subtasks.length} subtasks`);
    } else {
      console.log(`      ⚠️ Failed: ${expandResult.error?.slice(0, 50)}`);
      task.subtasks = [];
    }
  }
  
  // Step 3: Generate prompts with subtask folders
  console.log('\n📁 Step 3: Generating prompt folders...');
  const promptResult = await callIntakeAgent({
    operation: 'generate_prompts',
    payload: {
      tasks,
      output_dir: join(outputDir, 'docs'),
      project_name: 'alert-management',
      include_subtasks: true
    }
  });
  
  if (!promptResult.success) throw new Error(`generate_prompts failed: ${promptResult.error}`);
  console.log(`   ✅ ${promptResult.data.file_count} files generated`);
  console.log(`   ✅ ${promptResult.data.subtask_count} subtask folders`);
  
  // Save tasks.json
  const tasksPath = join(outputDir, 'tasks/tasks.json');
  writeFileSync(tasksPath, JSON.stringify(tasks, null, 2));
  console.log(`\n💾 Saved tasks.json`);
  
  // Print structure
  console.log('\n📊 Generated Structure:');
  for (const task of tasks) {
    console.log(`\n📁 task-${task.id}/`);
    console.log(`   ├── prompt.md`);
    console.log(`   ├── prompt.xml`);
    console.log(`   ├── acceptance.md`);
    if (task.subtasks && task.subtasks.length > 0) {
      console.log(`   └── subtasks/`);
      for (const st of task.subtasks) {
        console.log(`       └── task-${task.id}.${st.id}/ [${st.subagentType}]`);
      }
    }
  }
  
  console.log('\n✅ Done!');
}

main().catch(e => { console.error('❌ Error:', e.message); process.exit(1); });
