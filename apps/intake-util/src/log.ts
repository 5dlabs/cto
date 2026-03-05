/**
 * Structured pipeline logger — outputs JSON to stderr.
 * Lobster captures stdout for step data; Fluent-Bit captures stderr for observability.
 */

type LogLevel = 'info' | 'warn' | 'error';
type ActivityType = 'thought' | 'action' | 'elicitation' | 'response' | 'error';

interface LogContext {
  runId?: string;
  agent?: string;
  step?: string;
  cli?: string;
  activityType?: ActivityType;
}

export function log(level: LogLevel, message: string, context: LogContext = {}): void {
  const entry = {
    level,
    message,
    timestamp: new Date().toISOString(),
    ...context,
  };
  process.stderr.write(JSON.stringify(entry) + '\n');
}
