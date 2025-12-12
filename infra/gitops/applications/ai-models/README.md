# AI Models

This directory contains ArgoCD applications for deploying AI/ML models on Kubernetes.

## Three Options for Model Deployment

| Feature | KubeAI | Ollama Operator | LlamaStack |
|---------|--------|-----------------|------------|
| **Backend** | vLLM (GPU optimized) | Ollama (easy setup) | Multi-backend (vLLM, Ollama, Bedrock, TGI) |
| **Best for** | Production GPU inference | CPU/mixed workloads | Full AI stack with safety/tooling |
| **Scale from zero** | ✅ Yes | ❌ No | ❌ No |
| **OpenAI API** | ✅ Native | ✅ Compatible | ✅ Compatible |
| **Model format** | HuggingFace, Ollama | Ollama library | Multiple providers |
| **Safety/Guardrails** | ❌ No | ❌ No | ✅ Built-in |
| **Tool calling** | Basic | Basic | ✅ Advanced |
| **License** | Apache 2.0 | Apache 2.0 | MIT |
| **Maintainer** | Substratusai | Community | Meta |

## Quick Start

### Prerequisites

Choose your operator(s):

- **KubeAI** (`operators/kubeai.yaml`) - For vLLM-based GPU inference with scale-from-zero
- **Ollama Operator** (`operators/ollama-operator.yaml`) - For native Ollama models (simple CRD)
- **LlamaStack Operator** (`operators/llamastack-operator.yaml`) - Meta's full AI stack with multi-backend support
- **NVIDIA GPU Operator** (`operators/nvidia-gpu-operator.yaml`) - For GPU workloads

### 2. Enable Models (for KubeAI)

Edit `kubeai-models.yaml` and set `enabled: true` for the models you want:

```yaml
catalog:
  llama-3.1-8b-instruct-fp8-l4:
    enabled: true  # Enable this model
    minReplicas: 1  # Keep 1 replica always running (optional)
```

### 3. Access KubeAI Models

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

### 4. Access Ollama Operator Models

```bash
# Port-forward to Ollama model service
kubectl port-forward svc/ollama-model-phi4 -n ollama-operator-system 11434:11434

# Use Ollama CLI
ollama run phi4

# Or use OpenAI-compatible API
curl http://localhost:11434/v1/chat/completions \
  -d '{"model": "phi4", "messages": [{"role": "user", "content": "Hello!"}]}'
```

### 5. Access LlamaStack

```bash
# Port-forward to LlamaStack service
kubectl port-forward svc/llamastack-starter -n llama-stack 8321:8321

# List models
curl http://localhost:8321/v1/models

# Chat with safety guardrails
curl http://localhost:8321/v1/chat/completions \
  -d '{"model": "llama3.2:3b", "messages": [{"role": "user", "content": "Hello!"}]}'
```

## Available Models

### Text Generation (LLMs)

| Model | GPU | VRAM | Best For |
|-------|-----|------|----------|
| `llama-3.1-8b-instruct-fp8-l4` | L4 x1 | 24GB | General tasks, coding |
| `llama-3.1-70b-instruct-fp8-h100` | H100 x2 | 160GB | Complex reasoning |
| `glm-4.5-air-fp8-h100` | H100 x2 | 160GB | **Agentic tasks, tool calling** |
| `deepseek-r1-1.5b-cpu` | CPU | 4GB | Light reasoning (always on) |
| `qwen2.5-7b-instruct-l4` | L4 x1 | 24GB | Multilingual, coding |

### Embeddings

| Model | GPU | Best For |
|-------|-----|----------|
| `nomic-embed-text-cpu` | CPU | General text embeddings |

## Model Licenses

⚠️ **Important**: While KubeAI is Apache 2.0, individual models have their own licenses:

| Model Family | License | Commercial Use |
|--------------|---------|----------------|
| Llama 3.x | Meta Llama License | ✅ Yes (with conditions) |
| **GLM-4.5** | **MIT** | ✅ **Yes (fully permissive)** |
| Qwen | Apache 2.0 | ✅ Yes |
| Gemma | Gemma Terms | ✅ Yes (with conditions) |
| Mistral | Apache 2.0 | ✅ Yes |
| DeepSeek | MIT | ✅ Yes |
| BGE | MIT | ✅ Yes |
| Nomic | Apache 2.0 | ✅ Yes |

### GLM-4.5 - Recommended for Agentic Tasks

[GLM-4.5](https://z.ai/blog/glm-4.5) by Zhipu AI is particularly noteworthy:
- **MIT License** - Fully permissive, no restrictions
- **Hybrid Reasoning** - Thinking mode for complex tasks, non-thinking for fast responses
- **Top 3 on Agent Benchmarks** - Excellent for agentic/tool-calling use cases
- **128K Context** - Large context window

**Deployment Options:**

| Method | Model | Hardware |
|--------|-------|----------|
| KubeAI (vLLM) | `glm-4.5-air-fp8-h100` | H100 x 2 |
| Ollama Operator | `MichelRosselli/GLM-4.5-Air:Q4_K_M` | 80GB+ VRAM |
| LlamaStack | `llamastack-glm45` | H100 x 2 |

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
