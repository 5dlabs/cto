#!/usr/bin/env bun
/**
 * Generate nested task structure: Task → Subtasks
 * 
 * This script demonstrates the full workflow:
 * 1. Parse PRD → Tasks with decisionPoints
 * 2. Expand each task → Subtasks with subagentType
 * 3. Combine into nested structure
 */

import { spawn } from 'child_process';
import { writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const INTAKE_AGENT = join(__dirname, '../dist/intake-agent');
const OUTPUT_DIR = join(__dirname, '../tests/e2e-nested-tasks');

interface DecisionPoint {
  id: string;
  category: string;
  description: string;
  options: string[];
  requiresApproval: boolean;
  constraintType: string;
}

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
  decisionPoints?: DecisionPoint[];
  subtasks?: Subtask[];
}

async function callIntakeAgent(request: object): Promise<any> {
  return new Promise((resolve, reject) => {
    const proc = spawn(INTAKE_AGENT, [], {
      stdio: ['pipe', 'pipe', 'pipe']
    });
    
    let stdout = '';
    let stderr = '';
    
    proc.stdout.on('data', (data) => { stdout += data; });
    proc.stderr.on('data', (data) => { stderr += data; });
    
    proc.on('close', (code) => {
      // Find JSON in output (skip log lines)
      const lines = stdout.split('\n');
      const jsonLine = lines.find(l => l.startsWith('{'));
      
      if (jsonLine) {
        try {
          resolve(JSON.parse(jsonLine));
        } catch (e) {
          reject(new Error(`Failed to parse JSON: ${jsonLine}`));
        }
      } else {
        reject(new Error(`No JSON output. stderr: ${stderr}`));
      }
    });
    
    proc.stdin.write(JSON.stringify(request));
    proc.stdin.end();
  });
}

async function parsePrd(prdContent: string, numTasks: number): Promise<Task[]> {
  console.log(`📄 Parsing PRD into ${numTasks} tasks...`);
  
  const result = await callIntakeAgent({
    operation: 'parse_prd',
    payload: {
      prd_content: prdContent,
      prd_path: 'test.md',
      num_tasks: numTasks,
      next_id: 1
    }
  });
  
  if (!result.success) {
    throw new Error(`parse_prd failed: ${result.error}`);
  }
  
  console.log(`   ✅ Generated ${result.data.tasks.length} tasks`);
  return result.data.tasks;
}

async function expandTask(task: Task, subtaskCount: number): Promise<Subtask[]> {
  console.log(`   🔧 Expanding task ${task.id}: ${task.title.slice(0, 50)}...`);
  
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
      subtask_count: subtaskCount,
      next_subtask_id: 1,
      enable_subagents: true
    }
  });
  
  if (!result.success) {
    console.log(`   ⚠️ Failed to expand: ${result.error}`);
    return [];
  }
  
  console.log(`      ✅ Generated ${result.data.subtasks.length} subtasks`);
  return result.data.subtasks;
}

async function main() {
  const prd = `# Notification Service PRD

## Overview
Build a real-time notification service supporting push, email, and SMS delivery.

## Features
1. User notification preferences (channel preferences, quiet hours)
2. Message queue for reliable delivery with retry logic
3. Rate limiting per user and global
4. Delivery status tracking and webhooks
5. Template engine with variable substitution

## Technical Requirements
- PostgreSQL for persistence
- Redis for queuing and rate limiting
- Horizontally scalable workers
- REST API for clients
- Admin dashboard for monitoring`;

  console.log('🚀 Generating nested task structure\n');
  
  // Step 1: Parse PRD
  const tasks = await parsePrd(prd, 4);
  
  // Step 2: Expand each task into subtasks
  console.log('\n📦 Expanding tasks into subtasks...');
  
  for (const task of tasks) {
    const subtasks = await expandTask(task, 4);
    task.subtasks = subtasks;
  }
  
  // Step 3: Save output
  const output = {
    meta: {
      generatedAt: new Date().toISOString(),
      prd: 'notification-service',
      taskCount: tasks.length,
      subtaskCount: tasks.reduce((sum, t) => sum + (t.subtasks?.length || 0), 0)
    },
    tasks
  };
  
  const outputPath = join(OUTPUT_DIR, 'tasks-with-subtasks.json');
  writeFileSync(outputPath, JSON.stringify(output, null, 2));
  
  console.log(`\n✅ Saved to ${outputPath}`);
  console.log(`   Tasks: ${output.meta.taskCount}`);
  console.log(`   Subtasks: ${output.meta.subtaskCount}`);
  
  // Print structure
  console.log('\n📊 Structure:');
  for (const task of tasks) {
    console.log(`\nTask ${task.id}: ${task.title}`);
    if (task.decisionPoints?.length) {
      console.log(`   Decision Points: ${task.decisionPoints.length}`);
    }
    if (task.subtasks?.length) {
      for (const st of task.subtasks) {
        const parallel = st.parallelizable ? '∥' : '→';
        console.log(`   ${parallel} Subtask ${st.id}: ${st.title.slice(0, 50)}... [${st.subagentType}]`);
      }
    }
  }
}

main().catch(console.error);
