/**
 * Enhanced logging utilities for intake-agent operations.
 * 
 * Features:
 * - Debug mode via LOG_LEVEL=debug or DEBUG=1
 * - Trace level for very granular logging (LOG_LEVEL=trace)
 * - Colored output for terminal readability
 * - Structured context logging
 * - Operation timing utilities
 * 
 * Usage:
 *   LOG_LEVEL=debug ./intake-agent    # Enable debug logging
 *   LOG_LEVEL=trace ./intake-agent    # Enable trace + debug logging
 *   DEBUG=1 ./intake-agent            # Shorthand for debug mode
 */

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'silent';

const LOG_LEVELS: Record<LogLevel, number> = {
  trace: 0,
  debug: 1,
  info: 2,
  warn: 3,
  error: 4,
  silent: 5,
};

// ANSI color codes for terminal output
const COLORS = {
  reset: '\x1b[0m',
  dim: '\x1b[2m',
  bright: '\x1b[1m',
  // Levels
  trace: '\x1b[90m',    // Gray
  debug: '\x1b[36m',    // Cyan
  info: '\x1b[32m',     // Green
  warn: '\x1b[33m',     // Yellow
  error: '\x1b[31m',    // Red
  // Context
  context: '\x1b[35m',  // Magenta
  time: '\x1b[90m',     // Gray
};

// Check if colors should be used
const useColors = process.stderr.isTTY !== false && !process.env.NO_COLOR;

/**
 * Apply color to text if colors are enabled.
 */
function color(text: string, colorCode: string): string {
  return useColors ? `${colorCode}${text}${COLORS.reset}` : text;
}

/**
 * Get the current log level from environment.
 */
function getLogLevel(): LogLevel {
  // DEBUG=1 is shorthand for debug mode
  if (process.env.DEBUG === '1' || process.env.DEBUG === 'true') {
    return 'debug';
  }
  
  const level = (process.env.LOG_LEVEL || 'info').toLowerCase() as LogLevel;
  return LOG_LEVELS[level] !== undefined ? level : 'info';
}

const currentLevel = getLogLevel();
const minLevel = LOG_LEVELS[currentLevel];

/**
 * Check if a log level is enabled.
 */
export function isLevelEnabled(level: LogLevel): boolean {
  return LOG_LEVELS[level] >= minLevel;
}

/**
 * Format context object for logging.
 */
function formatContext(context: Record<string, unknown>): string {
  const entries = Object.entries(context);
  if (entries.length === 0) return '';
  
  // For debug/trace, show full JSON
  if (isLevelEnabled('debug')) {
    return '\n' + color(JSON.stringify(context, null, 2), COLORS.context);
  }
  
  // For info+, show compact inline format
  const compact = entries
    .map(([k, v]) => {
      const val = typeof v === 'string' ? v : JSON.stringify(v);
      return `${k}=${val}`;
    })
    .join(' ');
  return color(` {${compact}}`, COLORS.dim);
}

/**
 * Log a message with level and optional context.
 */
export function log(level: LogLevel, message: string, context?: Record<string, unknown>): void {
  if (LOG_LEVELS[level] < minLevel) return;

  const timestamp = new Date().toISOString();
  const levelStr = level.toUpperCase().padEnd(5);
  
  const timeStr = color(timestamp, COLORS.time);
  const levelColor = COLORS[level] || COLORS.info;
  const levelColored = color(levelStr, levelColor);
  
  const contextStr = context ? formatContext(context) : '';
  
  console.error(`${timeStr} ${levelColored} ${message}${contextStr}`);
}

/**
 * Logger interface with all methods.
 */
export interface Logger {
  trace: (msg: string, ctx?: Record<string, unknown>) => void;
  debug: (msg: string, ctx?: Record<string, unknown>) => void;
  info: (msg: string, ctx?: Record<string, unknown>) => void;
  warn: (msg: string, ctx?: Record<string, unknown>) => void;
  error: (msg: string, ctx?: Record<string, unknown>) => void;
  /** Check if debug logging is enabled */
  isDebug: () => boolean;
  /** Check if trace logging is enabled */
  isTrace: () => boolean;
  /** Create a child logger with additional prefix */
  child: (childPrefix: string) => Logger;
}

/**
 * Create a logger with a fixed prefix.
 */
export function createLogger(prefix: string): Logger {
  const prefixStr = color(`[${prefix}]`, COLORS.bright);
  
  const logger: Logger = {
    trace: (msg, ctx) => log('trace', `${prefixStr} ${msg}`, ctx),
    debug: (msg, ctx) => log('debug', `${prefixStr} ${msg}`, ctx),
    info: (msg, ctx) => log('info', `${prefixStr} ${msg}`, ctx),
    warn: (msg, ctx) => log('warn', `${prefixStr} ${msg}`, ctx),
    error: (msg, ctx) => log('error', `${prefixStr} ${msg}`, ctx),
    isDebug: () => isLevelEnabled('debug'),
    isTrace: () => isLevelEnabled('trace'),
    child: (childPrefix) => createLogger(`${prefix}:${childPrefix}`),
  };
  
  return logger;
}

/**
 * Root logger for the application.
 */
export const rootLogger = createLogger('intake');

/**
 * Log operation timing with automatic start/end.
 */
export async function withTiming<T>(
  operation: string,
  fn: () => Promise<T>,
  logger: Logger = rootLogger
): Promise<T> {
  const start = Date.now();
  logger.debug(`${operation} started`);
  
  try {
    const result = await fn();
    const elapsed = Date.now() - start;
    logger.info(`${operation} completed`, { elapsed_ms: elapsed });
    return result;
  } catch (error) {
    const elapsed = Date.now() - start;
    logger.error(`${operation} failed`, { 
      elapsed_ms: elapsed, 
      error: error instanceof Error ? error.message : String(error) 
    });
    throw error;
  }
}

/**
 * Create a timing tracker for multi-step operations.
 */
export function createTimer(operation: string, logger: Logger = rootLogger) {
  const start = Date.now();
  const steps: Array<{ name: string; elapsed: number }> = [];
  
  return {
    /** Mark a step completion */
    step(name: string) {
      const elapsed = Date.now() - start;
      steps.push({ name, elapsed });
      logger.debug(`${operation}:${name}`, { elapsed_ms: elapsed });
    },
    
    /** Complete the operation and log summary */
    done(context?: Record<string, unknown>) {
      const total = Date.now() - start;
      logger.info(`${operation} complete`, {
        total_ms: total,
        steps: steps.map(s => `${s.name}:${s.elapsed}ms`).join(' → '),
        ...context,
      });
      return total;
    },
    
    /** Get elapsed time without completing */
    elapsed() {
      return Date.now() - start;
    },
  };
}

/**
 * Log a separator line for visual grouping.
 */
export function logSeparator(label?: string, logger: Logger = rootLogger): void {
  if (!isLevelEnabled('debug')) return;
  
  const line = '─'.repeat(60);
  if (label) {
    logger.debug(`${line} ${label} ${line}`);
  } else {
    logger.debug(line);
  }
}

/**
 * Log the current configuration for debugging.
 */
export function logConfig(logger: Logger = rootLogger): void {
  if (!isLevelEnabled('debug')) return;
  
  logger.debug('Configuration', {
    log_level: currentLevel,
    colors: useColors,
    env: {
      LOG_LEVEL: process.env.LOG_LEVEL || '(not set)',
      DEBUG: process.env.DEBUG || '(not set)',
      NO_COLOR: process.env.NO_COLOR || '(not set)',
    },
  });
}
