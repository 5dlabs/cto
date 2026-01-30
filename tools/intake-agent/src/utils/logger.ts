/**
 * Logging utilities for intake-agent operations.
 * Provides structured logging with levels and context.
 */

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

// Get log level from env, default to 'info'
const currentLevel = (process.env.LOG_LEVEL as LogLevel) || 'info';
const minLevel = LOG_LEVELS[currentLevel] ?? LOG_LEVELS.info;

/**
 * Log a message with level and optional context.
 */
export function log(level: LogLevel, message: string, context?: Record<string, unknown>): void {
  if (LOG_LEVELS[level] < minLevel) return;

  const timestamp = new Date().toISOString();
  const prefix = `[${timestamp}] [${level.toUpperCase()}]`;
  
  if (context) {
    console.error(`${prefix} ${message}`, JSON.stringify(context, null, 2));
  } else {
    console.error(`${prefix} ${message}`);
  }
}

/**
 * Create a logger with a fixed prefix.
 */
export function createLogger(prefix: string) {
  return {
    debug: (msg: string, ctx?: Record<string, unknown>) => log('debug', `[${prefix}] ${msg}`, ctx),
    info: (msg: string, ctx?: Record<string, unknown>) => log('info', `[${prefix}] ${msg}`, ctx),
    warn: (msg: string, ctx?: Record<string, unknown>) => log('warn', `[${prefix}] ${msg}`, ctx),
    error: (msg: string, ctx?: Record<string, unknown>) => log('error', `[${prefix}] ${msg}`, ctx),
  };
}

/**
 * Log operation timing.
 */
export function withTiming<T>(
  operation: string,
  fn: () => Promise<T>,
  logger = createLogger('timing')
): Promise<T> {
  const start = Date.now();
  logger.info(`${operation} started`);
  
  return fn()
    .then((result) => {
      const elapsed = Date.now() - start;
      logger.info(`${operation} completed`, { elapsed_ms: elapsed });
      return result;
    })
    .catch((error) => {
      const elapsed = Date.now() - start;
      logger.error(`${operation} failed`, { elapsed_ms: elapsed, error: String(error) });
      throw error;
    });
}
