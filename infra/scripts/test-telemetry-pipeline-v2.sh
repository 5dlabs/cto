#!/bin/bash

echo "=== Testing Claude Code Telemetry Pipeline ==="
echo ""

# 1. Check Claude Code pod status
echo "1. Claude Code Pod Status:"
kubectl get pods -n claude-code -o wide
echo ""

# 2. Check Claude Code telemetry configuration
echo "2. Claude Code Telemetry Configuration:"
kubectl exec -n claude-code deploy/claude-code -- env | grep -E "(OTEL_|CLAUDE_CODE_ENABLE)" | sort
echo ""

# 3. Check OTLP Collector status
echo "3. OTLP Collector Status:"
kubectl get pods -n observability -l app.kubernetes.io/name=opentelemetry-collector
echo ""

# 4. Test OTLP collector metrics endpoint
echo "4. Testing OTLP Collector Prometheus Endpoint:"
kubectl port-forward -n observability svc/otel-collector-opentelemetry-collector 8889:8889 > /dev/null 2>&1 &
PF_PID=$!
sleep 3
echo "Checking for any metrics..."
curl -s http://localhost:8889/metrics | grep -E "^(claude_code|# TYPE)" | head -20
kill $PF_PID 2>/dev/null
echo ""

# 5. Check Prometheus remote write ingestion
echo "5. Checking Prometheus Status:"
kubectl logs -n observability deploy/prometheus-server --tail=20 | grep -i "remote" || echo "No remote write logs found"
echo ""

# 6. Query Prometheus for metrics
echo "6. Querying Prometheus for Claude Code metrics:"
kubectl port-forward -n observability svc/prometheus-server 9090:9090 > /dev/null 2>&1 &
PF_PID=$!
sleep 3
curl -s "http://localhost:9090/api/v1/label/__name__/values" | jq -r '.data[]' | grep -E "claude_code" || echo "No claude_code metrics found"
kill $PF_PID 2>/dev/null
echo ""

# 7. Check Loki log ingestion
echo "7. Checking Loki Status:"
kubectl get pods -n observability -l app.kubernetes.io/name=loki
echo ""

# 8. Check OTLP collector logs for errors
echo "8. OTLP Collector Recent Logs:"
kubectl logs -n observability deploy/otel-collector-opentelemetry-collector --tail=10
echo ""

echo "=== Summary ==="
echo "The telemetry pipeline should flow as:"
echo "Claude Code -> OTLP Collector (HTTP) -> Prometheus (Remote Write)"
echo "Claude Code -> OTLP Collector (HTTP) -> Loki (OTLP)"
echo "Additionally, Prometheus scrapes OTLP Collector's metrics endpoint"
