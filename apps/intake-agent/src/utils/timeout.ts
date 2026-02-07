/**
 * Timeout utilities for async operations.
 * Based on patterns from Taskmaster AI.
 */

/**
 * Wrap a promise with a timeout.
 */
export async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  operationName = 'Operation'
): Promise<T> {
  let timeoutHandle: ReturnType<typeof setTimeout>;

  const timeoutPromise = new Promise<never>((_, reject) => {
    timeoutHandle = setTimeout(() => {
      reject(new Error(`${operationName} timed out after ${timeoutMs / 1000} seconds`));
    }, timeoutMs);
  });

  try {
    const result = await Promise.race([promise, timeoutPromise]);
    clearTimeout(timeoutHandle!);
    return result;
  } catch (error) {
    clearTimeout(timeoutHandle!);
    throw error;
  }
}

/**
 * Soft timeout - returns default value instead of throwing.
 */
export async function withSoftTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  defaultValue: T
): Promise<T> {
  let timeoutHandle: ReturnType<typeof setTimeout>;

  const timeoutPromise = new Promise<T>((resolve) => {
    timeoutHandle = setTimeout(() => resolve(defaultValue), timeoutMs);
  });

  try {
    const result = await Promise.race([promise, timeoutPromise]);
    clearTimeout(timeoutHandle!);
    return result;
  } catch {
    clearTimeout(timeoutHandle!);
    return defaultValue;
  }
}

/**
 * Duration helpers for readable timeout specs.
 */
export const Duration = {
  seconds: (n: number) => n * 1000,
  minutes: (n: number) => n * 60 * 1000,
  hours: (n: number) => n * 60 * 60 * 1000,
};

/**
 * Default timeouts for various operations.
 */
export const Timeouts = {
  /** Parse PRD - scales with task count */
  parsePrd: (taskCount: number) => Duration.minutes(Math.max(5, Math.ceil(taskCount / 10))),
  
  /** Expand task into subtasks */
  expandTask: Duration.minutes(3),
  
  /** Analyze complexity */
  analyzeComplexity: Duration.minutes(2),
  
  /** Generic generation */
  generate: Duration.minutes(2),
  
  /** Research operations */
  research: Duration.minutes(10),
};
