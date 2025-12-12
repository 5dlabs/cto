# AI Models for KubeAI

This directory contains ArgoCD applications for deploying AI/ML models via KubeAI.

## Quick Start

### 1. Ensure Prerequisites

- **KubeAI Operator** must be deployed (`operators/kubeai.yaml`)
- **NVIDIA GPU Operator** should be deployed for GPU workloads (`operators/nvidia-gpu-operator.yaml`)

### 2. Enable Models

Edit `kubeai-models.yaml` and set `enabled: true` for the models you want:

```yaml
catalog:
  llama-3.1-8b-instruct-fp8-l4:
    enabled: true  # Enable this model
    minReplicas: 1  # Keep 1 replica always running (optional)
```

### 3. Access Models

Models expose an **OpenAI-compatible API**:

```bash
# Port-forward to KubeAI service
kubectl port-forward svc/kubeai -n kubeai 8000:80

# List available models
curl http://localhost:8000/openai/v1/models

# Chat completion
curl http://localhost:8000/openai/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-3.1-8b-instruct-fp8-l4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### 4. Use with OpenAI SDK

```python
from openai import OpenAI

client = OpenAI(
    api_key="ignored",  # KubeAI doesn't require auth by default
    base_url="http://kubeai.kubeai.svc/openai/v1"  # In-cluster URL
)

response = client.chat.completions.create(
    model="llama-3.1-8b-instruct-fp8-l4",
    messages=[{"role": "user", "content": "Explain Kubernetes in one sentence"}]
)
print(response.choices[0].message.content)
```

## Available Models

### Text Generation (LLMs)

| Model | GPU | VRAM | Best For |
|-------|-----|------|----------|
| `llama-3.1-8b-instruct-fp8-l4` | L4 x1 | 24GB | General tasks, coding |
| `llama-3.1-70b-instruct-fp8-h100` | H100 x2 | 160GB | Complex reasoning |
| `deepseek-r1-1.5b-cpu` | CPU | 4GB | Light reasoning (always on) |
| `deepseek-r1-distill-llama-8b-l4` | L4 x1 | 24GB | Reasoning tasks |
| `qwen2.5-7b-instruct-l4` | L4 x1 | 24GB | Multilingual, coding |
| `qwen2.5-coder-1.5b-cpu` | CPU | 4GB | Code assistance |
| `mistral-small-24b-instruct-h100` | H100 x1 | 80GB | High quality output |

### Embeddings

| Model | GPU | Best For |
|-------|-----|----------|
| `nomic-embed-text-cpu` | CPU | General text embeddings |
| `bge-embed-text-cpu` | CPU | Fast embeddings |

### Reranking (for RAG)

| Model | GPU | Best For |
|-------|-----|----------|
| `bge-rerank-base-cpu` | CPU | Search result reranking |

### Speech-to-Text

| Model | GPU | Best For |
|-------|-----|----------|
| `faster-whisper-medium-en-cpu` | CPU | English transcription |

### Vision (Multimodal)

| Model | GPU | Best For |
|-------|-----|----------|
| `llama-3.2-11b-vision-instruct-l4` | L4 x1 | Image understanding |

## GPU Resource Profiles

KubeAI uses resource profiles to configure GPU allocation:

| Profile | GPUs | Memory | Use Case |
|---------|------|--------|----------|
| `cpu:N` | 0 | N cores | Small models, embeddings |
| `nvidia-gpu-l4:N` | N x L4 | N x 24GB | Cost-effective inference |
| `nvidia-gpu-h100:N` | N x H100 | N x 80GB | High performance |
| `nvidia-gpu-a100-80gb:N` | N x A100 | N x 80GB | Training & inference |
| `nvidia-gpu-gh200:N` | N x GH200 | N x 96GB | Large models |

## Scaling Behavior

- **minReplicas: 0** - Model scales to zero when idle (saves resources)
- **minReplicas: 1** - Always keep one replica running (faster first response)
- **maxReplicas: N** - Maximum concurrent instances
- **targetRequests: N** - Requests per replica before scaling up

## Adding Custom Models

You can add any Hugging Face or Ollama model:

```yaml
catalog:
  my-custom-model:
    enabled: true
    features: [TextGeneration]
    url: "hf://organization/model-name"  # Hugging Face
    # OR
    url: "ollama://model:tag"            # Ollama
    engine: VLLM  # or OLlama, Infinity, FasterWhisper
    resourceProfile: nvidia-gpu-l4:1
    minReplicas: 0
    args:
      - --max-model-len=8192
      - --gpu-memory-utilization=0.9
```

## Model Licenses

⚠️ **Important**: While KubeAI is Apache 2.0, individual models have their own licenses:

| Model Family | License | Commercial Use |
|--------------|---------|----------------|
| Llama 3.x | Meta Llama License | ✅ Yes (with conditions) |
| Qwen | Apache 2.0 | ✅ Yes |
| Gemma | Gemma Terms | ✅ Yes (with conditions) |
| Mistral | Apache 2.0 | ✅ Yes |
| DeepSeek | MIT | ✅ Yes |
| BGE | MIT | ✅ Yes |
| Nomic | Apache 2.0 | ✅ Yes |

Always verify the license for your specific use case at the model's Hugging Face page.

## Monitoring

KubeAI exposes Prometheus metrics. Models show up with labels for:
- Request latency (TTFT, TBT)
- Token throughput
- Queue depth
- Replica count

## Troubleshooting

### Model not starting

```bash
# Check Model CRD status
kubectl get models -n kubeai

# Check pod logs
kubectl logs -n kubeai -l app.kubernetes.io/name=kubeai-model-<name>
```

### Out of memory (OOM)

Reduce `--max-model-len` or `--gpu-memory-utilization` in the model args.

### Slow first response

Set `minReplicas: 1` to keep at least one replica always running.
