# AI Services Ingress

This directory contains ingress configurations for AI services.

## Quick Start

1. **Update hostname** in each ingress file:
   ```yaml
   host: ai.your-domain.com  # Replace with your domain
   ```

2. **Configure DNS** to point your domain to the ingress controller's external IP

3. **Add TLS** (recommended for production):
   ```yaml
   spec:
     tls:
       - hosts:
           - ai.your-domain.com
         secretName: ai-tls-secret
   ```

## Access Patterns

| Service | Path | Port-Forward Alternative |
|---------|------|-------------------------|
| KubeAI | `/kubeai/*` | `kubectl port-forward svc/kubeai -n kubeai 8000:80` |
| Ollama | `/ollama/*` | `kubectl port-forward svc/ollama-model-phi4 -n ollama-operator-system 11434:11434` |
| LlamaStack | `/llamastack/*` | `kubectl port-forward svc/llamastack-starter -n llama-stack 8321:8321` |

## NGINX Configuration

All ingresses include these optimizations for AI workloads:

- `proxy-body-size: 100m` - Large request bodies
- `proxy-read-timeout: 600` - Long inference times
- `proxy-send-timeout: 600` - Streaming responses
- `proxy-buffering: off` - Real-time streaming
