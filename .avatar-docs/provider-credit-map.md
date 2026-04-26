# Morgan avatar provider and credit contingency map

This document tracks model/API/GPU providers that could help us reach the
primary goal: **maximize GLB quality and minimize manual touch-up** for the
Morgan avatar pipeline.

Scope is intentionally broad. It includes managed model APIs, 3D-specific
providers, hyperscalers, serverless GPU platforms, neoclouds, bare-metal GPU
providers, and startup-credit programs. The goal is to keep contingencies ready
so we do not stop/replan every time one model or provider fails.

## Current credit/relationship assumptions

| Provider | Current status | Confidence | Notes |
| --- | --- | --- | --- |
| Fireworks | Have some credits | User-reported + repo config | Useful for LLM/prompting/multimodal model access; no direct GLB generator identified yet. |
| Bastion | Applied for credits | User-reported | Exact provider/program needs verification; web search was inconclusive. |
| Google / Vertex / Gemini | Have access/credits | User-reported + repo config | High-value for Gemini image/reference prep, Imagen, Veo, and Vertex AI. |
| Microsoft / Azure | Have access | User-reported | Potentially useful for Azure ML/GPU and Azure Speech visemes; TRELLIS is Microsoft-origin open-source but not confirmed as hosted Azure model. |
| Alibaba Cloud | Have some credits/access | User-reported | Investigate DashScope/Qwen/Wan/Tongyi and GPU ECS/PAI. |
| DigitalOcean | Have some credits/access | User-reported + prior docs | Treat as ephemeral GPU VM, not true serverless GPU. |
| Scaleway | Likely eligible; user plans to reapply | User-reported + public startup program | EU cloud/GPU contingency; Startup Program advertises up to EUR36,000 in cloud credits, with an NVIDIA Inception member path advertising up to EUR42,000. |
| Baseten | Exact provider identified: <https://www.baseten.co/> | User-provided + public startup program | High-priority custom model deployment/training provider; startup program advertises up to $25,000 for Dedicated Inference or Training plus up to $2,500 for Model APIs. |
| Hugging Face | Have access/possibly credits | User-reported | Useful for Spaces, Inference Endpoints, and hosting open-source avatar models. |
| OVHcloud | Previously used; app removed | Prior docs | Do not relaunch without explicit re-approval. Useful only if credits remain and serverless options fail. |
| Scenario | Free account/MCP access | Verified via MCP | Use now for recommendations, schemas, hosted paid model trials, and workflows if credits/free tier allow. |

## Provider map

| Provider | Type | Known credit status | Startup program to apply? | GLB/DAG relevance |
| --- | --- | --- | --- | --- |
| Scenario | Managed model/workflow marketplace | Free account via MCP; paid status unknown | **Ask/request credits** | Direct access to Hunyuan 3D 3.1 Pro, Tripo, Rodin, Meshy, Kling, Veo/LTX-style video, Sync/Veed/HeyGen/Pixverse, Gemini workflows. High-priority for paid GLB-quality experiments. |
| Google Cloud / Vertex AI | Hyperscaler + model API | Have access/credits | **Apply/upgrade AI startup tier if not already** | Gemini 3.1 Flash for references/texture/source prep; Imagen/Veo for image/video; Vertex GPUs for self-hosting if needed. |
| Microsoft Azure | Hyperscaler + model/API platform | Have access | **Apply/confirm Founders Hub credits** | Azure GPU/ML; Azure Speech visemes for runtime; possible TRELLIS/Microsoft model ecosystem adjacency. |
| Alibaba Cloud / DashScope / Model Studio | Hyperscaler + model API | Have some credits/access | **Apply/confirm AI/startup credits** | Qwen for reasoning/prompting, Wan/Tongyi for image/video, GPU ECS/PAI for self-hosting; possible 3D/multiview offerings to investigate. |
| AWS / Bedrock / SageMaker | Hyperscaler | Possible credits in another account | **Apply/confirm AWS Activate** | GPU EC2/SageMaker and Bedrock. Not first choice for GLB quality unless credits are large and GPU quota is easy. |
| DigitalOcean / Paperspace / Gradient | GPU cloud + VM | Have some credits/access | **Confirm Startup Launchpad/Hatch status** | Ephemeral GPU VM runner for AniGen/Hunyuan3D/TRELLIS/UniRig; not true serverless but can create-run-destroy jobs. |
| Fireworks AI | Model API/inference | Have credits | **Apply/confirm startup credits** | Strong for hosted open model inference, prompt generation, visual reasoning if supported. No direct GLB generator yet. |
| Bastion | Unknown / user-applied | Applied | **Verify exact program** | Treat as pending contingency until exact product, GPU/model support, and credit terms are known. |
| Hugging Face | Model hub, Spaces, Inference Endpoints | Have access/possibly credits | **Ask via startup/partner channel if needed** | Key for AniGen, Hunyuan3D, TRELLIS, UniRig, SkinTokens, LivePortrait, RMBG/BiRefNet, and quick hosted demos. |
| RunPod | Serverless GPU + GPU cloud | Unknown | **Request startup credits/contact sales** | Best practical fit for true short-lived self-hosted GPU workers if DO is too VM-like. |
| Modal | Serverless GPU/platform | Unknown | **Apply startup program** | Strong developer workflow for GPU jobs, Python-first DAG nodes, fast iteration; good for AniGen/Hunyuan/TRELLIS workers. |
| Replicate | Managed model API/serverless GPU | Unknown | **Use free tier; ask for credits if usage grows** | Easy access to many image/video/3D models; good quick benchmarks without owning images. |
| fal.ai | Managed/serverless media models | Unknown | **Ask for startup/pilot credits** | Strong image/video/audio ecosystem; potentially useful for source prep, restoration, video fallback, maybe 3D if available. |
| Baseten | Model serving/serverless GPU | User identified exact provider; credits not applied/confirmed | **Apply to AI Startup Program** for up to $25,000 Dedicated Inference/Training credits plus up to $2,500 Model API credits; eligibility appears early-stage AI-first, Seed-Series A, under 5 years, net-new customer. | Host our own custom/fine-tuned GPU workers behind production-grade endpoints; strong fit for trained repair nodes, segmentation/texture models, and any reusable avatar DAG service. |
| Beam Cloud | Serverless GPU/ML apps | Unknown | **Apply/request credits** | Candidate for GPU DAG worker deployment; likely easier than raw VMs. |
| Cerebrium | Serverless GPU | Unknown | **Apply/request credits** | Candidate for GPU DAG worker deployment; evaluate cold starts and image limits. |
| Koyeb | Serverless apps + GPU | Unknown | **Apply/request credits** | Possible model serving backend, less proven for heavy 3D GPU jobs. |
| Northflank | App platform + GPU/serverless ops | Unknown | **Apply/request startup credits** | Potential orchestrator/worker platform; evaluate GPU support before using. |
| Railway | App platform / possible GPU waitlist | Unknown | **Apply/check startup credits and GPU availability** | Good app platform, but GPU/serverless fit is uncertain; not first for AniGen. |
| Render | App platform | Unknown | **Apply/check startup credits** | Good web services, not a primary GPU/3D path unless GPU support fits. |
| Fly.io | App platform | Unknown | **Apply/request credits** | Useful for edge services, not primary GLB/GPU path. |
| Vercel | App/platform | Unknown | **Apply/startup program if useful** | Frontend/runtime hosting, not GPU GLB generation. |
| CoreWeave | Neocloud GPU/Kubernetes | Unknown | **Pursue via sales/partner if large GPU need emerges** | Excellent for scale/training, likely overkill for first GLB benchmarks. |
| Lambda Labs | GPU cloud | Unknown | **Ask for startup/research credits** | Strong developer GPU instances; good self-host fallback. |
| Crusoe Cloud | Neocloud GPU | Unknown | **Ask for startup/enterprise credits** | Sustainable GPU cloud, useful if pricing/availability beats others. |
| Nebius AI Cloud | Neocloud GPU + AI platform | Unknown | **Apply/request credits** | Good candidate for European/AI GPU workloads; evaluate quotas and startup program. |
| Scaleway | Cloud/bare metal/GPU | User likely qualifies and plans to reapply | **Reapply to Startup Program**. Public tiers show Founders up to EUR1,000/year, go-to-market up to EUR1,500/month for 6 months (EUR9,000), Growth up to EUR3,000/month for 12 months (EUR36,000); NVIDIA Inception route advertises up to EUR42,000. | Strong EU GPU/cloud contingency for self-hosted AniGen/Hunyuan/TRELLIS/UniRig workers if RunPod/Modal/DO are blocked or if Scaleway credits are approved. |
| OVHcloud | Cloud/GPU AI Deploy | Previously used | **Use only if credits remain and explicitly re-approved** | Works, but long-running AI Deploy can burn cost; app is currently removed. |
| Vultr | Cloud/GPU | Unknown | **Apply partner/startup credits** | GPU instances and simpler cloud; contingency if DO/OVH fail. |
| Civo | Kubernetes/cloud | Unknown | **Apply/hackathon/startup credits** | Possible app/K8s hosting, not primary GPU path. |
| TensorDock | GPU marketplace/cloud | Unknown | **Ask for startup credits/discounts** | Cost-effective GPU jobs; evaluate reliability/security. |
| Thunder Compute | GPU cloud | Unknown | **Claim starter credits/matches** | Good for small immediate GPU tests if available; not primary production. |
| Vast.ai | GPU marketplace | Unknown | No formal startup program; cheap marketplace | Cheap self-hosted GPU fallback; higher ops/security variability. |
| Fluidstack | GPU/neocloud | Unknown | **Ask for startup/AI credits** | Useful if we need large GPU clusters or enterprise commitments. |
| Voltage Park | GPU/neocloud | Unknown | **Ask via sales/partners** | Large GPU capacity; overkill unless training/fine-tuning becomes heavy. |
| Latitude.sh / Hivelocity / bare metal partners | Bare metal/cloud | Existing partner docs for Hivelocity; credits unknown | **Explore partner credits** | Useful for persistent GPU/bare-metal if serverless cost is too high. |
| Together AI | Model API/fine-tuning | Unknown | **Apply startup grants/credits** | Hosted open models, possible fine-tuning; useful for prompt/vision/reasoning, not direct GLB. |
| Groq | Fast inference API | Unknown/free tier likely | **Claim developer/startup credits** | Fast LLM reasoning/prompting, not GLB generation. |
| Cerebras Cloud | Fast inference API | Unknown/free tier likely | **Claim developer/startup credits** | Fast LLM reasoning/prompting, not GLB generation. |
| SambaNova Cloud | Inference API/platform | Unknown | **Ask/pilot** | Hosted LLMs; not direct GLB. |
| DeepInfra | Inference API | Unknown/free tier likely | **Use free tier; ask if scaling** | Cheap hosted models; not direct GLB. |
| Lepton AI | Model serving/API | Unknown | **Ask/request credits** | Potential serving/fine-tuning platform; evaluate if custom workers needed. |
| Anyscale | Ray/cloud AI platform | Unknown | **Apply/request credits** | Useful for distributed jobs or model serving; not first GLB route. |
| OpenRouter | Model routing API | Unknown | Possibly not relevant | Useful for LLM routing, not GLB generation. |
| Stability AI | Model API/open models | Unknown | **Check credits/API access** | Image generation/upscale; TripoSR/open models adjacency, not primary GLB quality. |
| Black Forest Labs | Image model provider | Unknown | **Check partner/API credits** | Flux/Kontext image editing/reference prep; Scenario may wrap some capabilities. |
| Luma AI | Image/video/3D-ish creative API | Unknown | **Check startup/API access** | Potential video/scene fallback; no direct runtime GLB path confirmed. |
| Runway | Video model provider | Unknown | **Check startup/education/partner credits** | Hero/fallback video only. |
| Pika | Video model provider | Unknown | **Check startup/program credits** | Hero/fallback video only. |
| Kling / Kuaishou | Video/avatar/lip-sync | Unknown | **Use via Scenario first** | High-quality image-to-video/talking-video fallback. |
| PixVerse | Video/lip-sync | Unknown | **Use via Scenario first** | Stylized video fallback/redub. |
| HeyGen | Talking avatar SaaS | Unknown | **Check trial/startup options** | Quality benchmark or fallback for video, not GLB runtime. |
| Sync Labs | Lip-sync API | Unknown | **Check startup credits** | Video lip-sync repair/fallback, not GLB runtime controls. |
| VEED / Fabric | Video/lip-sync | Unknown | **Use via Scenario if needed** | Video lip-sync fallback. |
| Tripo AI | 3D generation/rigging/retopo | Unknown | **Ask for startup/API credits** | High-priority GLB quality and rigging route. |
| Meshy | 3D generation/API | Unknown | **Ask for credits/trial** | Text/image-to-3D fallback and quick concept volume. |
| Rodin / Hyper3D | 3D generation/API | Unknown | **Ask for credits/API access** | 3D fallback with T/A-pose and PBR options. |
| Tencent Hunyuan | 3D/image/video models | Unknown | **Use Scenario first; explore direct API later** | Hunyuan 3D quality path; direct credits may be harder than Scenario. |

## Discovery expansion: providers not originally named

This section captures additional providers found in the broader search pass. They
are not all top-priority, but they should stay visible until we know which stages
fail and which credits are easiest to activate.

| Provider | Type | Startup/free-credit signal | Avatar/GLB relevance |
| --- | --- | --- | --- |
| 3D AI Studio | Unified 3D generation API | Free/trial credits likely; paid credits API | Wraps/compares Hunyuan3D, Tripo, Meshy-style APIs. Useful as a one-key benchmark layer if direct provider accounts are slow. |
| Hypereal | Unified AI model/API marketplace | Advertises free credits/no minimums | Offers Tripo3D, Hunyuan3D, Meshy through a unified API. Good contingency for direct 3D API access. |
| Sloyd | 3D asset generator/API/SDK | Paid plans; API access/waitlist/status should be checked | Strong for clean, controllable, game-ready props and parametric assets; less certain for Morgan character fidelity, but useful for topology/control ideas. |
| Common Sense Machines / CSM | 3D generation and asset decomposition | Unknown | Potential image/video-to-3D and part decomposition provider; evaluate for separable parts and mesh segmentation if Morgan assets need surgery. |
| Kaedim | Production 3D asset generation service | Unknown; likely enterprise/sales-led | Human/AI hybrid production asset path. Could reduce manual touch-up but may be too service-like for automated DAG. |
| 3DFY.ai | Text-to-3D/API | Unknown | Low-poly/object generation; likely lower priority for Morgan character but useful reference for API shape and prop generation. |
| Masterpiece X | Text-to-3D creation tool | Unknown | Rapid prototyping path; evaluate export quality before any pipeline use. |
| PixCap | 3D design/asset platform | Unknown | Asset library/design workflow more than core avatar generation; possible ancillary asset source. |
| Spline AI | Browser 3D/web design | Unknown | Useful for lightweight web 3D concepts; not a primary production GLB generator. |
| Luma AI / Genie | 3D/video creative model provider | Trial/paid; startup credits unknown | Photorealistic 3D/capture-like outputs, often need retopology; useful visual baseline, not first rigging path. |
| Stability AI / Stable Fast 3D / TripoSR ecosystem | Open/commercial model API | Trial/API credits unknown | Fast open-source-ish mesh baseline; quality may lag Hunyuan/Tripo but useful for local fallback or cheap batch testing. |
| Novita AI | Serverless GPU/model API | Free/credit signal in provider lists | Candidate for serverless GPU and media model APIs; evaluate custom container support before self-hosted workers. |
| Inferless | Serverless GPU/model deployment | Free tier signal in GPU vendor directories | Candidate for custom model endpoints; evaluate image size, cold starts, and GPU choices. |
| Google Cloud Run with GPUs | Hyperscaler serverless GPU/container | Covered by GCP credits if available | More serverless-like than raw GCE. Candidate if custom CUDA worker images fit Cloud Run GPU constraints. |
| Azure Container Apps with GPUs | Hyperscaler serverless/container GPU | Covered by Azure credits if available | Candidate if Azure GPU container support is available in our regions/account. |
| Clarifai | AI platform/model hosting/catalog | Startup/partner signal unknown | Broad model deployment/catalog platform; possible wrapper for custom inference and model catalog search. |
| NVIDIA Inception | Startup program/ecosystem | Free startup program, not direct cloud credits | Apply/join if not already; useful for partner credits, GPU vendor intros, and technical support. |
| Genesis Cloud | GPU cloud | Startup/credit status unknown | EU/GDPR GPU fallback; appears in GPU price comparisons. |
| DataCrunch / Verda | GPU cloud | Startup/credit status unknown | Cost-effective GPU cloud fallback for self-hosted workers. |
| Jarvis Labs | GPU cloud | Unknown | Simple GPU notebook/instance fallback for manual and scripted experiments. |
| GMI Cloud | GPU/neocloud | Startup/sales credit status unknown | Specialized GPU provider with H100/H200/Blackwell positioning; evaluate if larger training becomes necessary. |
| Hyperstack / NexGen Cloud | GPU cloud | Startup/credit status unknown | GPU cloud candidate; evaluate quotas and regions. |
| Sesterce Cloud | GPU cloud | Startup/credit status unknown | Listed in GPU provider directories; evaluate only if top-tier providers block. |
| Shadeform | GPU cloud aggregator | Unknown | Aggregates provider capacity; useful if we need multi-provider GPU access quickly. |
| Valdi | Sustainable GPU cloud | Unknown | Lower-priority GPU fallback. |
| Cudo Compute | GPU cloud | Unknown | Lower-priority GPU fallback. |
| Massed Compute | Distributed GPU cloud | Unknown | Lower-priority GPU fallback; evaluate reliability before serious use. |
| SaladCloud | Distributed/community GPU | Low-cost/free trial unknown | Very cheap but reliability/security risk; only for non-sensitive experiments. |
| Akash Network | Decentralized GPU marketplace | No startup-credit dependency | Low-cost decentralized GPU option; operational friction and security posture must be evaluated. |
| Fluence | Decentralized/cloud GPU | Unknown | Possible compute marketplace; low priority until conventional providers fail. |
| Oracle Cloud | Hyperscaler GPU | Startup credits possible | Sometimes generous credits; GPU availability varies. Useful if existing account credits exist. |

## GLB-quality provider shortlist

| Use case | Best provider options | Why |
| --- | --- | --- |
| Fastest paid quality test | Scenario Hunyuan 3D 3.1 Pro, Tripo 3.1/P1 | Avoids custom GPU setup and gives us a quality ceiling quickly. |
| Open-source controllable asset | AniGen on RunPod/Modal/DO/HF | Best current skinned/animatable GLB-like candidate. |
| Mesh repair | Tripo Retopology, Blender worker, InstantMesh/TRELLIS experiments | Reduces manual Blender cleanup if raw mesh is too dense or malformed. |
| Rigging repair | Tripo Rigging 1.0, UniRig, Blender/Faceit | Converts good static mesh into runtime candidate. |
| Face/mouth controls | ARKit/Oculus/VRM transfer tooling, Blender/Faceit, template head route | Main edge-case gap because Morgan is canine, not human. |
| Source/reference quality | Gemini 3.1 Flash, RMBG/BiRefNet, SAM2, Flux/Kontext | Better references lower downstream manual cleanup. |
| Hero/fallback video | Kling, Veo, LTX, LivePortrait/Hallo/EchoMimic | Useful demos but not a substitute for GLB runtime. |

## Training/fine-tuning contingency

Training or fine-tuning becomes worth considering if paid/off-the-shelf models
consistently fail on Morgan because he is a canine executive avatar rather than
a human.

| Contingency | When to trigger | Candidate providers/platforms |
| --- | --- | --- |
| Train reference/image style LoRA | Morgan identity/style drifts across refs or generated views. | Scenario custom models, Replicate/fal/Baseten/Modal, Fireworks/Together for supporting LLM/prompt tasks, Google/Vertex if image tuning is available. |
| Fine-tune 3D generator or adapter | Hunyuan/Tripo/AniGen produce good humans but poor canine facial/head topology. | Hugging Face self-host, RunPod/Modal/CoreWeave/Lambda/Nebius for training, VAST/TRELLIS/Hunyuan open-source stacks. |
| Train segmentation/background model | Fur/ears/muzzle keep getting clipped and alpha artifacts poison 3D generation. | RMBG/BiRefNet fine-tune on HF/Modal/RunPod/DO; SAM2 annotation loop. |
| Build a template-head transfer pipeline | Good mesh exists but face controls fail repeatedly. | Blender/Faceit, ARKit/Oculus/VRM template assets, custom morph transfer worker. |
| Train/evaluate canine-specific expression controls | Human ARKit mouth shapes do not map well to Morgan muzzle. | Blender-generated synthetic data + custom morph targets; GPU training on Modal/RunPod/CoreWeave/Lambda/Google. |

## Application priority

| Priority | Apply / verify | Reason |
| --- | --- | --- |
| P0 | Scenario credits / paid-plan trial | Directly unlocks Hunyuan 3D 3.1 Pro, Tripo, rigging, retopo, and video fallback without infra. |
| P0 | Tripo API/startup credits | Direct GLB/rig/retopo provider; could reduce manual Blender work. |
| P0 | Google AI startup tier / Vertex credits | Gemini source prep and possible Veo/Imagen; already high leverage. |
| P0 | RunPod or Modal startup credits | Best fit for short-lived self-hosted AniGen/Hunyuan/TRELLIS/UniRig workers. |
| P1 | DigitalOcean Launchpad/Hatch status | Useful credits, but VM-like GPU lifecycle means more orchestration. |
| P1 | Alibaba Cloud AI/DashScope credits | Potential Qwen/Wan/Tongyi + GPU ECS/PAI fallback. |
| P1 | Hugging Face credits/Inference Endpoint access | Easiest place to test OSS model spaces/endpoints. |
| P1 | Fireworks startup credits expansion | Useful for LLM/vision/prompting support and general agent/model tasks. |
| P1 | Scaleway reapplication | User likely qualifies; possible EUR36,000-EUR42,000 cloud credit path makes this a serious EU GPU contingency. |
| P1 | Baseten AI Startup Program | High-value if we need to deploy custom/fine-tuned avatar DAG nodes as production APIs. |
| P1 | OVH / Vultr / Lambda / Nebius | GPU capacity contingencies if RunPod/Modal/DO/Scaleway are blocked. |
| P2 | Railway / Render / Fly / Vercel | App/runtime hosting only unless GPU support clearly fits. |

## Operating rules

- Keep provider choices DAG-node-specific. Do not pick one provider for the
  whole avatar pipeline.
- Use hosted paid models to discover the quality ceiling before investing in
  self-hosting optimizations.
- Prefer open-source/self-hosted once we know which stage needs repeated runs.
- Do not keep GPUs running while waiting for manual review.
- Store every provider run in the same artifact layout so outputs are comparable:
  `input/`, `raw/`, `normalized/`, `renders/`, `validation/`, `report.json`,
  and `report.md`.
- Before running any paid provider, record model name, provider, schema/settings,
  cost estimate when available, input artifact IDs, and expected output format.
