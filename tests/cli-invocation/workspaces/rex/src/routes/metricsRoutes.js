/**
 * Metrics and Observability Routes
 */

const express = require('express');
const metricsCollector = require('../observability/metrics');
const { asyncHandler } = require('../middleware/errorHandler');

const router = express.Router();

/**
 * @route   GET /metrics
 * @desc    Get application metrics in JSON format
 * @access  Public (in production, should be protected)
 */
router.get('/metrics', asyncHandler(async (req, res) => {
  const metrics = metricsCollector.getMetrics();

  res.status(200).json({
    status: 'success',
    data: {
      metrics,
    },
  });
}));

/**
 * @route   GET /metrics/prometheus
 * @desc    Get metrics in Prometheus format
 * @access  Public (in production, should be protected)
 */
router.get('/metrics/prometheus', asyncHandler(async (req, res) => {
  const prometheusMetrics = metricsCollector.getPrometheusMetrics();

  res.set('Content-Type', 'text/plain; version=0.0.4');
  res.status(200).send(prometheusMetrics);
}));

module.exports = router;
