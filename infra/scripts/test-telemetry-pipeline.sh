#\!/bin/bash

echo "Testing telemetry pipeline..."

# Test metrics via OTLP HTTP
echo "Sending test metrics..."
curl -X POST http://otel-http.local:31251/v1/metrics \
  -H "Content-Type: application/json" \
  -d '{
    "resourceMetrics": [{
      "resource": {
        "attributes": [{
          "key": "service.name",
          "value": {"stringValue": "claude-code-test"}
        }]
      },
      "scopeMetrics": [{
        "scope": {
          "name": "test.metrics"
        },
        "metrics": [{
          "name": "claude_code_sessions_total",
          "description": "Test session counter",
          "sum": {
            "dataPoints": [{
              "attributes": [{
                "key": "user_id",
                "value": {"stringValue": "test-user-1"}
              }],
              "startTimeUnixNano": "'"$(date -u +%s)000000000"'",
              "timeUnixNano": "'"$(date -u +%s)000000000"'",
              "asInt": "1"
            }],
            "aggregationTemporality": 2,
            "isMonotonic": true
          }
        }]
      }]
    }]
  }'

echo ""
echo "Sending test logs..."
curl -X POST http://otel-http.local:31251/v1/logs \
  -H "Content-Type: application/json" \
  -d '{
    "resourceLogs": [{
      "resource": {
        "attributes": [{
          "key": "service.name",
          "value": {"stringValue": "claude-code"}
        },{
          "key": "app",
          "value": {"stringValue": "claude-code"}
        }]
      },
      "scopeLogs": [{
        "scope": {
          "name": "test.logs"
        },
        "logRecords": [{
          "timeUnixNano": "'"$(date -u +%s)000000000"'",
          "severityNumber": 9,
          "severityText": "INFO",
          "body": {
            "stringValue": "{\"timestamp\":\"'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'\",\"level\":\"info\",\"message\":\"Test log from telemetry pipeline\",\"user_id\":\"test-user-1\",\"action\":\"test\"}"
          },
          "attributes": [{
            "key": "app",
            "value": {"stringValue": "claude-code"}
          }]
        }]
      }]
    }]
  }'

echo ""
echo "Test data sent successfully!"
echo ""
echo "You can now check:"
echo "1. Grafana dashboards at http://grafana.local:31251"
echo "2. Prometheus metrics at: http://grafana.local:31251/explore (select Prometheus datasource)"
echo "3. Loki logs at: http://grafana.local:31251/explore (select Loki datasource)"