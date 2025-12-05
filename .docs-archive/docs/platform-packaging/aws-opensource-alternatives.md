# AWS Services → Open Source Alternatives Matrix

## Purpose

This document maps AWS services to their open source alternatives for the CTO Platform 
Service Marketplace. Each alternative is evaluated for self-hosting viability, license 
compatibility, and production readiness.

**Legend:**
- ✅ **Recommended** - Production-ready, actively maintained, good license
- 🟡 **Viable** - Works well but may have limitations or complexity
- 🟠 **Partial** - Covers some functionality, not a complete replacement
- ❌ **No Good Alternative** - Build custom or use managed service
- 📦 **Marketplace Priority** - High priority for CTO Platform marketplace

---

## Compute

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **EC2** | Proxmox VE | AGPL-3.0 | Full virtualization platform | ✅ |
| | OpenStack Nova | Apache 2.0 | Enterprise-grade, complex | 🟡 |
| | Harvester | Apache 2.0 | K8s-native HCI | ✅ 📦 |
| **EC2 Auto Scaling** | Kubernetes HPA/VPA | Apache 2.0 | Native K8s scaling | ✅ 📦 |
| | KEDA | Apache 2.0 | Event-driven autoscaling | ✅ 📦 |
| **Lambda** | OpenFaaS | MIT | Functions as a Service | ✅ 📦 |
| | Knative | Apache 2.0 | Serverless on K8s | ✅ 📦 |
| | Fission | Apache 2.0 | Fast cold starts | 🟡 |
| | Kubeless | Apache 2.0 | Native K8s serverless | 🟡 |
| | OpenWhisk | Apache 2.0 | IBM-backed, enterprise | 🟡 |
| **Elastic Beanstalk** | Dokku | MIT | Heroku-like PaaS | ✅ 📦 |
| | CapRover | Apache 2.0 | PaaS with UI | ✅ |
| | Coolify | Apache 2.0 | Modern Heroku alternative | ✅ 📦 |
| **App Runner** | Knative Serving | Apache 2.0 | Container auto-deploy | ✅ |
| **Batch** | Kubernetes Jobs | Apache 2.0 | Native batch processing | ✅ |
| | Argo Workflows | Apache 2.0 | Complex batch workflows | ✅ 📦 |
| | Apache Airflow | Apache 2.0 | Workflow orchestration | ✅ 📦 |
| **Lightsail** | Coolify | Apache 2.0 | Simple app hosting | ✅ |
| **Compute Optimizer** | Goldilocks | Apache 2.0 | K8s resource recommendations | ✅ |

---

## Containers

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **ECS** | Kubernetes | Apache 2.0 | Industry standard | ✅ 📦 |
| | Docker Swarm | Apache 2.0 | Simpler, less features | 🟡 |
| | Nomad | MPL 2.0 | HashiCorp, lighter weight | ✅ |
| **EKS** | K3s | Apache 2.0 | Lightweight K8s | ✅ 📦 |
| | Talos Linux | MPL 2.0 | Immutable K8s OS | ✅ 📦 |
| | RKE2 | Apache 2.0 | Rancher K8s | ✅ |
| | MicroK8s | Apache 2.0 | Canonical's mini K8s | 🟡 |
| **Fargate** | Virtual Kubelet | Apache 2.0 | Serverless-like pods | 🟡 |
| **ECR** | Harbor | Apache 2.0 | Enterprise container registry | ✅ 📦 |
| | Zot | Apache 2.0 | OCI-native registry | ✅ |
| | Distribution | Apache 2.0 | Docker's own registry | 🟡 |
| | Dragonfly | Apache 2.0 | P2P image distribution | ✅ |
| **App2Container** | Kompose | Apache 2.0 | Docker Compose to K8s | 🟡 |

---

## Storage

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **S3** | MinIO | AGPL-3.0 | S3-compatible, production-ready | ✅ 📦 |
| | SeaweedFS | Apache 2.0 | Distributed storage | ✅ 📦 |
| | Ceph (RGW) | LGPL 2.1 | S3 API via RADOS Gateway | ✅ |
| | Garage | AGPL-3.0 | Lightweight S3 | 🟡 |
| **S3 Glacier** | MinIO Tiering | AGPL-3.0 | Lifecycle policies | 🟡 |
| **EBS** | Longhorn | Apache 2.0 | K8s block storage | ✅ 📦 |
| | OpenEBS | Apache 2.0 | Container-native storage | ✅ |
| | Rook-Ceph | Apache 2.0 | Ceph on K8s | ✅ |
| | Piraeus/LINSTOR | GPL 3.0 | DRBD-based storage | 🟡 |
| **EFS** | NFS Server | Various | Simple shared storage | ✅ |
| | GlusterFS | GPL 3.0 | Distributed file system | 🟡 |
| | CephFS | LGPL 2.1 | POSIX-compliant | ✅ |
| | JuiceFS | Apache 2.0 | Cloud-native file system | ✅ |
| **FSx for Lustre** | Lustre | GPL 2.0 | HPC parallel file system | 🟡 |
| **FSx for Windows** | Samba | GPL 3.0 | SMB/CIFS | ✅ |
| **Storage Gateway** | Rclone | MIT | Cloud sync tool | ✅ |
| **Backup** | Velero | Apache 2.0 | K8s backup/restore | ✅ 📦 |
| | Restic | BSD-2 | Deduplicating backup | ✅ |
| | BorgBackup | BSD-3 | Encrypted backup | ✅ |
| | Barman | GPL 3.0 | PostgreSQL backup | ✅ |
| **Disaster Recovery** | Velero | Apache 2.0 | Cross-cluster restore | ✅ |
| | Kasten K10 | Proprietary | K8s DR (free tier) | 🟠 |

---

## Databases

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **RDS (MySQL)** | MySQL | GPL 2.0 | Direct equivalent | ✅ |
| | MariaDB | GPL 2.0 | MySQL fork | ✅ |
| | Vitess | Apache 2.0 | MySQL clustering | ✅ 📦 |
| **RDS (PostgreSQL)** | PostgreSQL | PostgreSQL | The gold standard | ✅ 📦 |
| | CloudNativePG | Apache 2.0 | K8s operator | ✅ 📦 |
| | Patroni | MIT | HA PostgreSQL | ✅ |
| | Crunchy PGO | Apache 2.0 | K8s operator | ✅ |
| **Aurora** | CockroachDB | BSL/CCL | Distributed SQL | 🟡 |
| | YugabyteDB | Apache 2.0 | Distributed PostgreSQL | ✅ 📦 |
| | TiDB | Apache 2.0 | MySQL-compatible | ✅ |
| **DynamoDB** | ScyllaDB | AGPL-3.0 | DynamoDB-compatible | ✅ 📦 |
| | Cassandra | Apache 2.0 | Wide-column store | ✅ |
| | FoundationDB | Apache 2.0 | Apple-backed | 🟡 |
| **ElastiCache (Redis)** | Redis | BSD-3 | In-memory cache | ✅ 📦 |
| | KeyDB | BSD-3 | Multi-threaded Redis | ✅ |
| | Dragonfly | BSL | Redis-compatible | 🟡 |
| | Valkey | BSD-3 | Redis fork (Linux Foundation) | ✅ 📦 |
| **ElastiCache (Memcached)** | Memcached | BSD-3 | Original caching | ✅ |
| **MemoryDB** | Redis + AOF | BSD-3 | Durable Redis | ✅ |
| **DocumentDB** | MongoDB | SSPL | Document database | 🟡 |
| | FerretDB | Apache 2.0 | MongoDB-compatible on PG | ✅ 📦 |
| **Keyspaces** | Apache Cassandra | Apache 2.0 | Wide-column store | ✅ |
| | ScyllaDB | AGPL-3.0 | Cassandra-compatible | ✅ |
| **Neptune** | Neo4j | GPL 3.0 | Graph database | 🟡 |
| | JanusGraph | Apache 2.0 | Distributed graph | 🟡 |
| | Dgraph | Apache 2.0 | Native GraphQL | ✅ |
| | Apache AGE | Apache 2.0 | Graph on PostgreSQL | ✅ |
| **Timestream** | TimescaleDB | Apache 2.0 | Time-series on PG | ✅ 📦 |
| | InfluxDB | MIT | Time-series | ✅ |
| | QuestDB | Apache 2.0 | Fast time-series | ✅ |
| | VictoriaMetrics | Apache 2.0 | Metrics & time-series | ✅ 📦 |
| **QLDB** | Hyperledger Fabric | Apache 2.0 | Blockchain ledger | 🟡 |
| | immudb | Apache 2.0 | Immutable database | ✅ |
| **Redshift** | ClickHouse | Apache 2.0 | Analytics database | ✅ 📦 |
| | Apache Druid | Apache 2.0 | Real-time analytics | ✅ |
| | DuckDB | MIT | Embedded analytics | ✅ |
| | StarRocks | Apache 2.0 | MPP analytics | ✅ |
| **RDS Proxy** | PgBouncer | ISC | PostgreSQL pooler | ✅ 📦 |
| | ProxySQL | GPL 3.0 | MySQL proxy | ✅ |

---

## Networking & Content Delivery

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **VPC** | Kubernetes CNI | Various | Calico, Cilium, Flannel | ✅ 📦 |
| | Open vSwitch | Apache 2.0 | Software switch | ✅ |
| **CloudFront** | Varnish | BSD-2 | HTTP accelerator | ✅ |
| | Nginx | BSD-2 | Reverse proxy/cache | ✅ 📦 |
| | Apache Traffic Server | Apache 2.0 | CDN caching | ✅ |
| | KeyCDN (self-host) | N/A | Need infrastructure | 🟠 |
| **Route 53** | CoreDNS | Apache 2.0 | K8s DNS | ✅ 📦 |
| | PowerDNS | GPL 2.0 | Authoritative DNS | ✅ |
| | BIND | MPL 2.0 | Classic DNS | ✅ |
| | dnsmasq | GPL 2.0 | Lightweight DNS | 🟡 |
| **Global Accelerator** | Cloudflare Tunnel | Proprietary | Free tier available | 🟠 |
| | HAProxy | GPL 2.0 | Load balancer | ✅ |
| **Direct Connect** | OpenVPN | GPL 2.0 | Site-to-site VPN | ✅ |
| | WireGuard | GPL 2.0 | Modern VPN | ✅ 📦 |
| **VPN** | WireGuard | GPL 2.0 | Fast, modern VPN | ✅ 📦 |
| | OpenVPN | GPL 2.0 | Proven VPN | ✅ |
| | Tailscale | BSD-3 | Mesh VPN (open client) | ✅ |
| | Headscale | BSD-3 | Self-hosted Tailscale | ✅ 📦 |
| **Transit Gateway** | Cilium | Apache 2.0 | Multi-cluster networking | ✅ |
| **PrivateLink** | Cilium Cluster Mesh | Apache 2.0 | Service connectivity | ✅ |
| **App Mesh** | Istio | Apache 2.0 | Service mesh | ✅ 📦 |
| | Linkerd | Apache 2.0 | Lightweight mesh | ✅ 📦 |
| | Cilium | Apache 2.0 | eBPF-based mesh | ✅ |
| **Cloud Map** | Consul | MPL 2.0 | Service discovery | ✅ 📦 |
| | etcd | Apache 2.0 | K8s service discovery | ✅ |
| **ALB** | Nginx Ingress | Apache 2.0 | K8s ingress | ✅ 📦 |
| | Traefik | MIT | Modern ingress | ✅ 📦 |
| | HAProxy Ingress | Apache 2.0 | High-performance | ✅ |
| | Envoy | Apache 2.0 | Edge proxy | ✅ |
| **NLB** | MetalLB | Apache 2.0 | Bare-metal LB | ✅ 📦 |
| | kube-vip | Apache 2.0 | K8s VIP | ✅ |
| **Gateway LB** | Cilium | Apache 2.0 | eBPF-based | ✅ |
| **Network Firewall** | Calico | Apache 2.0 | Network policies | ✅ 📦 |
| | Cilium | Apache 2.0 | eBPF firewall | ✅ |
| | OPNsense | BSD-2 | Firewall appliance | ✅ |
| | pfSense | Apache 2.0 | Firewall/router | ✅ |

---

## Security, Identity & Compliance

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **IAM** | Keycloak | Apache 2.0 | Identity & access mgmt | ✅ 📦 |
| | Zitadel | Apache 2.0 | Modern IAM | ✅ |
| | Authelia | Apache 2.0 | Auth proxy | ✅ |
| | Ory (Hydra/Kratos) | Apache 2.0 | Identity infrastructure | ✅ |
| **Cognito** | Keycloak | Apache 2.0 | User authentication | ✅ 📦 |
| | SuperTokens | Apache 2.0 | Auth for apps | ✅ |
| | Logto | MPL 2.0 | Modern auth | ✅ |
| | Authentik | MIT | IdP solution | ✅ |
| **Organizations** | Rancher | Apache 2.0 | Multi-cluster mgmt | ✅ |
| **Directory Service** | FreeIPA | GPL 3.0 | Identity mgmt | ✅ |
| | OpenLDAP | OpenLDAP | Directory service | ✅ |
| | Samba AD | GPL 3.0 | Active Directory | ✅ |
| **Secrets Manager** | HashiCorp Vault | BSL | Secrets management | 🟡 |
| | OpenBao | MPL 2.0 | Vault fork | ✅ 📦 |
| | Infisical | MIT | Secrets sync | ✅ |
| | External Secrets | Apache 2.0 | K8s secrets sync | ✅ 📦 |
| | Sealed Secrets | Apache 2.0 | Encrypted K8s secrets | ✅ |
| **KMS** | OpenBao Transit | MPL 2.0 | Encryption as service | ✅ |
| | age | BSD-3 | Modern encryption | ✅ |
| | SOPS | MPL 2.0 | Secrets encryption | ✅ 📦 |
| **CloudHSM** | SoftHSM | BSD-2 | Software HSM | 🟡 |
| **Certificate Manager** | cert-manager | Apache 2.0 | K8s cert automation | ✅ 📦 |
| | step-ca | Apache 2.0 | Private CA | ✅ |
| | Smallstep | Apache 2.0 | Certificate mgmt | ✅ |
| **GuardDuty** | Falco | Apache 2.0 | Runtime security | ✅ 📦 |
| | OSSEC | GPL 2.0 | HIDS | ✅ |
| | Wazuh | GPL 2.0 | Security platform | ✅ |
| **Inspector** | Trivy | Apache 2.0 | Vulnerability scanner | ✅ 📦 |
| | Grype | Apache 2.0 | Container scanning | ✅ |
| | Clair | Apache 2.0 | Container analysis | ✅ |
| **Detective** | OpenSearch + SIEM | Apache 2.0 | Security analytics | 🟡 |
| **Macie** | PII Scanner scripts | Various | Custom implementation | 🟠 |
| **Security Hub** | OWASP DefectDojo | BSD-3 | Security findings | ✅ |
| | Prowler | Apache 2.0 | Security assessments | ✅ |
| **Shield/WAF** | ModSecurity | Apache 2.0 | WAF engine | ✅ |
| | Coraza | Apache 2.0 | Modern WAF | ✅ |
| | NAXSI | GPL 3.0 | Nginx WAF | 🟡 |
| **Firewall Manager** | OPA Gatekeeper | Apache 2.0 | Policy enforcement | ✅ 📦 |
| | Kyverno | Apache 2.0 | K8s policies | ✅ 📦 |
| **Audit Manager** | OpenSCAP | LGPL 2.1 | Compliance scanning | ✅ |

---

## Machine Learning & AI

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **Bedrock** | Ollama | MIT | Local LLM runtime | ✅ 📦 |
| | vLLM | Apache 2.0 | LLM inference | ✅ 📦 |
| | LocalAI | MIT | OpenAI-compatible | ✅ |
| | llama.cpp | MIT | Efficient inference | ✅ |
| | text-generation-inference | Apache 2.0 | HuggingFace TGI | ✅ |
| **SageMaker** | Kubeflow | Apache 2.0 | ML platform on K8s | ✅ 📦 |
| | MLflow | Apache 2.0 | ML lifecycle | ✅ 📦 |
| | Seldon Core | Apache 2.0 | Model serving | ✅ |
| | BentoML | Apache 2.0 | Model deployment | ✅ |
| **SageMaker Studio** | JupyterHub | BSD-3 | Multi-user notebooks | ✅ 📦 |
| | Kubeflow Notebooks | Apache 2.0 | K8s notebooks | ✅ |
| **SageMaker Pipelines** | Kubeflow Pipelines | Apache 2.0 | ML workflows | ✅ |
| | Argo Workflows | Apache 2.0 | General workflows | ✅ |
| | Prefect | Apache 2.0 | Data workflows | ✅ |
| **Rekognition** | DeepFace | MIT | Face recognition | ✅ |
| | YOLO | AGPL-3.0 | Object detection | 🟡 |
| | OpenCV | Apache 2.0 | Computer vision | ✅ |
| **Textract** | Tesseract | Apache 2.0 | OCR | ✅ |
| | PaddleOCR | Apache 2.0 | Advanced OCR | ✅ |
| **Comprehend** | spaCy | MIT | NLP | ✅ |
| | Hugging Face | Apache 2.0 | NLP models | ✅ |
| **Translate** | LibreTranslate | AGPL-3.0 | Translation API | ✅ |
| | Argos Translate | MIT | Offline translation | ✅ |
| **Transcribe** | Whisper | MIT | Speech-to-text | ✅ 📦 |
| | Vosk | Apache 2.0 | Offline STT | ✅ |
| **Polly** | Coqui TTS | MPL 2.0 | Text-to-speech | ✅ |
| | Piper | MIT | Fast TTS | ✅ |
| **Lex** | Rasa | Apache 2.0 | Conversational AI | ✅ |
| | Botpress | MIT | Chatbot platform | ✅ |
| **Personalize** | LensKit | MIT | Recommendations | 🟡 |
| | Surprise | BSD-3 | Recommender systems | 🟡 |
| **Forecast** | Prophet | MIT | Time-series forecast | ✅ |
| | NeuralProphet | MIT | DL forecasting | ✅ |
| | Darts | Apache 2.0 | Forecasting | ✅ |
| **Fraud Detector** | PyOD | BSD-2 | Anomaly detection | 🟡 |

---

## Analytics

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **Athena** | Trino | Apache 2.0 | Distributed SQL | ✅ 📦 |
| | Presto | Apache 2.0 | Original version | ✅ |
| | DuckDB | MIT | Embedded analytics | ✅ |
| **EMR** | Apache Spark | Apache 2.0 | Big data processing | ✅ 📦 |
| | Apache Flink | Apache 2.0 | Stream processing | ✅ |
| **Kinesis Streams** | Apache Kafka | Apache 2.0 | Event streaming | ✅ 📦 |
| | Redpanda | BSL | Kafka-compatible | 🟡 |
| | Apache Pulsar | Apache 2.0 | Messaging/streaming | ✅ |
| | NATS JetStream | Apache 2.0 | Lightweight streaming | ✅ |
| **Kinesis Firehose** | Kafka Connect | Apache 2.0 | Data pipelines | ✅ |
| | Vector | MPL 2.0 | Data pipeline | ✅ 📦 |
| | Fluent Bit | Apache 2.0 | Log forwarding | ✅ |
| **Kinesis Analytics** | Apache Flink | Apache 2.0 | Stream analytics | ✅ |
| | ksqlDB | Confluent | Kafka SQL | 🟡 |
| **OpenSearch Service** | OpenSearch | Apache 2.0 | Search & analytics | ✅ 📦 |
| | Elasticsearch | Elastic/SSPL | Original (license issues) | 🟡 |
| | Meilisearch | MIT | Fast search | ✅ |
| | Typesense | GPL 3.0 | Search engine | ✅ |
| | Zinc | Apache 2.0 | Lightweight search | ✅ |
| **QuickSight** | Apache Superset | Apache 2.0 | BI dashboards | ✅ 📦 |
| | Metabase | AGPL-3.0 | Business analytics | ✅ 📦 |
| | Grafana | AGPL-3.0 | Visualization | ✅ 📦 |
| | Redash | BSD-2 | Query & visualize | ✅ |
| | Lightdash | MIT | dbt-native BI | ✅ |
| **Glue** | Apache Spark | Apache 2.0 | ETL processing | ✅ |
| | Apache NiFi | Apache 2.0 | Data flow | ✅ |
| | dbt | Apache 2.0 | Data transformation | ✅ 📦 |
| | Airbyte | MIT | Data integration | ✅ 📦 |
| | Meltano | MIT | ELT pipelines | ✅ |
| **Lake Formation** | Apache Iceberg | Apache 2.0 | Table format | ✅ |
| | Delta Lake | Apache 2.0 | ACID tables | ✅ |
| | Apache Hudi | Apache 2.0 | Data lake tables | ✅ |
| **Data Exchange** | CKAN | AGPL-3.0 | Data portal | ✅ |
| **CloudSearch** | OpenSearch | Apache 2.0 | Full-text search | ✅ |

---

## Application Integration

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **EventBridge** | Apache Kafka | Apache 2.0 | Event streaming | ✅ |
| | NATS | Apache 2.0 | Messaging system | ✅ 📦 |
| | RabbitMQ | MPL 2.0 | Message broker | ✅ 📦 |
| **SNS** | NATS | Apache 2.0 | Pub/sub messaging | ✅ 📦 |
| | RabbitMQ | MPL 2.0 | Pub/sub support | ✅ |
| | Apache Pulsar | Apache 2.0 | Unified messaging | ✅ |
| **SQS** | RabbitMQ | MPL 2.0 | Message queuing | ✅ 📦 |
| | Redis Streams | BSD-3 | Lightweight queuing | ✅ |
| | BullMQ | MIT | Redis-based queues | ✅ |
| | PostgreSQL (SKIP LOCKED) | PostgreSQL | Simple queuing | ✅ |
| **Step Functions** | Temporal | MIT | Workflow orchestration | ✅ 📦 |
| | Apache Airflow | Apache 2.0 | DAG workflows | ✅ 📦 |
| | Argo Workflows | Apache 2.0 | K8s workflows | ✅ 📦 |
| | Prefect | Apache 2.0 | Modern orchestration | ✅ |
| | n8n | Sustainable Use | Workflow automation | 🟡 |
| **MQ** | RabbitMQ | MPL 2.0 | ActiveMQ alternative | ✅ |
| | Apache ActiveMQ | Apache 2.0 | Direct equivalent | ✅ |
| **AppFlow** | Airbyte | MIT | Data integration | ✅ 📦 |
| | Meltano | MIT | ELT platform | ✅ |
| | Apache NiFi | Apache 2.0 | Data flows | ✅ |
| **AppSync** | Hasura | Apache 2.0 | GraphQL engine | ✅ 📦 |
| | PostGraphile | MIT | GraphQL for PG | ✅ |
| | Apollo Server | MIT | GraphQL server | ✅ |
| **MWAA (Airflow)** | Apache Airflow | Apache 2.0 | Self-hosted | ✅ 📦 |

---

## Management & Governance

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **CloudFormation** | Terraform | MPL 2.0 | IaC | ✅ 📦 |
| | OpenTofu | MPL 2.0 | Terraform fork | ✅ 📦 |
| | Pulumi | Apache 2.0 | Programming IaC | ✅ |
| | Crossplane | Apache 2.0 | K8s-native IaC | ✅ |
| **CloudTrail** | Audit logging | Various | Built into most tools | ✅ |
| **CloudWatch** | Prometheus | Apache 2.0 | Metrics | ✅ 📦 |
| | Grafana | AGPL-3.0 | Visualization | ✅ 📦 |
| | VictoriaMetrics | Apache 2.0 | Metrics storage | ✅ 📦 |
| **CloudWatch Logs** | Loki | AGPL-3.0 | Log aggregation | ✅ 📦 |
| | OpenSearch | Apache 2.0 | Log analysis | ✅ |
| | Graylog | SSPL | Log management | 🟡 |
| | Fluentd | Apache 2.0 | Log collection | ✅ 📦 |
| **X-Ray** | Jaeger | Apache 2.0 | Distributed tracing | ✅ 📦 |
| | Zipkin | Apache 2.0 | Tracing | ✅ |
| | Tempo | AGPL-3.0 | Grafana tracing | ✅ 📦 |
| | SigNoz | MIT | Full observability | ✅ |
| **Config** | OPA | Apache 2.0 | Policy as code | ✅ |
| | Checkov | Apache 2.0 | IaC scanning | ✅ |
| **Systems Manager** | Ansible | GPL 3.0 | Configuration mgmt | ✅ |
| | Salt | Apache 2.0 | Remote execution | ✅ |
| **Trusted Advisor** | Popeye | Apache 2.0 | K8s scanner | ✅ |
| | Polaris | Apache 2.0 | K8s best practices | ✅ |
| **Managed Grafana** | Grafana | AGPL-3.0 | Self-hosted | ✅ 📦 |
| **Managed Prometheus** | Prometheus | Apache 2.0 | Self-hosted | ✅ 📦 |
| | VictoriaMetrics | Apache 2.0 | Prometheus-compatible | ✅ 📦 |
| | Mimir | AGPL-3.0 | Grafana metrics | ✅ |

---

## Developer Tools

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **CodeCommit** | Gitea | MIT | Lightweight Git | ✅ 📦 |
| | GitLab | MIT (CE) | Full DevOps platform | ✅ 📦 |
| | Forgejo | MIT | Gitea community fork | ✅ |
| | Gogs | MIT | Simple Git server | 🟡 |
| **CodeBuild** | Jenkins | MIT | CI/CD server | ✅ |
| | Drone | Apache 2.0 | Container-native CI | ✅ |
| | Tekton | Apache 2.0 | K8s-native CI/CD | ✅ 📦 |
| | Woodpecker | Apache 2.0 | Drone fork | ✅ |
| | Buildkite Agent | MIT | Self-hosted runners | 🟡 |
| **CodeDeploy** | ArgoCD | Apache 2.0 | GitOps deployments | ✅ 📦 |
| | Flux | Apache 2.0 | GitOps toolkit | ✅ |
| | Spinnaker | Apache 2.0 | Multi-cloud CD | 🟡 |
| **CodePipeline** | Tekton Pipelines | Apache 2.0 | K8s pipelines | ✅ |
| | Argo Workflows | Apache 2.0 | Workflow automation | ✅ |
| | Jenkins | MIT | Pipeline orchestration | ✅ |
| **CodeArtifact** | Nexus Repository | EPL 1.0 | Artifact repository | ✅ |
| | Artifactory | Proprietary | Enterprise (free tier) | 🟠 |
| | Verdaccio | MIT | npm registry | ✅ |
| | Sonatype Nexus | EPL 1.0 | Multi-format | ✅ |
| **Cloud9** | code-server | MIT | VS Code in browser | ✅ 📦 |
| | Eclipse Theia | EPL 2.0 | Cloud IDE | ✅ |
| | Jupyter | BSD-3 | Notebook IDE | ✅ |
| **Fault Injection** | Chaos Mesh | Apache 2.0 | K8s chaos engineering | ✅ |
| | Litmus | Apache 2.0 | Chaos engineering | ✅ |

---

## Migration & Transfer

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **DataSync** | Rclone | MIT | Multi-cloud sync | ✅ |
| | Syncthing | MPL 2.0 | P2P sync | ✅ |
| | rsync | GPL 3.0 | File sync | ✅ |
| **Transfer Family** | SFTP Server | Various | OpenSSH | ✅ |
| | MinIO SFTP | AGPL-3.0 | S3 + SFTP | ✅ |

---

## IoT

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **IoT Core** | Eclipse Mosquitto | EPL/EDL | MQTT broker | ✅ |
| | EMQX | Apache 2.0 | Scalable MQTT | ✅ 📦 |
| | VerneMQ | Apache 2.0 | Distributed MQTT | ✅ |
| | HiveMQ (CE) | Apache 2.0 | Enterprise MQTT | 🟡 |
| **IoT Greengrass** | EdgeX Foundry | Apache 2.0 | Edge IoT platform | ✅ |
| | K3s + custom | Apache 2.0 | Edge K8s | ✅ |
| **IoT Analytics** | Apache Kafka + Flink | Apache 2.0 | Stream analytics | ✅ |
| **IoT Events** | Node-RED | Apache 2.0 | Flow-based IoT | ✅ |
| **IoT SiteWise** | ThingsBoard | Apache 2.0 | Industrial IoT | ✅ |
| **IoT TwinMaker** | Eclipse Ditto | EPL 2.0 | Digital twins | 🟡 |

---

## Front-End Web & Mobile

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **Amplify** | Supabase | Apache 2.0 | Backend as service | ✅ 📦 |
| | Appwrite | BSD-3 | Backend platform | ✅ |
| | PocketBase | MIT | Single-file backend | ✅ |
| | Parse | Apache 2.0 | Mobile backend | ✅ |
| **API Gateway** | Kong | Apache 2.0 | API gateway | ✅ 📦 |
| | KrakenD | Apache 2.0 | High-performance | ✅ |
| | Tyk | MPL 2.0 | API management | 🟡 |
| | APISIX | Apache 2.0 | Dynamic gateway | ✅ |
| | Envoy + custom | Apache 2.0 | API proxy | ✅ |
| **Pinpoint** | Novu | MIT | Notifications | ✅ |
| | OneSignal | Proprietary | Push notifications | 🟠 |
| **Location Service** | OpenStreetMap | ODbL | Map data | ✅ |
| | Nominatim | GPL 2.0 | Geocoding | ✅ |
| | OSRM | BSD-2 | Routing | ✅ |
| **SES** | Postal | MIT | Mail server | ✅ 📦 |
| | Mailcow | GPL 3.0 | Mail suite | ✅ |
| | Mailu | MIT | Mail server | ✅ |
| | listmonk | AGPL-3.0 | Newsletter/campaigns | ✅ |

---

## Business Applications

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **Connect** | FreeSWITCH | MPL 1.1 | Telephony | 🟡 |
| | Asterisk | GPL 2.0 | PBX system | 🟡 |
| **Chime** | Jitsi Meet | Apache 2.0 | Video conferencing | ✅ 📦 |
| | BigBlueButton | LGPL 3.0 | Web conferencing | ✅ |
| | Element (Matrix) | Apache 2.0 | Team chat + calls | ✅ |
| **WorkMail** | Mailcow | GPL 3.0 | Email + calendar | ✅ |
| | Zimbra | Various | Email suite | 🟡 |
| **WorkDocs** | Nextcloud | AGPL-3.0 | File sharing | ✅ 📦 |
| | Seafile | GPL 2.0 | File sync | ✅ |
| | OnlyOffice | AGPL-3.0 | Doc collaboration | ✅ |
| **Wickr** | Element (Matrix) | Apache 2.0 | E2E encrypted chat | ✅ |
| | Signal Server | AGPL-3.0 | Secure messaging | 🟡 |
| **Honeycode** | NocoDB | AGPL-3.0 | No-code database | ✅ 📦 |
| | Baserow | MIT | Airtable alternative | ✅ |
| | Budibase | GPL 3.0 | Low-code apps | ✅ |
| | Appsmith | Apache 2.0 | Internal tools | ✅ |
| | Tooljet | GPL 3.0 | Low-code platform | ✅ |

---

## End User Computing

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **WorkSpaces** | Apache Guacamole | Apache 2.0 | Remote desktop gateway | ✅ |
| | Kasm Workspaces | Various | Browser-based desktops | 🟡 |
| | Proxmox VE | AGPL-3.0 | VDI solution | ✅ |
| **AppStream** | Apache Guacamole | Apache 2.0 | Remote apps | ✅ |

---

## Blockchain

| AWS Service | Open Source Alternative | License | Notes | Status |
|-------------|------------------------|---------|-------|--------|
| **Managed Blockchain** | Hyperledger Fabric | Apache 2.0 | Enterprise blockchain | ✅ |
| | Hyperledger Besu | Apache 2.0 | Ethereum client | ✅ |
| **QLDB** | immudb | Apache 2.0 | Immutable database | ✅ |

---

## Summary: CTO Platform Marketplace Priorities

### Tier 1: Core Infrastructure (Included by Default)

| Category | Service | Alternative |
|----------|---------|-------------|
| Orchestration | EKS/ECS | Kubernetes (Talos) |
| Storage | S3 | MinIO |
| Database | RDS | PostgreSQL (CNPG) |
| Cache | ElastiCache | Redis/Valkey |
| Secrets | Secrets Manager | OpenBao |
| Observability | CloudWatch | Prometheus + Grafana + Loki |
| Ingress | ALB | Traefik/Nginx |
| GitOps | CodeDeploy | ArgoCD |

### Tier 2: One-Click Add-ons (Marketplace)

| Category | Service | Alternative | Priority |
|----------|---------|-------------|----------|
| Search | OpenSearch | OpenSearch | High |
| Messaging | SQS/SNS | RabbitMQ/NATS | High |
| Workflow | Step Functions | Temporal | High |
| BI | QuickSight | Metabase/Superset | High |
| ETL | Glue | Airbyte + dbt | High |
| Streaming | Kinesis | Kafka | High |
| Serverless | Lambda | Knative/OpenFaaS | Medium |
| AI/ML | SageMaker | Kubeflow | Medium |
| LLM | Bedrock | Ollama/vLLM | High |
| API Gateway | API Gateway | Kong | Medium |
| Auth | Cognito | Keycloak | High |
| Email | SES | Postal | Medium |
| Video | Chime | Jitsi | Low |
| Files | WorkDocs | Nextcloud | Medium |
| No-code | Honeycode | NocoDB | Low |
| GraphQL | AppSync | Hasura | Medium |

### Tier 3: Enterprise Add-ons

| Category | Service | Alternative |
|----------|---------|-------------|
| Service Mesh | App Mesh | Istio/Linkerd |
| Chaos | Fault Injection | Chaos Mesh |
| ML Platform | SageMaker | Kubeflow |
| Graph DB | Neptune | Dgraph |
| Time-series | Timestream | TimescaleDB |
| Analytics | Redshift | ClickHouse |

---

## License Compatibility Notes

### ✅ Safe to Bundle (Apache 2.0, MIT, BSD, PostgreSQL, MPL 2.0)
- Most K8s ecosystem tools
- Prometheus, Grafana (with attribution)
- PostgreSQL and ecosystem
- MinIO (with AGPL consideration)

### 🟡 Use with Caution (AGPL-3.0, GPL)
- AGPL requires source disclosure if modified and served over network
- GPL requires source disclosure if distributed
- Can run as separate service, not embedded

### ❌ Cannot Bundle Commercially
- SSPL (MongoDB, Graylog)
- BSL (HashiCorp newer versions, CockroachDB)
- Some proprietary "community editions"

---

*Last updated: November 2024*
*For CTO Platform Service Marketplace planning*
