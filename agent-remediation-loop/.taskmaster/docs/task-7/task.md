# Task 7: Build Label-Based Workflow Orchestration

## Overview
Implement a comprehensive PR label management system for tracking remediation state and iteration counts. This system provides workflow state tracking through GitHub labels, enabling automated state transitions, iteration tracking, and human override capabilities.

## Technical Context
The Agent Remediation Loop requires clear state visibility and control mechanisms. GitHub PR labels provide an ideal interface for both automated systems and human operators to understand and control workflow state. This system creates a sophisticated label-based state machine that tracks remediation progress and provides override capabilities.

### Label Schema Design
The system uses a hierarchical label structure:
- **Task Association**: `task-{id}` - Persistent task identification
- **Iteration Tracking**: `iteration-{n}` - Current remediation cycle
- **Status Labels**: Workflow state indicators
- **Override Controls**: Human intervention capabilities

## Implementation Guide

### Step 1: Design and Document Label Schema

#### 1.1 Complete Label Schema Definition
```typescript
// Label schema types and interfaces
interface TaskLabel {
  pattern: 'task-{id}';
  examples: ['task-42', 'task-123'];
  lifecycle: 'permanent'; // Never automatically removed
  purpose: 'task association';
}

interface IterationLabel {
  pattern: 'iteration-{n}';
  examples: ['iteration-1', 'iteration-3', 'iteration-10'];
  lifecycle: 'updated-per-cycle';
  purpose: 'iteration tracking';
}

interface StatusLabel {
  labels: {
    'needs-remediation': 'Added by Tess when issues found',
    'remediation-in-progress': 'Active during Rex fixes',
    'ready-for-qa': 'Added after Cleo completion',
    'approved': 'Final success state',
    'failed-remediation': 'Max iterations reached',
  };
  lifecycle: 'state-based';
  purpose: 'workflow status';
}

interface OverrideLabel {
  labels: {
    'skip-automation': 'Disables all automated workflows',
    'manual-review-required': 'Human intervention needed',
    'pause-remediation': 'Temporary workflow suspension',
  };
  lifecycle: 'manual-control';
  purpose: 'human override';
}
```

#### 1.2 State Machine Definition
```typescript
enum WorkflowState {
  Initial = 'initial',
  NeedsRemediation = 'needs-remediation',
  RemediationInProgress = 'remediation-in-progress',
  ReadyForQA = 'ready-for-qa',
  Approved = 'approved',
  Failed = 'failed-remediation',
  ManualOverride = 'manual-override'
}

interface StateTransition {
  from: WorkflowState;
  to: WorkflowState;
  trigger: string;
  conditions?: string[];
  actions: string[];
}

const STATE_MACHINE: StateTransition[] = [
  {
    from: WorkflowState.Initial,
    to: WorkflowState.NeedsRemediation,
    trigger: 'tess_feedback_received',
    actions: ['add_needs_remediation', 'increment_iteration']
  },
  {
    from: WorkflowState.NeedsRemediation,
    to: WorkflowState.RemediationInProgress,
    trigger: 'rex_remediation_started',
    actions: ['remove_needs_remediation', 'add_remediation_in_progress']
  },
  {
    from: WorkflowState.RemediationInProgress,
    to: WorkflowState.ReadyForQA,
    trigger: 'rex_remediation_completed',
    actions: ['remove_remediation_in_progress', 'add_ready_for_qa']
  },
  {
    from: WorkflowState.ReadyForQA,
    to: WorkflowState.NeedsRemediation,
    trigger: 'tess_additional_feedback',
    actions: ['remove_ready_for_qa', 'add_needs_remediation', 'increment_iteration']
  },
  {
    from: WorkflowState.ReadyForQA,
    to: WorkflowState.Approved,
    trigger: 'tess_approval',
    actions: ['remove_ready_for_qa', 'add_approved']
  },
  {
    from: WorkflowState.RemediationInProgress,
    to: WorkflowState.Failed,
    trigger: 'max_iterations_reached',
    conditions: ['iteration >= 10'],
    actions: ['remove_remediation_in_progress', 'add_failed_remediation']
  }
];
```

### Step 2: Implement GitHub API Label Integration

#### 2.1 Label API Client Implementation
```typescript
import { Octokit } from '@octokit/rest';
import { retry } from '@octokit/plugin-retry';
import { throttling } from '@octokit/plugin-throttling';

const OctokitWithPlugins = Octokit.plugin(retry, throttling);

export class GitHubLabelClient {
  private octokit: InstanceType<typeof OctokitWithPlugins>;
  private owner: string;
  private repo: string;

  constructor(token: string, owner: string, repo: string) {
    this.owner = owner;
    this.repo = repo;
    this.octokit = new OctokitWithPlugins({
      auth: token,
      throttle: {
        onRateLimit: (retryAfter, options) => {
          console.warn(`Rate limit exceeded, retrying after ${retryAfter} seconds`);
          return true; // Retry
        },
        onSecondaryRateLimit: (retryAfter, options) => {
          console.warn(`Secondary rate limit exceeded, retrying after ${retryAfter} seconds`);
          return true; // Retry
        },
      },
      retry: {
        doNotRetry: ['400', '401', '403', '404', '422'],
      },
    });
  }

  async getLabels(prNumber: number): Promise<string[]> {
    try {
      const { data: pr } = await this.octokit.rest.pulls.get({
        owner: this.owner,
        repo: this.repo,
        pull_number: prNumber,
      });

      return pr.labels.map(label => label.name);
    } catch (error) {
      throw new LabelOperationError(`Failed to get labels for PR ${prNumber}`, error);
    }
  }

  async addLabels(prNumber: number, labels: string[]): Promise<void> {
    if (labels.length === 0) return;

    try {
      await this.octokit.rest.issues.addLabels({
        owner: this.owner,
        repo: this.repo,
        issue_number: prNumber,
        labels,
      });
    } catch (error) {
      throw new LabelOperationError(`Failed to add labels ${labels.join(', ')} to PR ${prNumber}`, error);
    }
  }

  async removeLabel(prNumber: number, label: string): Promise<void> {
    try {
      await this.octokit.rest.issues.removeLabel({
        owner: this.owner,
        repo: this.repo,
        issue_number: prNumber,
        name: label,
      });
    } catch (error) {
      if (error.status === 404) {
        // Label doesn't exist, which is fine for removal
        return;
      }
      throw new LabelOperationError(`Failed to remove label ${label} from PR ${prNumber}`, error);
    }
  }

  async replaceLabels(prNumber: number, labels: string[]): Promise<void> {
    try {
      await this.octokit.rest.issues.replaceAllLabels({
        owner: this.owner,
        repo: this.repo,
        issue_number: prNumber,
        labels,
      });
    } catch (error) {
      throw new LabelOperationError(`Failed to replace labels on PR ${prNumber}`, error);
    }
  }

  async updateLabelsAtomic(prNumber: number, operations: LabelOperation[]): Promise<void> {
    const maxRetries = 5;
    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        // Get current state with ETag
        const { data: pr, headers } = await this.octokit.rest.pulls.get({
          owner: this.owner,
          repo: this.repo,
          pull_number: prNumber,
        });

        const etag = headers.etag;
        const currentLabels = pr.labels.map(label => label.name);
        
        // Calculate new labels
        const newLabels = this.calculateNewLabels(currentLabels, operations);
        
        // Attempt atomic update
        await this.octokit.rest.issues.replaceAllLabels({
          owner: this.owner,
          repo: this.repo,
          issue_number: prNumber,
          labels: newLabels,
          headers: {
            'If-Match': etag,
          },
        });

        return; // Success
      } catch (error) {
        lastError = error;
        
        if (error.status === 412) {
          // Precondition failed - concurrent modification
          const backoffMs = Math.min(1000 * Math.pow(2, attempt - 1), 5000);
          await this.delay(backoffMs + Math.random() * 1000); // Add jitter
          continue;
        }
        
        // Other errors are not retryable
        throw new LabelOperationError(`Atomic label update failed on PR ${prNumber}`, error);
      }
    }

    throw new LabelOperationError(`Atomic label update failed after ${maxRetries} attempts`, lastError);
  }

  private calculateNewLabels(currentLabels: string[], operations: LabelOperation[]): string[] {
    const labelSet = new Set(currentLabels);

    for (const operation of operations) {
      switch (operation.type) {
        case 'add':
          operation.labels.forEach(label => labelSet.add(label));
          break;
        case 'remove':
          operation.labels.forEach(label => labelSet.delete(label));
          break;
        case 'replace':
          if (operation.fromLabel) {
            labelSet.delete(operation.fromLabel);
          }
          operation.labels.forEach(label => labelSet.add(label));
          break;
      }
    }

    return Array.from(labelSet);
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

interface LabelOperation {
  type: 'add' | 'remove' | 'replace';
  labels: string[];
  fromLabel?: string; // For replace operations
}

class LabelOperationError extends Error {
  public readonly cause: any;

  constructor(message: string, cause?: any) {
    super(message);
    this.name = 'LabelOperationError';
    this.cause = cause;
  }
}
```

### Step 3: Build State Transition Logic Engine

#### 3.1 Label Orchestrator Implementation
```typescript
export class LabelOrchestrator {
  private labelClient: GitHubLabelClient;
  private stateManager: StateManager; // From Task 4
  
  constructor(labelClient: GitHubLabelClient, stateManager: StateManager) {
    this.labelClient = labelClient;
    this.stateManager = stateManager;
  }

  async transitionState(
    prNumber: number, 
    taskId: string,
    trigger: string, 
    context?: any
  ): Promise<void> {
    // Check for override labels first
    const currentLabels = await this.labelClient.getLabels(prNumber);
    
    if (this.hasOverrideLabel(currentLabels)) {
      throw new AutomationDisabledError('Automation disabled by human override');
    }

    // Get current state
    const currentState = this.determineCurrentState(currentLabels);
    
    // Find valid transition
    const transition = this.findTransition(currentState, trigger);
    if (!transition) {
      throw new InvalidTransitionError(`No valid transition from ${currentState} with trigger ${trigger}`);
    }

    // Check transition conditions
    if (!await this.checkTransitionConditions(transition, taskId, context)) {
      throw new TransitionConditionError(`Transition conditions not met for ${trigger}`);
    }

    // Execute transition
    await this.executeTransition(prNumber, taskId, transition, context);
  }

  private hasOverrideLabel(labels: string[]): boolean {
    const overrideLabels = ['skip-automation', 'manual-review-required', 'pause-remediation'];
    return labels.some(label => overrideLabels.includes(label));
  }

  private determineCurrentState(labels: string[]): WorkflowState {
    if (labels.includes('approved')) return WorkflowState.Approved;
    if (labels.includes('failed-remediation')) return WorkflowState.Failed;
    if (labels.includes('ready-for-qa')) return WorkflowState.ReadyForQA;
    if (labels.includes('remediation-in-progress')) return WorkflowState.RemediationInProgress;
    if (labels.includes('needs-remediation')) return WorkflowState.NeedsRemediation;
    
    return WorkflowState.Initial;
  }

  private findTransition(currentState: WorkflowState, trigger: string): StateTransition | null {
    return STATE_MACHINE.find(
      transition => transition.from === currentState && transition.trigger === trigger
    ) || null;
  }

  private async checkTransitionConditions(
    transition: StateTransition, 
    taskId: string, 
    context?: any
  ): Promise<boolean> {
    if (!transition.conditions) return true;

    for (const condition of transition.conditions) {
      if (condition.startsWith('iteration ')) {
        const currentIteration = await this.getCurrentIteration(taskId);
        const conditionResult = this.evaluateIterationCondition(condition, currentIteration);
        if (!conditionResult) return false;
      }
      
      // Add more condition types as needed
    }

    return true;
  }

  private async executeTransition(
    prNumber: number,
    taskId: string,
    transition: StateTransition,
    context?: any
  ): Promise<void> {
    const operations: LabelOperation[] = [];
    let iterationUpdate: number | null = null;

    // Process transition actions
    for (const action of transition.actions) {
      switch (action) {
        case 'add_needs_remediation':
          operations.push({ type: 'add', labels: ['needs-remediation'] });
          break;
        case 'remove_needs_remediation':
          operations.push({ type: 'remove', labels: ['needs-remediation'] });
          break;
        case 'add_remediation_in_progress':
          operations.push({ type: 'add', labels: ['remediation-in-progress'] });
          break;
        case 'remove_remediation_in_progress':
          operations.push({ type: 'remove', labels: ['remediation-in-progress'] });
          break;
        case 'add_ready_for_qa':
          operations.push({ type: 'add', labels: ['ready-for-qa'] });
          break;
        case 'remove_ready_for_qa':
          operations.push({ type: 'remove', labels: ['ready-for-qa'] });
          break;
        case 'add_approved':
          operations.push({ type: 'add', labels: ['approved'] });
          break;
        case 'add_failed_remediation':
          operations.push({ type: 'add', labels: ['failed-remediation'] });
          break;
        case 'increment_iteration':
          iterationUpdate = await this.incrementIteration(taskId, operations);
          break;
      }
    }

    // Execute label operations atomically
    if (operations.length > 0) {
      await this.labelClient.updateLabelsAtomic(prNumber, operations);
    }

    // Log state transition
    console.log(`State transition: ${transition.from} -> ${transition.to} (${transition.trigger})`, {
      prNumber,
      taskId,
      iteration: iterationUpdate,
      context
    });
  }

  private async incrementIteration(taskId: string, operations: LabelOperation[]): Promise<number> {
    // Get current iteration from state manager
    const newIteration = await this.stateManager.incrementIteration(taskId);
    
    // Remove old iteration label and add new one
    const currentLabels = await this.labelClient.getLabels(prNumber);
    const oldIterationLabel = currentLabels.find(label => label.startsWith('iteration-'));
    
    if (oldIterationLabel) {
      operations.push({ type: 'remove', labels: [oldIterationLabel] });
    }
    
    operations.push({ type: 'add', labels: [`iteration-${newIteration}`] });
    
    return newIteration;
  }

  private async getCurrentIteration(taskId: string): Promise<number> {
    return await this.stateManager.getCurrentIteration(taskId);
  }

  private evaluateIterationCondition(condition: string, currentIteration: number): boolean {
    const match = condition.match(/iteration\s*(>=|<=|>|<|==)\s*(\d+)/);
    if (!match) return false;

    const [, operator, valueStr] = match;
    const value = parseInt(valueStr, 10);

    switch (operator) {
      case '>=': return currentIteration >= value;
      case '<=': return currentIteration <= value;
      case '>': return currentIteration > value;
      case '<': return currentIteration < value;
      case '==': return currentIteration === value;
      default: return false;
    }
  }
}

class AutomationDisabledError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AutomationDisabledError';
  }
}

class InvalidTransitionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'InvalidTransitionError';
  }
}

class TransitionConditionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TransitionConditionError';
  }
}
```

### Step 4: Add Skip-Automation Override Detection

#### 4.1 Override Detection System
```typescript
export class OverrideDetector {
  private labelClient: GitHubLabelClient;
  private notificationService: NotificationService;

  constructor(labelClient: GitHubLabelClient, notificationService: NotificationService) {
    this.labelClient = labelClient;
    this.notificationService = notificationService;
  }

  async checkOverrideStatus(prNumber: number, taskId: string): Promise<OverrideStatus> {
    const labels = await this.labelClient.getLabels(prNumber);
    
    const overrides = this.detectOverrides(labels);
    
    if (overrides.length > 0) {
      await this.logOverrideEvent(prNumber, taskId, overrides);
      return { 
        hasOverride: true, 
        overrideType: overrides[0].type,
        message: overrides[0].message,
        overrides: overrides
      };
    }

    return { hasOverride: false };
  }

  private detectOverrides(labels: string[]): Override[] {
    const overrides: Override[] = [];

    if (labels.includes('skip-automation')) {
      overrides.push({
        type: 'skip-automation',
        message: 'All automation disabled by skip-automation label',
        severity: 'high',
        action: 'halt_all_automation'
      });
    }

    if (labels.includes('manual-review-required')) {
      overrides.push({
        type: 'manual-review-required',
        message: 'Manual review required before automation continues',
        severity: 'medium',
        action: 'pause_until_review'
      });
    }

    if (labels.includes('pause-remediation')) {
      overrides.push({
        type: 'pause-remediation',
        message: 'Remediation temporarily paused',
        severity: 'low',
        action: 'pause_remediation_only'
      });
    }

    return overrides;
  }

  private async logOverrideEvent(prNumber: number, taskId: string, overrides: Override[]): Promise<void> {
    for (const override of overrides) {
      console.log(`Override detected: ${override.type}`, {
        prNumber,
        taskId,
        severity: override.severity,
        message: override.message
      });

      // Send notification to relevant channels
      await this.notificationService.sendOverrideAlert({
        prNumber,
        taskId,
        overrideType: override.type,
        message: override.message,
        severity: override.severity
      });
    }
  }

  async createBypassRequest(prNumber: number, taskId: string, reason: string, requester: string): Promise<BypassRequest> {
    // This would integrate with an approval system for emergency bypasses
    const bypassRequest: BypassRequest = {
      id: `bypass-${prNumber}-${Date.now()}`,
      prNumber,
      taskId,
      reason,
      requester,
      status: 'pending',
      createdAt: new Date(),
      approvers: []
    };

    // Store bypass request (implementation depends on storage mechanism)
    await this.storeBypassRequest(bypassRequest);
    
    // Notify approvers
    await this.notificationService.sendBypassRequest(bypassRequest);

    return bypassRequest;
  }

  private async storeBypassRequest(request: BypassRequest): Promise<void> {
    // Implementation would store in database or configuration system
  }
}

interface Override {
  type: string;
  message: string;
  severity: 'low' | 'medium' | 'high';
  action: string;
}

interface OverrideStatus {
  hasOverride: boolean;
  overrideType?: string;
  message?: string;
  overrides?: Override[];
}

interface BypassRequest {
  id: string;
  prNumber: number;
  taskId: string;
  reason: string;
  requester: string;
  status: 'pending' | 'approved' | 'denied';
  createdAt: Date;
  approvers: string[];
}
```

### Step 5: Implement Label Cleanup System

#### 5.1 Cleanup Manager Implementation
```typescript
export class LabelCleanupManager {
  private labelClient: GitHubLabelClient;
  private stateManager: StateManager;

  constructor(labelClient: GitHubLabelClient, stateManager: StateManager) {
    this.labelClient = labelClient;
    this.stateManager = stateManager;
  }

  async cleanupCompletedTask(prNumber: number, taskId: string): Promise<void> {
    const currentLabels = await this.labelClient.getLabels(prNumber);
    const state = this.determineCurrentState(currentLabels);

    if (!this.isTerminalState(state)) {
      throw new CleanupError(`Cannot cleanup non-terminal state: ${state}`);
    }

    const labelsToRemove = this.determineCleanupLabels(currentLabels, state);
    const labelsToKeep = this.determineHistoryLabels(currentLabels, taskId);

    await this.performCleanup(prNumber, labelsToRemove, labelsToKeep);
    
    console.log(`Cleanup completed for task ${taskId} on PR ${prNumber}`, {
      finalState: state,
      labelsRemoved: labelsToRemove,
      labelsKept: labelsToKeep
    });
  }

  async cleanupAbandonedTask(prNumber: number, taskId: string, ttlDays: number = 30): Promise<void> {
    const state = await this.stateManager.getState(taskId);
    
    if (!state || this.isRecentlyActive(state, ttlDays)) {
      return; // Task is not abandoned
    }

    const currentLabels = await this.labelClient.getLabels(prNumber);
    const workflowLabels = this.getWorkflowLabels(currentLabels);
    
    // Remove all workflow labels but keep task association
    const operations: LabelOperation[] = [
      { type: 'remove', labels: workflowLabels.filter(label => !label.startsWith('task-')) }
    ];

    await this.labelClient.updateLabelsAtomic(prNumber, operations);
    
    console.log(`Abandoned task cleanup completed for ${taskId}`, {
      prNumber,
      labelsRemoved: workflowLabels.length,
      daysSinceLastUpdate: this.daysSinceUpdate(state)
    });
  }

  async performScheduledCleanup(): Promise<CleanupResult> {
    // This would be called by a scheduled job
    const result: CleanupResult = {
      tasksProcessed: 0,
      tasksAbandoned: 0,
      tasksCompleted: 0,
      errors: []
    };

    try {
      // Get list of active tasks from state manager
      const activeTasks = await this.stateManager.getActiveTasks();
      
      for (const task of activeTasks) {
        try {
          result.tasksProcessed++;
          
          if (this.isTerminalState(task.status)) {
            await this.cleanupCompletedTask(task.prNumber, task.taskId);
            result.tasksCompleted++;
          } else if (this.isRecentlyActive(task, 30)) {
            await this.cleanupAbandonedTask(task.prNumber, task.taskId);
            result.tasksAbandoned++;
          }
        } catch (error) {
          result.errors.push(`Task ${task.taskId}: ${error.message}`);
        }
      }
    } catch (error) {
      result.errors.push(`Cleanup job failed: ${error.message}`);
    }

    return result;
  }

  private determineCurrentState(labels: string[]): WorkflowState {
    // Implementation same as in LabelOrchestrator
    if (labels.includes('approved')) return WorkflowState.Approved;
    if (labels.includes('failed-remediation')) return WorkflowState.Failed;
    if (labels.includes('ready-for-qa')) return WorkflowState.ReadyForQA;
    if (labels.includes('remediation-in-progress')) return WorkflowState.RemediationInProgress;
    if (labels.includes('needs-remediation')) return WorkflowState.NeedsRemediation;
    return WorkflowState.Initial;
  }

  private isTerminalState(state: WorkflowState): boolean {
    return [WorkflowState.Approved, WorkflowState.Failed].includes(state);
  }

  private determineCleanupLabels(labels: string[], finalState: WorkflowState): string[] {
    const workflowLabels = [
      'needs-remediation',
      'remediation-in-progress', 
      'ready-for-qa'
    ];

    return labels.filter(label => 
      workflowLabels.includes(label) || label.startsWith('iteration-')
    );
  }

  private determineHistoryLabels(labels: string[], taskId: string): string[] {
    // Labels to keep for history
    return labels.filter(label => 
      label.startsWith('task-') || 
      label === 'approved' || 
      label === 'failed-remediation'
    );
  }

  private async performCleanup(prNumber: number, labelsToRemove: string[], labelsToKeep: string[]): Promise<void> {
    if (labelsToRemove.length === 0) return;

    const operations: LabelOperation[] = [
      { type: 'remove', labels: labelsToRemove }
    ];

    await this.labelClient.updateLabelsAtomic(prNumber, operations);
  }

  private getWorkflowLabels(labels: string[]): string[] {
    const workflowPrefixes = ['iteration-', 'task-'];
    const workflowLabels = [
      'needs-remediation', 
      'remediation-in-progress', 
      'ready-for-qa', 
      'approved', 
      'failed-remediation'
    ];

    return labels.filter(label =>
      workflowLabels.includes(label) ||
      workflowPrefixes.some(prefix => label.startsWith(prefix))
    );
  }

  private isRecentlyActive(state: any, ttlDays: number): boolean {
    const cutoffDate = new Date(Date.now() - ttlDays * 24 * 60 * 60 * 1000);
    return state.lastUpdate > cutoffDate;
  }

  private daysSinceUpdate(state: any): number {
    const now = new Date();
    const lastUpdate = new Date(state.lastUpdate);
    return Math.floor((now.getTime() - lastUpdate.getTime()) / (24 * 60 * 60 * 1000));
  }
}

interface CleanupResult {
  tasksProcessed: number;
  tasksAbandoned: number;
  tasksCompleted: number;
  errors: string[];
}

class CleanupError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CleanupError';
  }
}
```

### Step 6: Handle Concurrent Label Updates

#### 6.1 Concurrency Control Implementation
```typescript
export class ConcurrentLabelManager {
  private labelClient: GitHubLabelClient;
  private lockManager: Map<number, Promise<void>>; // PR-based locking
  
  constructor(labelClient: GitHubLabelClient) {
    this.labelClient = labelClient;
    this.lockManager = new Map();
  }

  async withLock<T>(prNumber: number, operation: () => Promise<T>): Promise<T> {
    // Get or create lock for this PR
    const existingLock = this.lockManager.get(prNumber);
    
    const newLock = (existingLock || Promise.resolve()).then(async () => {
      try {
        return await operation();
      } finally {
        // Clean up lock if it's the current one
        if (this.lockManager.get(prNumber) === newLock) {
          this.lockManager.delete(prNumber);
        }
      }
    });

    this.lockManager.set(prNumber, newLock);
    return await newLock;
  }

  async updateLabelsWithRetry(
    prNumber: number, 
    operations: LabelOperation[], 
    maxRetries: number = 5
  ): Promise<void> {
    return this.withLock(prNumber, async () => {
      let lastError: Error | null = null;

      for (let attempt = 1; attempt <= maxRetries; attempt++) {
        try {
          await this.labelClient.updateLabelsAtomic(prNumber, operations);
          return; // Success
        } catch (error) {
          lastError = error;
          
          if (this.isRetryableError(error)) {
            const backoffMs = this.calculateBackoff(attempt);
            await this.delay(backoffMs);
            continue;
          }
          
          // Non-retryable error
          throw error;
        }
      }

      throw new ConcurrentUpdateError(
        `Failed to update labels after ${maxRetries} attempts`,
        lastError
      );
    });
  }

  async batchOperations(operations: BatchOperation[]): Promise<BatchResult> {
    const results: BatchResult = {
      successful: [],
      failed: []
    };

    // Group operations by PR number
    const groupedOps = this.groupOperationsByPR(operations);
    
    // Execute operations for each PR concurrently
    const promises = Object.entries(groupedOps).map(async ([prNumberStr, ops]) => {
      const prNumber = parseInt(prNumberStr, 10);
      
      try {
        await this.updateLabelsWithRetry(prNumber, ops);
        results.successful.push(prNumber);
      } catch (error) {
        results.failed.push({ prNumber, error: error.message });
      }
    });

    await Promise.all(promises);
    return results;
  }

  private groupOperationsByPR(operations: BatchOperation[]): Record<number, LabelOperation[]> {
    const grouped: Record<number, LabelOperation[]> = {};
    
    for (const batchOp of operations) {
      if (!grouped[batchOp.prNumber]) {
        grouped[batchOp.prNumber] = [];
      }
      grouped[batchOp.prNumber].push(batchOp.operation);
    }
    
    return grouped;
  }

  private isRetryableError(error: any): boolean {
    // Check for specific error conditions that should trigger retry
    if (error.status === 412) return true; // Precondition failed
    if (error.status === 409) return true; // Conflict
    if (error.status === 502) return true; // Bad gateway
    if (error.status === 503) return true; // Service unavailable
    if (error.status === 504) return true; // Gateway timeout
    if (error.message?.includes('rate limit')) return true;
    
    return false;
  }

  private calculateBackoff(attempt: number): number {
    // Exponential backoff with jitter
    const baseDelay = 1000; // 1 second
    const maxDelay = 30000;  // 30 seconds
    const exponentialDelay = baseDelay * Math.pow(2, attempt - 1);
    const cappedDelay = Math.min(exponentialDelay, maxDelay);
    
    // Add jitter (±25%)
    const jitter = cappedDelay * 0.25 * (Math.random() - 0.5);
    return Math.max(cappedDelay + jitter, baseDelay);
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

interface BatchOperation {
  prNumber: number;
  operation: LabelOperation;
}

interface BatchResult {
  successful: number[];
  failed: { prNumber: number; error: string }[];
}

class ConcurrentUpdateError extends Error {
  public readonly cause: any;

  constructor(message: string, cause?: any) {
    super(message);
    this.name = 'ConcurrentUpdateError';
    this.cause = cause;
  }
}
```

## Integration Points

### State Management Integration
The label orchestration system integrates closely with Task 4's state management:
- Iteration tracking synchronized between labels and state
- State transitions trigger label updates
- Label changes update internal state tracking

### Sensor Integration
Integration with existing sensors for automated state transitions:
- Webhook events trigger state transitions
- Sensor payloads provide context for transitions
- Label updates trigger downstream processing

### Human Interface
Labels provide clear visibility and control:
- Developers can see current workflow state
- QA team can understand iteration progress
- Override labels provide emergency controls

## Performance Considerations

### API Rate Limiting
- Implement comprehensive retry logic
- Use ETag-based conditional requests
- Batch operations where possible
- Monitor and respect GitHub rate limits

### Concurrency Management
- Per-PR locking prevents race conditions
- Atomic operations ensure consistency
- Optimistic concurrency with conflict resolution

### Scalability
- Stateless orchestration logic
- Efficient label querying and filtering
- Minimal GitHub API calls per operation

## Success Criteria
- Complete label schema implemented with consistent naming
- State transitions work reliably across all workflow states
- Concurrent label updates handled without conflicts
- Override system provides human control capabilities
- Cleanup system maintains label hygiene
- Integration with existing sensors and state management
- Performance requirements met under load
- Comprehensive monitoring and alerting