/**
 * Request Logging Middleware
 * Logs HTTP requests with relevant information and records metrics
 */

const metricsCollector = require('../observability/metrics');

const logger = (req, res, next) => {
  const start = Date.now();

  res.on('finish', () => {
    const duration = Date.now() - start;
    const log = {
      timestamp: new Date().toISOString(),
      method: req.method,
      url: req.originalUrl,
      status: res.statusCode,
      duration: `${duration}ms`,
      ip: req.ip || req.connection.remoteAddress,
      userAgent: req.get('user-agent') || 'unknown',
    };

    const logLevel = res.statusCode >= 500 ? 'ERROR' : res.statusCode >= 400 ? 'WARN' : 'INFO';
    console.log(`[${logLevel}]`, JSON.stringify(log));

    // Record metrics
    metricsCollector.recordHttpRequest(
      req.method,
      req.originalUrl,
      res.statusCode,
      duration
    );
  });

  next();
};

module.exports = logger;
