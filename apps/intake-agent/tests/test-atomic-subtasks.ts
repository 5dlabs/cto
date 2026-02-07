#!/usr/bin/env bun
/**
 * Test script to verify subtasks are atomic (one component per subtask)
 * 
 * This tests the fix for the anti-pattern where multiple components
 * were grouped into single subtasks.
 */

import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

// Anti-patterns to detect (multiple components in one subtask)
const ANTI_PATTERNS = [
  /postgresql.*mongo/i,
  /mongo.*postgresql/i,
  /redis.*mongo/i,
  /mongo.*redis/i,
  /postgresql.*redis/i,
  /redis.*postgresql/i,
  /kafka.*rabbit/i,
  /rabbit.*kafka/i,
  /seaweed.*kafka/i,
  /kafka.*seaweed/i,
  /seaweed.*rabbit/i,
  /rabbit.*seaweed/i,
  // Generic patterns for "X and Y" or "X, Y, and Z"
  /deploy.*(?:postgresql|postgres).*(?:and|,).*(?:mongo|redis|kafka|rabbit)/i,
  /deploy.*(?:mongo).*(?:and|,).*(?:postgres|redis|kafka|rabbit)/i,
  /deploy.*(?:redis|valkey).*(?:and|,).*(?:postgres|mongo|kafka|rabbit)/i,
  /setup.*(?:kafka).*(?:and|,).*(?:rabbit|postgres|mongo|redis)/i,
  /(?:database|databases).*(?:postgresql|postgres).*(?:mongo|redis)/i,
  /(?:messaging|message).*(?:kafka).*(?:rabbit)/i,
];

interface Subtask {
  id: number;
  title: string;
  description: string;
  details?: string;
}

interface Task {
  id: number;
  title: string;
  description: string;
  details?: string;
  subtasks?: Subtask[];
}

interface ExpandTaskRequest {
  operation: 'expand_task';
  model: string;
  payload: {
    task: {
      id: string;
      title: string;
      description: string;
      details: string;
      status: string;
      dependencies: string[];
    };
    subtask_count: number;
    next_subtask_id: number;
    enable_subagents: boolean;
  };
}

interface ExpandTaskResponse {
  success: boolean;
  data?: {
    subtasks: Subtask[];
  };
  error?: string;
}

function detectAntiPattern(text: string): string | null {
  for (const pattern of ANTI_PATTERNS) {
    if (pattern.test(text)) {
      return pattern.toString();
    }
  }
  return null;
}

function analyzeSubtask(subtask: Subtask): { hasAntiPattern: boolean; pattern?: string; field?: string } {
  // Check title
  let pattern = detectAntiPattern(subtask.title);
  if (pattern) {
    return { hasAntiPattern: true, pattern, field: 'title' };
  }
  
  // Check description
  pattern = detectAntiPattern(subtask.description);
  if (pattern) {
    return { hasAntiPattern: true, pattern, field: 'description' };
  }
  
  return { hasAntiPattern: false };
}

async function expandTask(task: Task): Promise<Subtask[]> {
  const request: ExpandTaskRequest = {
    operation: 'expand_task',
    model: 'claude-sonnet-4-20250514',
    payload: {
      task: {
        id: task.id.toString(),
        title: task.title,
        description: task.description,
        details: task.details || '',
        status: 'pending',
        dependencies: [],
      },
      subtask_count: 10, // Allow more subtasks to avoid combining components
      next_subtask_id: 1,
      enable_subagents: true,
    },
  };

  const proc = Bun.spawn(['bun', 'run', 'src/index.ts'], {
    cwd: join(import.meta.dir, '..'),
    stdin: new Blob([JSON.stringify(request)]),
    stdout: 'pipe',
    stderr: 'pipe',
  });

  const output = await new Response(proc.stdout).text();
  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    const stderr = await new Response(proc.stderr).text();
    console.error(`Task ${task.id} failed:`, stderr);
    return [];
  }

  try {
    const response: ExpandTaskResponse = JSON.parse(output);
    if (response.success && response.data?.subtasks) {
      return response.data.subtasks;
    }
    console.error(`Task ${task.id} returned error:`, response.error);
    return [];
  } catch (e) {
    console.error(`Task ${task.id} parse error:`, e);
    return [];
  }
}

async function main() {
  // Load tasks
  const tasksPath = join(import.meta.dir, 'alerthub-full/.tasks/tasks/tasks.json');
  const tasks: Task[] = JSON.parse(readFileSync(tasksPath, 'utf-8'));
  
  console.log(`\n🔍 Testing ${tasks.length} tasks for atomic subtask generation...\n`);
  
  const results: {
    taskId: number;
    taskTitle: string;
    subtaskCount: number;
    antiPatterns: { subtaskId: number; subtaskTitle: string; pattern: string; field: string }[];
  }[] = [];
  
  // Test first 10 tasks to ensure comprehensive coverage
  const tasksToTest = tasks.slice(0, 10);
  
  for (const task of tasksToTest) {
    console.log(`\n📋 Task ${task.id}: ${task.title}`);
    console.log('   Expanding...');
    
    const subtasks = await expandTask(task);
    
    if (subtasks.length === 0) {
      console.log('   ❌ Failed to expand task');
      continue;
    }
    
    console.log(`   Generated ${subtasks.length} subtasks:`);
    
    const antiPatterns: { subtaskId: number; subtaskTitle: string; pattern: string; field: string }[] = [];
    
    for (const subtask of subtasks) {
      const analysis = analyzeSubtask(subtask);
      const status = analysis.hasAntiPattern ? '❌' : '✅';
      console.log(`     ${status} ${subtask.id}. ${subtask.title}`);
      
      if (analysis.hasAntiPattern) {
        antiPatterns.push({
          subtaskId: subtask.id,
          subtaskTitle: subtask.title,
          pattern: analysis.pattern!,
          field: analysis.field!,
        });
      }
    }
    
    results.push({
      taskId: task.id,
      taskTitle: task.title,
      subtaskCount: subtasks.length,
      antiPatterns,
    });
  }
  
  // Summary
  console.log('\n' + '='.repeat(80));
  console.log('📊 SUMMARY');
  console.log('='.repeat(80));
  
  let totalTasks = results.length;
  let tasksWithAntiPatterns = results.filter(r => r.antiPatterns.length > 0).length;
  let totalSubtasks = results.reduce((acc, r) => acc + r.subtaskCount, 0);
  let totalAntiPatterns = results.reduce((acc, r) => acc + r.antiPatterns.length, 0);
  
  console.log(`\nTasks tested: ${totalTasks}`);
  console.log(`Total subtasks generated: ${totalSubtasks}`);
  console.log(`Tasks with anti-patterns: ${tasksWithAntiPatterns}`);
  console.log(`Total anti-pattern violations: ${totalAntiPatterns}`);
  
  if (totalAntiPatterns === 0) {
    console.log('\n✅ SUCCESS: All subtasks are atomic (one component per subtask)');
  } else {
    console.log('\n❌ FAILED: Some subtasks combine multiple components');
    for (const result of results) {
      if (result.antiPatterns.length > 0) {
        console.log(`\n  Task ${result.taskId}: ${result.taskTitle}`);
        for (const ap of result.antiPatterns) {
          console.log(`    - Subtask ${ap.subtaskId}: "${ap.subtaskTitle}"`);
          console.log(`      Pattern: ${ap.pattern} in ${ap.field}`);
        }
      }
    }
  }
  
  // Save results
  const resultsPath = join(import.meta.dir, 'test-results.json');
  writeFileSync(resultsPath, JSON.stringify(results, null, 2));
  console.log(`\nResults saved to: ${resultsPath}`);
}

main().catch(console.error);
