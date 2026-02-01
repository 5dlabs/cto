/**
 * Metrics Collection and Monitoring
 * Provides application metrics for observability and monitoring
 */

class MetricsCollector {
  constructor() {
    this.metrics = {
      http: {
        requests: {
          total: 0,
          byMethod: {},
          byStatus: {},
          byPath: {},
        },
        responseTime: {
          sum: 0,
          count: 0,
          min: Infinity,
          max: 0,
          p50: [],
          p95: [],
          p99: [],
        },
      },
      tasks: {
        total: 0,
        created: 0,
        updated: 0,
        deleted: 0,
        byStatus: {
          todo: 0,
          in_progress: 0,
          completed: 0,
          archived: 0,
        },
        byPriority: {
          low: 0,
          medium: 0,
          high: 0,
          urgent: 0,
        },
      },
      auth: {
        logins: { success: 0, failed: 0 },
        registrations: 0,
        activeTokens: 0,
      },
      errors: {
        total: 0,
        byType: {},
        byStatusCode: {},
      },
      system: {
        startTime: Date.now(),
        uptime: 0,
      },
    };

    // Update uptime every second
    this.uptimeInterval = setInterval(() => {
      this.metrics.system.uptime = Date.now() - this.metrics.system.startTime;
    }, 1000);
  }

  /**
   * Record HTTP request
   */
  recordHttpRequest(method, path, statusCode, duration) {
    const metrics = this.metrics.http;

    // Total requests
    metrics.requests.total++;

    // By method
    metrics.requests.byMethod[method] = (metrics.requests.byMethod[method] || 0) + 1;

    // By status code
    const statusCategory = `${Math.floor(statusCode / 100)}xx`;
    metrics.requests.byStatus[statusCategory] = (metrics.requests.byStatus[statusCategory] || 0) + 1;

    // By path (sanitized)
    const sanitizedPath = this.sanitizePath(path);
    metrics.requests.byPath[sanitizedPath] = (metrics.requests.byPath[sanitizedPath] || 0) + 1;

    // Response time
    this.recordResponseTime(duration);
  }

  /**
   * Record response time for percentile calculation
   */
  recordResponseTime(duration) {
    const rt = this.metrics.http.responseTime;

    rt.sum += duration;
    rt.count++;
    rt.min = Math.min(rt.min, duration);
    rt.max = Math.max(rt.max, duration);

    // Store for percentile calculation (keep last 1000)
    rt.p50.push(duration);
    rt.p95.push(duration);
    rt.p99.push(duration);

    if (rt.p50.length > 1000) {
      rt.p50.shift();
      rt.p95.shift();
      rt.p99.shift();
    }
  }

  /**
   * Sanitize path to remove IDs
   */
  sanitizePath(path) {
    return path
      .replace(/\/[a-f0-9-]{8,}/gi, '/:id')
      .replace(/\/\d+/g, '/:id')
      .replace(/task-\d+/, ':id');
  }

  /**
   * Record task operation
   */
  recordTaskOperation(operation, task) {
    const metrics = this.metrics.tasks;

    switch (operation) {
      case 'create':
        metrics.created++;
        metrics.total++;
        if (task) {
          metrics.byStatus[task.status] = (metrics.byStatus[task.status] || 0) + 1;
          metrics.byPriority[task.priority] = (metrics.byPriority[task.priority] || 0) + 1;
        }
        break;
      case 'update':
        metrics.updated++;
        break;
      case 'delete':
        metrics.deleted++;
        metrics.total = Math.max(0, metrics.total - 1);
        break;
    }
  }

  /**
   * Record authentication event
   */
  recordAuthEvent(event, success = true) {
    const metrics = this.metrics.auth;

    switch (event) {
      case 'login':
        if (success) {
          metrics.logins.success++;
          metrics.activeTokens++;
        } else {
          metrics.logins.failed++;
        }
        break;
      case 'register':
        metrics.registrations++;
        break;
      case 'logout':
        metrics.activeTokens = Math.max(0, metrics.activeTokens - 1);
        break;
    }
  }

  /**
   * Record error
   */
  recordError(error, statusCode) {
    const metrics = this.metrics.errors;

    metrics.total++;
    metrics.byStatusCode[statusCode] = (metrics.byStatusCode[statusCode] || 0) + 1;

    const errorType = error.name || 'UnknownError';
    metrics.byType[errorType] = (metrics.byType[errorType] || 0) + 1;
  }

  /**
   * Calculate percentile
   */
  calculatePercentile(values, percentile) {
    if (values.length === 0) return 0;

    const sorted = [...values].sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * sorted.length) - 1;
    return sorted[Math.max(0, index)];
  }

  /**
   * Get all metrics
   */
  getMetrics() {
    const rt = this.metrics.http.responseTime;

    return {
      ...this.metrics,
      http: {
        ...this.metrics.http,
        responseTime: {
          avg: rt.count > 0 ? rt.sum / rt.count : 0,
          min: rt.min === Infinity ? 0 : rt.min,
          max: rt.max,
          p50: this.calculatePercentile(rt.p50, 50),
          p95: this.calculatePercentile(rt.p95, 95),
          p99: this.calculatePercentile(rt.p99, 99),
        },
      },
    };
  }

  /**
   * Get metrics in Prometheus format
   */
  getPrometheusMetrics() {
    const metrics = this.getMetrics();
    const lines = [];

    // HTTP metrics
    lines.push('# HELP http_requests_total Total number of HTTP requests');
    lines.push('# TYPE http_requests_total counter');
    lines.push(`http_requests_total ${metrics.http.requests.total}`);

    lines.push('# HELP http_requests_by_method HTTP requests by method');
    lines.push('# TYPE http_requests_by_method counter');
    Object.entries(metrics.http.requests.byMethod).forEach(([method, count]) => {
      lines.push(`http_requests_by_method{method="${method}"} ${count}`);
    });

    lines.push('# HELP http_response_time_ms HTTP response time in milliseconds');
    lines.push('# TYPE http_response_time_ms summary');
    lines.push(`http_response_time_ms{quantile="0.5"} ${metrics.http.responseTime.p50.toFixed(2)}`);
    lines.push(`http_response_time_ms{quantile="0.95"} ${metrics.http.responseTime.p95.toFixed(2)}`);
    lines.push(`http_response_time_ms{quantile="0.99"} ${metrics.http.responseTime.p99.toFixed(2)}`);

    // Task metrics
    lines.push('# HELP tasks_total Total number of tasks');
    lines.push('# TYPE tasks_total gauge');
    lines.push(`tasks_total ${metrics.tasks.total}`);

    // Error metrics
    lines.push('# HELP errors_total Total number of errors');
    lines.push('# TYPE errors_total counter');
    lines.push(`errors_total ${metrics.errors.total}`);

    // System metrics
    lines.push('# HELP system_uptime_seconds System uptime in seconds');
    lines.push('# TYPE system_uptime_seconds gauge');
    lines.push(`system_uptime_seconds ${(metrics.system.uptime / 1000).toFixed(2)}`);

    return lines.join('\n');
  }

  /**
   * Reset metrics (for testing)
   */
  reset() {
    clearInterval(this.uptimeInterval);
    this.metrics = {
      http: { requests: { total: 0, byMethod: {}, byStatus: {}, byPath: {} }, responseTime: { sum: 0, count: 0, min: Infinity, max: 0, p50: [], p95: [], p99: [] } },
      tasks: { total: 0, created: 0, updated: 0, deleted: 0, byStatus: {}, byPriority: {} },
      auth: { logins: { success: 0, failed: 0 }, registrations: 0, activeTokens: 0 },
      errors: { total: 0, byType: {}, byStatusCode: {} },
      system: { startTime: Date.now(), uptime: 0 },
    };
  }

  /**
   * Cleanup
   */
  destroy() {
    if (this.uptimeInterval) {
      clearInterval(this.uptimeInterval);
    }
  }
}

// Export singleton instance
const metricsCollector = new MetricsCollector();

module.exports = metricsCollector;
