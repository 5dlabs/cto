# Enhanced PRD

## 1. Original Requirements

> # Project: Sigma-1 — Unified AI Business Platform for Perception Events
>
> - **Website:** https://sigma-1.com
>
> ## Vision
>
> Sigma-1 is a lighting and visual production company (Perception Events). This platform replaces their fragmented tools, manual processes, and administrative overhead with a single intelligent agent — **Morgan** — accessible through Signal, phone, and web.
>
> Instead of juggling rental software, spreadsheets, phone calls, accounting tools, and social media apps, everything runs through one interface: send Morgan a message, and it handles the rest.
>
> ---
>
> ## Architecture Overview
>
> ```
> ┌─────────────────────────────────────────────────────────────────────┐
> │                     Sigma-1 Platform                                 │
> ├─────────────────────────────────────────────────────────────────────┤
> │  Clients                                                             │
> │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
> │  │  Signal  │  │   Voice  │  │   Web    │  │  Mobile  │        │
> │  │  (Morgan)│  │ (ElevenLabs│ │ (Next.js)│  │  (Expo)  │        │
> │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
> │       │             │             │             │                 │
> ├───────┴─────────────┴─────────────┴─────────────┴─────────────────┤
> │  Backend Services                                                    │
> │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐      │
> │  │   Equipment    │  │     RMS        │  │    Finance     │      │
> │  │   Catalog      │  │   Service      │  │    Service     │      │
> │  │   (Rust/Axum)  │  │   (Go/gRPC)    │  │   (Rust/Axum)  │      │
> │  │     Rex        │  │     Grizz      │  │     Rex        │      │
> │  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘      │
> │          │                    │                    │                 │
> │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐      │
> │  │   (Out of      │  │     Social     │  │    Customer    │      │
> │  │    Scope)      │  │    Engine      │  │    Vetting     │      │
> │  │  (Phase 2)    │  │(Node/Elysia)  │  │  (Rust/Axum)   │      │
> │  │                │  │     Nova       │  │     Rex        │      │
> │  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘      │
> │          │                    │                    │                 │
> ├──────────┴────────────────────┴────────────────────┴─────────────────┤
> │  Infrastructure                                                      │
> │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │
> │  │PostgreSQL│ │  Redis  │ │  S3/R2  │ │ ElevenLabs│ │ Twilio  │     │
> │  │         │ │         │ │         │ │          │ │         │     │
> │  └─────────┘ └─────────┘ └─────────┘ └──────────┘ └─────────┘     │
> │  ┌─────────┐ ┌─────────┐                                             │
> │  │SignalCLI│ │OpenCorporates│                                         │
> │  └─────────┘ └─────────┘                                             │
> └─────────────────────────────────────────────────────────────────────┘
> ```
>
> ---
>
> ## Services (Workstreams)
>
> ### 1. Morgan AI Agent (OpenClaw)
>
> **Agent**: Morgan (OpenClaw agent)
> **Priority**: Critical
> **Runtime**: OpenClaw with MCP tools
>
> The central AI agent that handles all customer interactions via Signal, voice, and web chat.
>
> **Core Features**:
> - Signal messenger integration (receive/send messages, photos)
> - Voice calls via ElevenLabs (SIP/PSTN, natural conversation)
> - Web chat widget for website
> - Lead qualification and customer vetting
> - Quote generation coordination
> - Social media approval workflow
> - Natural language queries to all backend services
>
> **MCP Tools** (accesses via tool-server):
> ```
> sigma1_catalog_search     — search products by name/category/specs
> sigma1_check_availability — check date range availability for items
> sigma1_generate_quote     — create opportunity with line items
> sigma1_vet_customer       — run background check pipeline
> sigma1_score_lead         — compute GREEN/YELLOW/RED score
> sigma1_create_invoice     — generate invoice from project
> sigma1_finance_report     — pull financial summaries
> sigma1_social_curate      — trigger photo curation pipeline
> sigma1_social_publish     — publish approved draft
> sigma1_equipment_lookup   — search secondary markets for arbitrage
> ```
>
> **Skills**:
> - `sales-qual` — Lead qualification workflow
> - `customer-vet` — Background research (OpenCorporates, LinkedIn, Google Reviews)
> - `quote-gen` — Equipment quote generation
> - `upsell` — Insurance, services, packages recommendations
> - `finance` — Invoice generation, financial summaries
> - `social-media` — Photo curation, caption generation
> - `rms-*` — Rental management operations
> - `admin` — Calendar, email drafting, document management
>
> **Infrastructure Dependencies**:
> - Signal-CLI (sidecar or separate pod)
> - ElevenLabs (voice)
> - Twilio (phone numbers, SIP/PSTN)
> - All backend service APIs
>
> ---
>
> ### 2. Equipment Catalog Service (Rust/Axum)
>
> **Agent**: Rex
> **Priority**: High
> **Language**: Rust 1.75+
> **Framework**: Axum 0.7
>
> High-performance API for equipment inventory, availability checking, and self-service quoting.
>
> **Endpoints**:
> ```
> GET    /api/v1/catalog/categories           — List categories
> GET    /api/v1/catalog/products             — List products (filterable)
> GET    /api/v1/catalog/products/:id        — Get product details
> GET    /api/v1/catalog/products/:id/availability?from=&to= — Check availability
> POST   /api/v1/catalog/products             — Add product (admin)
> PATCH  /api/v1/catalog/products/:id         — Update product (admin)
> GET    /api/v1/equipment-api/catalog       — Machine-readable (for AI agents)
> POST   /api/v1/equipment-api/checkout      — Programmatic booking
> GET    /metrics                             — Prometheus metrics
> GET    /health/live                          — Liveness probe
> GET    /health/ready                        — Readiness probe
> ```
>
> **Core Features**:
> - 533+ products across 24 categories
> - Real-time availability checking
> - Barcode/SKU lookup
> - Image serving (S3/R2 CDN)
> - Machine-readable equipment API for other AI agents
> - Rate limiting per tenant
>
> **Data Models**:
> ```rust
> struct Product {
>     id: Uuid,
>     name: String,
>     category_id: Uuid,
>     description: String,
>     day_rate: Decimal,
>     weight_kg: Option<f32>,
>     dimensions: Option<Dimensions>,
>     image_urls: Vec<String>,
>     specs: JsonB,
>     created_at: DateTime<Utc>,
> }
>
> struct Category {
>     id: Uuid,
>     name: String,
>     parent_id: Option<Uuid>,
>     icon: String,
>     sort_order: i32,
> }
>
> struct Availability {
>     product_id: Uuid,
>     date_from: NaiveDate,
>     date_to: NaiveDate,
>     quantity_available: i32,
>     reserved: i32,
>     booked: i32,
> }
> ```
>
> **Infrastructure Dependencies**:
> - PostgreSQL (product catalog, availability)
> - Redis (rate limiting, caching)
> - S3/R2 (product images)
>
> ---
>
> ### 3. Rental Management System — RMS (Go/gRPC)
>
> **Agent**: Grizz
> **Priority**: High
> **Language**: Go 1.22+
> **Framework**: gRPC with grpc-gateway for REST
>
> Full replacement for Current RMS — bookings, projects, inventory, logistics, crew management.
>
> **gRPC Services**:
> ```protobuf
> service OpportunityService {
>   rpc CreateOpportunity(CreateOpportunityRequest) returns (Opportunity);
>   rpc GetOpportunity(GetOpportunityRequest) returns (Opportunity);
>   rpc UpdateOpportunity(UpdateOpportunityRequest) returns (Opportunity);
>   rpc ListOpportunities(ListOpportunitiesRequest) returns (ListOpportunitiesResponse);
>   rpc ScoreLead(ScoreLeadRequest) returns (LeadScore);
> }
>
> service ProjectService {
>   rpc CreateProject(CreateProjectRequest) returns (Project);
>   rpc GetProject(GetProjectRequest) returns (Project);
>   rpc UpdateProject(UpdateProjectRequest) returns (Project);
>   rpc CheckOut(CheckOutRequest) returns (CheckOutResponse);
>   rpc CheckIn(CheckInRequest) returns (CheckInResponse);
> }
>
> service InventoryService {
>   rpc GetStockLevel(GetStockLevelRequest) returns (StockLevel);
>   rpc RecordTransaction(RecordTransactionRequest) returns (Transaction);
>   rpc ScanBarcode(ScanBarcodeRequest) returns (InventoryItem);
> }
>
> service CrewService {
>   rpc ListCrew(ListCrewRequest) returns (ListCrewResponse);
>   rpc AssignCrew(AssignCrewRequest) returns (Project);
>   rpc ScheduleCrew(ScheduleCrewRequest) returns (Schedule);
> }
>
> service DeliveryService {
>   rpc ScheduleDelivery(ScheduleDeliveryRequest) returns (Delivery);
>   rpc UpdateDeliveryStatus(UpdateDeliveryStatusRequest) returns (Delivery);
>   rpc OptimizeRoute(OptimizeRouteRequest) returns (Route);
> }
> ```
>
> **REST Endpoints** (via grpc-gateway):
> ```
> # Opportunities (Quotes)
> POST   /api/v1/opportunities
> GET    /api/v1/opportunities/:id
> PATCH  /api/v1/opportunities/:id
> POST   /api/v1/opportunities/:id/approve
> POST   /api/v1/opportunities/:id/convert    # → project
>
> # Projects
> GET    /api/v1/projects
> GET    /api/v1/projects/:id
> POST   /api/v1/projects/:id/checkout
> POST   /api/v1/projects/:id/checkin
>
> # Inventory
> GET    /api/v1/inventory/transactions
> POST   /api/v1/inventory/transactions
>
> # Crew
> GET    /api/v1/crew
> POST   /api/v1/crew/assign
>
> # Deliveries
> POST   /api/v1/deliveries/schedule
> GET    /api/v1/deliveries/:id/route
> ```
>
> **Core Features**:
> - Quote-to-project workflow
> - Barcode scanning for check-out/check-in
> - Crew scheduling and assignment
> - Vehicle/delivery tracking
> - Calendar integration (Google Calendar)
> - Conflict detection
>
> **Data Models**:
> ```go
> type Opportunity struct {
>     ID          uuid.UUID
>     CustomerID  uuid.UUID
>     Status      string // pending, qualified, approved, converted
>     EventDateStart time.Time
>     EventDateEnd   time.Time
>     Venue       string
>     TotalEstimate decimal.Decimal
>     LeadScore   string // GREEN, YELLOW, RED
>     Notes       string
> }
>
> type Project struct {
>     ID              uuid.UUID
>     OpportunityID   uuid.UUID
>     CustomerID      uuid.UUID
>     Status          string // confirmed, in_progress, completed, cancelled
>     ConfirmedAt     *time.Time
>     EventDates      DateRange
>     VenueAddress    string
>     CrewNotes       string
> }
>
> type InventoryTransaction struct {
>     ID            uuid.UUID
>     InventoryID   uuid.UUID
>     Type          string // checkout, checkin, transfer
>     ProjectID     *uuid.UUID
>     FromStoreID   uuid.UUID
>     ToStoreID     uuid.UUID
>     Timestamp     time.Time
>     UserID        uuid.UUID
> }
> ```
>
> **Infrastructure Dependencies**:
> - PostgreSQL (all RMS data)
> - Redis (session cache)
> - Google Calendar API
>
> ---
>
> ### 4. Finance Service (Rust/Axum)
>
> **Agent**: Rex
> **Priority**: High
> **Language**: Rust 1.75+
> **Framework**: Axum 0.7
>
> Invoicing, payments, AP/AR, payroll, multi-currency support. Replaces QuickBooks/Xero.
>
> **Endpoints**:
> ```
> # Invoices
> POST   /api/v1/invoices                    — Create invoice
> GET    /api/v1/invoices                    — List invoices
> GET    /api/v1/invoices/:id                — Get invoice
> POST   /api/v1/invoices/:id/send           — Send to customer
> POST   /api/v1/invoices/:id/paid           — Record payment
>
> # Payments
> POST   /api/v1/payments                    — Record payment
> GET    /api/v1/payments                    — List payments
> GET    /api/v1/payments/invoice/:id        — Payments for invoice
>
> # Finance Reports
> GET    /api/v1/finance/reports/revenue?period=    — Revenue report
> GET    /api/v1/finance/reports/aging               — AR aging report
> GET    /api/v1/finance/reports/cashflow            — Cash flow report
> GET    /api/v1/finance/reports/profitability       — Job profitability
>
> # Payroll
> GET    /api/v1/payroll?period=            — Payroll report
> POST   /api/v1/payroll/entries            — Add payroll entry
>
> # Currency
> GET    /api/v1/currency/rates             — Current rates
> ```
>
> **Core Features**:
> - Quote-to-invoice conversion
> - Multi-currency support (USD, CAD, AUD, NZD, etc.)
> - Stripe integration for payments
> - Automated payment reminders
> - AR aging reports
> - Payroll tracking (contractor/employee)
> - Tax calculation (GST/HST, US sales tax, international)
> - Currency rate sync (scheduled job)
>
> **Data Models**:
> ```rust
> struct Invoice {
>     id: Uuid,
>     project_id: Uuid,
>     org_id: Uuid,
>     invoice_number: String,
>     status: InvoiceStatus, // draft, sent, viewed, paid, overdue
>     issued_at: DateTime<Utc>,
>     due_at: NaiveDate,
>     currency: String,
>     subtotal_cents: i64,
>     tax_cents: i64,
>     total_cents: i64,
>     paid_amount_cents: i64,
>     stripe_invoice_id: Option<String>,
> }
>
> struct Payment {
>     id: Uuid,
>     invoice_id: Uuid,
>     amount_cents: i64,
>     currency: String,
>     method: PaymentMethod, // cash, check, wire, card, stripe
>     stripe_payment_id: Option<String>,
>     received_at: DateTime<Utc>,
> }
>
> enum InvoiceStatus {
>     Draft,
>     Sent,
>     Viewed,
>     Paid,
>     Overdue,
>     Cancelled,
> }
> ```
>
> **Infrastructure Dependencies**:
> - PostgreSQL (finance data)
> - Stripe API
> - Redis (currency rate cache)
>
> ---
>
> ### 5. Customer Vetting Service (Rust/Axum)
>
> **Agent**: Rex
> **Priority**: High
> **Language**: Rust 1.75+
> **Framework**: Axum 0.7
>
> Automated background research on prospects: business registration, online presence, reputation, credit signals.
>
> **Endpoints**:
> ```
> POST   /api/v1/vetting/run                 — Run full vetting pipeline
> GET    /api/v1/vetting/:org_id             — Get vetting results
> GET    /api/v1/vetting/credit/:org_id      — Get credit signals
> ```
>
> **Core Features**:
> - OpenCorporates API integration (business registration verification)
> - LinkedIn company research
> - Google Reviews sentiment analysis
> - Credit signal lookup (via commercial APIs)
> - Automated GREEN/YELLOW/RED scoring
>
> **Vetting Pipeline**:
> 1. **Business Verification** — OpenCorporates: company exists, good standing, directors
> 2. **Online Presence** — LinkedIn page, website, social media
> 3. **Reputation** — Google Reviews, industry mentions
> 4. **Credit Signals** — Payment history indicators, financial health
> 5. **Final Score** — Weighted algorithm → GREEN/YELLOW/RED
>
> **Data Models**:
> ```rust
> struct VettingResult {
>     org_id: Uuid,
>     business_verified: bool,
>     opencorporates_data: Option<OpenCorporatesData>,
>     linkedin_exists: bool,
>     linkedin_followers: i32,
>     google_reviews_rating: Option<f32>,
>     google_reviews_count: i32,
>     credit_score: Option<i32>,
>     risk_flags: Vec<String>,
>     final_score: LeadScore,
>     vetted_at: DateTime<Utc>,
> }
>
> enum LeadScore {
>     GREEN,  // Proceed with confidence
>     YELLOW, // More verification needed
>     RED,    // High risk, decline or require deposit
> }
> ```
>
> **Infrastructure Dependencies**:
> - PostgreSQL (vetting results)
> - OpenCorporates API
> - LinkedIn API
> - Google Reviews (scraping or API)
> - Credit data APIs
>
> ---
>
> > **Note:** Trading Desk service is out of scope for Phase 1 (Python not in core stack).
>
> ### 7. Social Media Engine (Node.js/Elysia + Effect)
>
> **Agent**: Nova
> **Priority**: Medium
> **Runtime**: Node.js 20+
> **Framework**: Elysia 1.x with Effect TypeScript
>
> Automated content curation, caption generation, and multi-platform publishing.
>
> **Endpoints**:
> ```
> POST   /api/v1/social/upload               — Upload event photos
> GET    /api/v1/social/drafts                — List draft posts
> GET    /api/v1/social/drafts/:id            — Get draft details
> POST   /api/v1/social/drafts/:id/approve    — Approve for publishing
> POST   /api/v1/social/drafts/:id/reject    — Reject draft
> POST   /api/v1/social/drafts/:id/publish   — Publish to platforms
> GET    /api/v1/social/published            — List published posts
> ```
>
> **Core Features**:
> - **AI Curation** — Score compositions, select top 5-10 images
> - **Platform-specific cropping** — Instagram (square/Story), LinkedIn (landscape), TikTok
> - **Caption generation** — Event context, equipment featured, hashtags
> - **Multi-platform publishing** — Instagram, TikTok, LinkedIn, Facebook
> - **Approval workflow** — Morgan sends drafts to Mike via Signal, one-tap approval
> - **Portfolio sync** — Published content → website automatically
>
> **Content Pipeline**:
> ```
> Event Photos → AI Curation → Draft Generation → Signal Approval → Multi-Platform Publish
> ```
>
> **Effect Integration**:
> | Pattern | Usage |
> |---------|-------|
> | `Effect.Service` | InstagramService, LinkedInService, TikTokService |
> | `Effect.retry` | API delivery with exponential backoff |
> | `Effect.Schema` | Request/response validation |
>
> **Infrastructure Dependencies**:
> - PostgreSQL (drafts, published posts)
> - Instagram Graph API
> - LinkedIn API
> - Facebook Graph API
> - S3/R2 (photo storage)
> - OpenAI/Claude for caption generation
>
> ---
>
> ### 8. Website — Next.js 15 (React/Next.js + Effect)
>
> **Agent**: Blaze
> **Priority**: High
> **Framework**: Next.js 15 (App Router)
> **UI**: React 19, shadcn/ui, TailwindCSS 4
> **Type System**: Effect 3.x + TypeScript 5.x
>
> AI-optimized website with equipment catalog, self-service quotes, and Morgan web chat.
>
> **Pages**:
> | Route | Purpose | Effect Usage |
> |-------|---------|--------------| 
> | `/` | Hero, value prop, CTA | Static content |
> | `/equipment` | Browse 533+ products | Effect data fetching |
> | `/equipment/:id` | Product detail + availability | Effect Schema validation |
> | `/quote` | Self-service quote builder | Effect form validation |
> | `/portfolio` | Past events gallery | Effect data fetching |
> | `/llms.txt` | Machine-readable for AI agents | Static |
> | `/llms-full` | Full content dump for AI | Static |
>
> **Core Features**:
> - **Equipment catalog** with real-time availability
> - **Self-service quote builder** — Select products, dates → submit for review
> - **Morgan web chat** — Embedded chat widget
> - **AI-native optimization** — llms.txt, Schema.org structured data
> - **Project portfolio** — Event photos, equipment used, testimonials
>
> **Technology Stack**:
> | Component | Technology |
> |-----------|------------|
> | Framework | Next.js 15 App Router |
> | UI Library | React 19 |
> | Components | shadcn/ui |
> | Styling | TailwindCSS 4 |
> | Type System | Effect + TypeScript 5.x |
> | Validation | Effect Schema |
> | Data Fetching | TanStack Query + Effect |
> | Hosting | Cloudflare Pages |
>
> **Infrastructure Dependencies**:
> - Cloudflare Pages (static + SSR)
> - Equipment Catalog API
> - Morgan agent (web chat)
>
> ---
>
> ### 9. Infrastructure & Deployment (Kubernetes)
>
> **Agent**: Infra + Metal
> **Priority**: Critical
>
> **Kubernetes Resources**:
> ```yaml
> # PostgreSQL (CloudNative-PG)
> apiVersion: postgresql.cnpg.io/v1
> kind: Cluster
> metadata:
>   name: sigma1-postgres
>   namespace: databases
> spec:
>   instances: 1
>   storage:
>     size: 50Gi
>   bootstrap:
>     initdb:
>       database: sigma1
>       owner: sigma1_user
>   # Multiple schemas: rms, crm, finance, audit, public
>
> # Redis/Valkey
> apiVersion: redis.redis.opstreelabs.in/v1beta2
> kind: Redis
> metadata:
>   name: sigma1-valkey
>   namespace: databases
> spec:
>   kubernetesConfig:
>     image: valkey/valkey:7.2-alpine
>
> # Morgan Agent (OpenClaw)
> apiVersion: v1
> kind: Deployment
> metadata:
>   name: morgan
>   namespace: openclaw
> spec:
>   replicas: 1
>   template:
>     spec:
>       containers:
>       - name: agent
>         image: openclaw/openclaw-agent:latest
>         env:
>         - name: AGENT_ID
>           value: morgan
>         - name: MODEL
>           value: openai-api/gpt-5.4-pro
>         volumeMounts:
>         - name: workspace
>           mountPath: /workspace
>       volumes:
>       - name: workspace
>         persistentVolumeClaim:
>           claimName: morgan-workspace
>
> # Backend Services (Rust, Go, Node.js)
> apiVersion: apps/v1
> kind: Deployment
> metadata:
>   name: equipment-catalog
>   namespace: sigma1
> spec:
>   replicas: 2
>   # ...
>
> # Cloudflare Tunnel for Morgan
> apiVersion: v1
> kind: Service
> metadata:
>   name: morgan-tunnel
>   annotations:
>     cloudflare.com/ingress/controller: "true"
> ```
>
> **Infrastructure Components**:
> | Component | Technology | Purpose |
> |-----------|------------|---------|
> | Database | PostgreSQL 16 | All structured data |
> | Cache | Redis/Valkey | Rate limiting, sessions |
> | Object Storage | Cloudflare R2 / AWS S3 | Images, photos |
> | CDN | Cloudflare | Static assets, SSL |
> | Ingress | Cloudflare Tunnel | Morgan access |
> | Observability | Grafana + Loki + Prometheus | Existing OpenClaw stack |
>
> ---
>
> ## Technical Context
>
> | Service | Technology | Agent |
> |---------|------------|-------|
> | Morgan Agent | OpenClaw | Morgan |
> | Equipment Catalog | Rust 1.75+, Axum 0.7 | Rex |
> | RMS | Go 1.22+, gRPC | Grizz |
> | Finance | Rust 1.75+, Axum 0.7 | Rex |
> | Customer Vetting | Rust 1.75+, Axum 0.7 | Rex |
> | Trading Desk | ~~Python 3.12+~~ (Phase 2) | TBD |
> | Social Engine | Node.js 20+, Elysia + Effect | Nova |
> | Website | Next.js 15 + React 19 + Effect | Blaze |
> | Infrastructure | Kubernetes, CloudNative-PG | Infra + Metal |
>
> ---
>
> ## Data Flow Examples
>
> ### DF-1: Inbound Lead → Qualified Opportunity
>
> ```
> Customer (Signal) ──► Morgan
>                            │
>                     ┌──────┴──────┐
>                     │ Qualify Lead │
>                     │ 1. Parse intent │
>                     │ 2. Ask questions │
>                     │ 3. Check inventory │
>                     └──────┬──────┘
>                            │
>                     ┌──────▼──────┐
>                     │ Vet Customer │
>                     │ (Rex svc)   │
>                     └──────┬──────┘
>                            │
>                     ┌──────▼──────┐
>                     │ Score Lead  │
>                     │ GREEN/YELLOW/RED │
>                     └──────┬──────┘
>                            │
>                     Mike approves ──► Opportunity created
> ```
>
> ### DF-2: Quote → Invoice → Payment
>
> ```
> Quote Request ──► Morgan ──► Equipment Catalog (availability)
>                            │
>                     ┌──────▼──────┐
>                     │ Generate Quote │
>                     │ (RMS service) │
>                     └──────┬──────┘
>                            │
>                     Customer approves ──► Opportunity → Project
>                                               │
>                                     ┌────────▼────────┐
>                                     │ Generate Invoice │
>                                     │ (Finance svc)   │
>                                     └────────┬────────┘
>                                              │
>                                     ┌────────▼────────┐
>                                     │ Stripe Payment  │
>                                     └────────┬────────┘
>                                              │
>                                     ┌────────▼────────┐
>                                     │ Invoice Paid    │
>                                     └─────────────────┘
> ```
>
> ---
>
> ## Constraints
>
> - Morgan must respond within 10 seconds for simple queries
> - Equipment availability check < 500ms
> - Invoice generation < 5 seconds
> - Support 500+ concurrent Signal connections
> - 99.9% uptime for production services
> - GDPR compliant (data export, customer deletion)
>
> ---
>
> ## Quality Assurance & Review Workflow
>
> All code changes go through an automated quality pipeline leveraging multiple CTO agents for comprehensive coverage:
>
> ### 1. Automated Code Review (Stitch)
> - **Agent**: Stitch — Automated Code Reviewer
> - **Trigger**: On every pull request
> - **Scope**: Style, correctness, architecture alignment
> - **Tools**: GitHub PR integration via GitHub App
> - **MCP Tools**: `github_get_pull_request`, `github_get_pull_request_files`
>
> ### 2. Code Quality Enforcement (Cleo)
> - **Agent**: Cleo — Quality Guardian
> - **Trigger**: CI/CD pipeline
> - **Focus**: Maintainability, refactor opportunities, code smells
> - **Tools**: Clippy, ESLint, Rustfmt, biome.js, shadcn lint rules
> - **Output**: PR comments with improvement suggestions
>
> ### 3. Comprehensive Testing (Tess)
> - **Agent**: Tess — Testing Genius
> - **Trigger**: CI/CD pipeline after review approval
> - **Coverage**: Unit tests, integration tests, end-to-end tests
> - **Tools**: Jest/Vitest, PyTest, Cargo Test
> - **Enforcement**: Minimum 80% code coverage required
>
> ### 4. Security Scanning (Cipher)
> - **Agent**: Cipher — Security Sentinel
> - **Trigger**: CI/CD pipeline
> - **Focus**: Vulnerabilities, dependency scanning, OWASP compliance
> - **Tools**: Semgrep, CodeQL, Snyk/GitHub Dependabot
> - **Blocker**: Critical/high severity issues block merge
>
> ### 5. Merge Gate (Atlas)
> - **Agent**: Atlas — Integration Master
> - **Policy**: Required approvals + passing CI + passing QA
> - **Conflict Resolution**: Automatic merge conflict detection/resolution
> - **Tools**: GitHub merge automation
> - **MCP Tools**: `github_merge_pull_request`, `github_get_pull_request`
>
> ### 6. Deployment & Operations (Bolt)
> - **Agent**: Bolt — DevOps Engineer
> - **Platform**: Kubernetes, ArgoCD, CloudNative-PG
> - **Workflow**: GitOps with automatic rollbacks on failure
> - **Monitoring**: Grafana/Loki/Prometheus
> - **Tools**: `kubectl`, `helm`, `argocd` CLI, Cloudflare Terraform
>
> This automated workflow ensures production-ready quality with minimal human intervention.
>
> ---
>
> ## Non-Goals
>
> - SMS notifications (use Signal/Twilio)
> - Self-hosted deployment (managed by 5D Labs)
> - Multi-region deployment (single cluster initially)
> - Real-time equipment tracking (GPS)
> - Employee scheduling beyond crew
>
> ---
>
> ## Success Criteria
>
> 1. Morgan handles 80%+ of customer inquiries autonomously
> 2. Equipment catalog serves 533+ products with real-time availability
> 3. Quote-to-invoice workflow completes in < 2 minutes
> 4. Social media pipeline runs without manual intervention
> 5. All services build, test, and deploy successfully
> 6. End-to-end flow works: Signal message → Morgan → action → confirmation

---

## 2. Project Scope

The initial task decomposition identified **10 tasks** spanning infrastructure provisioning, backend service development, frontend development, and production hardening.

### Task Summary

| ID | Title | Agent | Stack | Priority | Dependencies |
|----|-------|-------|-------|----------|-------------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | High | None |
| 2 | Equipment Catalog Service API | Rex | Rust/Axum | High | Task 1 |
| 3 | Rental Management System (RMS) | Grizz | Go/gRPC | High | Task 1 |
| 4 | Finance Service | Rex | Rust/Axum | High | Task 1 |
| 5 | Customer Vetting Service | Rex | Rust/Axum | High | Task 1 |
| 6 | Social Media Engine | Nova | Node.js/Elysia + Effect | Medium | Task 1 |
| 7 | Morgan AI Agent | Angie | OpenClaw/MCP | High | Tasks 2, 3, 4, 5, 6 |
| 8 | Web Frontend | Blaze | Next.js 15/React 19 | High | Tasks 2, 7 |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | High | Tasks 2–8 |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit | Bolt | Kubernetes/Helm | High | Task 9 |

### Key Services and Components

- **Backend Services (5)**: Equipment Catalog (Rust), RMS (Go), Finance (Rust), Customer Vetting (Rust), Social Engine (Node.js)
- **AI Agent (1)**: Morgan (OpenClaw with MCP tools)
- **Frontend (1)**: Next.js 15 website with React 19, shadcn/ui, TailwindCSS 4
- **Infrastructure (3 task groups)**: Core provisioning, HA/CDN/TLS hardening, RBAC/secrets/audit hardening

### Agent Assignments

| Agent | Tasks | Technology |
|-------|-------|-----------|
| Bolt | 1, 9, 10 | Kubernetes, Helm, ArgoCD, CloudNative-PG |
| Rex | 2, 4, 5 | Rust 1.75+, Axum 0.7 |
| Grizz | 3 | Go 1.22+, gRPC, grpc-gateway |
| Nova | 6 | Node.js 20+, Elysia 1.x, Effect 3.x |
| Angie | 7 | OpenClaw, MCP tools |
| Blaze | 8 | Next.js 15, React 19, shadcn/ui, TailwindCSS 4 |

### Cross-Cutting Concerns

- **12 decision points** identified across tasks, covering platform choice, service topology, API design, data model, security, UX behavior, component library, and build-vs-buy
- **Multi-language stack**: Rust, Go, Node.js/TypeScript across backend services — flagged as operational risk in deliberation
- **Signal-CLI as single point of failure**: Critical dependency for Morgan's primary interaction channel
- **GDPR compliance**: Required across all services that handle customer data
- **Observability**: All services must expose Prometheus metrics, health endpoints, and integrate with Grafana/Loki/Prometheus stack

---

## 3. Resolved Decisions

### [D1] Valkey or Redis for caching/rate-limiting/sessions?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 4, 5, 9 — all services requiring cache, rate limiting, or session storage

**Context:** The infra YAML already specifies `valkey/valkey:7.2-alpine`. Both debaters agreed unanimously that Valkey is the correct choice.

**Decision:** **Valkey 7.2** (as specified in infra YAML), using existing Redis-compatible client libraries (`redis-rs`, `go-redis`, `ioredis`) across Rust, Go, and Node.js.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Full Redis API compatibility confirmed by every major client library. Linux Foundation governance eliminates Redis Labs re-licensing risk. Zero library changes needed.
- *Negative:* None identified. Valkey maintains the same RESP3 protocol.
- *Caveats:* None.

---

### [D3] What API paradigm for Morgan-to-backend communication?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 4, 5, 6, 7 — all services Morgan interacts with

**Context:** MCP tools are HTTP-native. The RMS service already exposes REST via grpc-gateway. Both debaters agreed standardizing on REST for Morgan simplifies the integration surface.

**Decision:** **Standardize on REST for all Morgan-to-backend calls.** gRPC remains available for internal service-to-service communication where beneficial (e.g., RMS internal calls).

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* One HTTP client, one auth pattern, one retry strategy, one logging format across all MCP tool implementations. grpc-gateway already exposes REST on RMS.
- *Negative:* Morgan does not leverage gRPC's binary efficiency for RMS calls, but this is negligible given the HTTP-native MCP architecture.
- *Caveats:* None.

---

### [D4] Cloudflare R2 or AWS S3 for object storage?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 6, 8, 9 — product images, social media photos, static assets

**Context:** The cluster already runs Cloudflare Operator and Tunnel. Both debaters agreed R2 is the natural choice.

**Decision:** **Cloudflare R2** with S3-compatible API.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Zero egress fees. Native CDN integration with existing Cloudflare infrastructure. S3-compatible API means no library changes.
- *Negative:* Vendor coupling to Cloudflare (mitigated by S3-compatible API for portability).
- *Caveats:* None.

---

### [D7] API versioning and exposure strategy?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 4, 5, 6, 8 — all API-serving services and the frontend

**Context:** The PRD already establishes a consistent `/api/v1/{service-path}` pattern across all services. Both debaters agreed this is correct.

**Decision:** **Unified `/api/v1/{service-path}`** with ingress-level routing to each service. Each service owns its path segment under `/api/v1/`.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Consistent with PRD documentation. Single version prefix keeps client SDKs simple. Cloudflare Tunnel handles routing.
- *Negative:* None.
- *Caveats:* None.

---

### [D8] Self-hosted Signal-CLI or managed Signal gateway?

**Status:** Accepted

**Task Context:** Tasks 1, 7, 9 — Morgan's primary interaction channel

**Context:** No production-grade managed Signal gateway SaaS exists. Signal's protocol is intentionally closed to third-party gateways. Both debaters agreed on self-hosting but the Pessimist raised critical operational risk.

**Decision:** **Self-hosted Signal-CLI** as a sidecar pod to Morgan.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Only viable path. Already specified in infra YAML.
- *Negative:* Signal-CLI is a single point of failure for the platform's primary interaction channel (500+ concurrent Signal connections, Morgan handles 80%+ of customer inquiries).
- *Caveats (critical):* Signal-CLI's registration state is phone-number-bound and non-trivially restorable. **Tasks 1 and 9 MUST include**: (1) PVC-backed persistent storage for Signal-CLI registration state, (2) automated PVC snapshots on a schedule, (3) a documented re-registration runbook, (4) alerting on Signal-CLI pod restarts/crashes. Without these, the 99.9% uptime target is unachievable.

---

### [D9] Primary navigation paradigm for Sigma-1 website?

**Status:** Accepted

**Task Context:** Task 8 — Web frontend

**Context:** Both debaters agreed on the navigation pattern. Equipment catalog with 533+ products across 24 categories needs structured browsing.

**Decision:** **Top navigation bar (desktop) with bottom tab bar (mobile); sidebar filtering on catalog pages.**

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Proven e-commerce pattern. Horizontal nav provides space for primary sections (Equipment, Quote, Portfolio). Sidebar filtering matches catalog depth. shadcn/ui supports both patterns natively.
- *Negative:* None.
- *Caveats:* None.

---

### [D10] Data table component for equipment catalog and quote builder?

**Status:** Accepted

**Task Context:** Task 8 — Web frontend (equipment catalog, quote builder)

**Context:** The Optimist correctly identified this as a false dichotomy — shadcn/ui's data table component is built on TanStack Table. Both debaters agreed.

**Decision:** **shadcn/ui data table** (which wraps TanStack Table internally), providing both design consistency and advanced features (sorting, filtering, virtualization).

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Single component that delivers both shadcn/ui theming consistency and TanStack Table's full feature set.
- *Negative:* None — this is not an either/or decision.
- *Caveats:* None.

---

### [D11] Access control model for admin endpoints?

**Status:** Accepted

**Task Context:** Tasks 2, 4, 5, 10 — admin endpoints for product management, finance, vetting, and RBAC enforcement

**Context:** Small team (Perception Events is a lighting company, not an enterprise). Clear role boundaries with no fine-grained attribute-based requirements in the PRD.

**Decision:** **RBAC with three roles: admin, service, read-only.** Maps directly to Kubernetes RBAC patterns already in use.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Simple, well-understood pattern. Admin (Mike), service accounts (Morgan, inter-service), read-only (reporting). Kubernetes RBAC already established in cluster.
- *Negative:* Cannot express fine-grained attribute-based policies if needed in future.
- *Caveats:* ABAC can be layered on later if requirements evolve beyond three roles.

---

### [D12] Direct Stripe or payment orchestration platform?

**Status:** Accepted

**Task Context:** Task 4 — Finance Service

**Context:** PRD specifies Stripe. No multi-provider routing requirement exists. Both debaters agreed.

**Decision:** **Direct Stripe API integration** using `stripe-rust`.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Single provider, mature library (500k+ downloads). Multi-currency, invoicing, and payment intents natively supported.
- *Negative:* Vendor lock-in to Stripe.
- *Caveats:* If multi-provider routing is needed in future, an orchestration layer can be added then.

---

## 4. Escalated Decisions

### [D2] Should the Rust/Axum services (Catalog, Finance, Vetting) be separate deployments or consolidated? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 2, 4, 5 — all three Rust/Axum services assigned to agent Rex

**Options:**

- **Option A (Optimist): Separate services.** Finance and Vetting are distinct bounded contexts with zero shared domain logic. Independent scaling profiles (Finance: bursty on payment events; Vetting: bursty on lead intake). ArgoCD makes additional Deployment manifests trivial. Separate failure domains prevent cross-contamination.

- **Option B (Pessimist): Single Rust/Axum binary with internal route modules.** All three share the same agent (Rex), same stack, same database, same cache. At 500 concurrent connections total, independent scaling is theoretical. A single binary cuts deployment manifests 3→1, eliminates 2 connection pools, reduces on-call debugging surface. Module boundaries in code enforce separation without operational overhead.

**Optimist argued:** Zero shared domain logic between invoicing/payments and business verification. A Stripe webhook surge shouldn't affect vetting latency. "The deployment overhead of a second Rust binary on Kubernetes is one additional Deployment manifest — trivial with ArgoCD."

**Pessimist argued:** The architecture already ships three backend languages (Rust, Go, Node.js) for a single-operator business. Three separate Rust services means three CI pipelines, three container images, three Deployments, three health checks, three connection pools to the same PostgreSQL cluster. At 500 concurrent connections, this isn't Netflix.

**Recommendation:** The Pessimist raises a legitimate operational concern about the multi-language stack's debugging surface. However, this decision has moderate reversibility — services can be consolidated later (or split later) with Axum's modular routing. **If the team prioritizes operational simplicity**, consolidate into a single Rust binary with three route modules. **If the team prioritizes domain isolation and independent deployment**, keep them separate. The PRD's task structure assumes separate services (Tasks 2, 4, 5), so the default path is separate unless explicitly overridden. Implementing agents should await resolution before finalizing service packaging.

---

### [D5] How should PostgreSQL schemas be organized and isolated? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 1, 2, 3, 4, 5, 6, 9 — all services that write to PostgreSQL

**Options:**

- **Option A (Optimist): Separate schemas per domain within a single CloudNative-PG cluster.** Allows clean migration boundaries, independent `pg_dump`, and explicit cross-schema references where needed. Single cluster avoids 3x operational overhead.

- **Option B (Pessimist): Same as Option A, but with a hard rule: no cross-schema foreign keys.** Services reference other domains via API calls, not database joins. This prevents hidden deployment-ordering dependencies that break ArgoCD's independent sync model.

**Optimist argued:** Single cluster, separate schemas is operationally correct for 500 concurrent connections. Per-schema migrations and backups provide clean domain boundaries.

**Pessimist argued:** Cross-schema foreign keys create deployment-ordering landmines. When Finance references `rms.projects.id` and ArgoCD syncs services independently, a Finance migration can arrive before the RMS migration that adds the referenced column. "API-based cross-domain references are explicit, version-aware, and debuggable."

**Recommendation:** Both positions agree on separate schemas in a single cluster. The only disagreement is whether cross-schema FKs should be allowed. The Pessimist's argument about ArgoCD deployment ordering is technically sound. **Recommend Option B: separate schemas, no cross-schema foreign keys, cross-domain references via API.** This is the safer default for a GitOps-deployed system with independent service lifecycles. If implementing agents proceed before resolution, they should default to Option B (no cross-schema FKs) as it is strictly more conservative and does not prevent any PRD-specified functionality.

---

### [D6] Internal service-to-service authentication mechanism? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 1, 2, 3, 4, 5, 6, 7, 9, 10 — all inter-service communication

**Options:**

- **Option A (Optimist): Cilium-managed mTLS for transport security + JWT service tokens** (rotated via External Secrets) for application-layer identity. Defense-in-depth with network-layer mTLS and app-layer JWT.

- **Option B (Pessimist): Cilium network policies for transport security + Kubernetes projected ServiceAccount tokens** (bound, audience-scoped) for app-layer identity. Eliminates custom JWT issuance infrastructure. TokenReview API works identically from Rust, Go, and Node.js.

**Optimist argued:** Cilium already provides transparent mTLS. External Secrets operator handles automated rotation. JWT gives explicit application-layer identity verification. Defense-in-depth for GDPR compliance.

**Pessimist argued:** JWT service tokens require a token issuer, a validation library in every language (Rust, Go, Node.js), and rotation infrastructure — that's three JWT validation implementations. Kubernetes already issues short-lived, auto-rotated tokens per pod via projected volumes. TokenReview API works identically across all languages. "Fewer moving parts than custom JWT."

**Recommendation:** The Pessimist's concern about three-language JWT implementations is pragmatic but not blocking — JWT validation libraries exist and are mature in all three languages. However, the Pessimist's alternative (K8s projected ServiceAccount tokens) is genuinely simpler and Kubernetes-native. **Recommend Option B (Cilium + K8s ServiceAccount tokens)** as it provides equivalent security with less custom infrastructure. Both options meet the GDPR compliance requirement. If Cilium mTLS is available in the cluster's Cilium configuration, it should be enabled regardless of which app-layer mechanism is chosen.

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Version | Decision |
|-------|-----------|---------|----------|
| Cache | Valkey | 7.2-alpine | D1 — Resolved |
| Object Storage | Cloudflare R2 | S3-compatible API | D4 — Resolved |
| Database | PostgreSQL 16 via CloudNative-PG | Single cluster, separate schemas | D5 — Escalated (default: no cross-schema FKs) |
| Payment Processing | Stripe (direct) | via `stripe-rust` | D12 — Resolved |
| Backend (Catalog) | Rust 1.75+, Axum 0.7 | — | PRD-specified |
| Backend (RMS) | Go 1.22+, gRPC + grpc-gateway | — | PRD-specified |
| Backend (Finance) | Rust 1.75+, Axum 0.7 | — | PRD-specified |
| Backend (Vetting) | Rust 1.75+, Axum 0.7 | — | PRD-specified |
| Backend (Social) | Node.js 20+, Elysia 1.x, Effect 3.x | — | PRD-specified |
| AI Agent | OpenClaw with MCP tools | — | PRD-specified |
| Frontend | Next.js 15, React 19, shadcn/ui, TailwindCSS 4, Effect 3.x | — | PRD-specified |
| Signal Integration | Self-hosted Signal-CLI | Sidecar to Morgan | D8 — Resolved |
| Hosting (Web) | Cloudflare Pages | — | PRD-specified |
| Orchestration | Kubernetes + ArgoCD | — | PRD-specified |
| CDN/Ingress | Cloudflare Tunnel + CDN | — | PRD-specified |
| Observability | Grafana + Loki + Prometheus | — | PRD-specified |

### Service Architecture and Communication

```
                    ┌──────────────────┐
                    │    Cloudflare    │
                    │  Tunnel + CDN   │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼──────┐  ┌───▼───┐  ┌──────▼───────┐
     │  Next.js 15   │  │Morgan │  │  API Clients │
     │  (Blaze)      │  │(Angie)│  │              │
     └────────┬──────┘  └───┬───┘  └──────┬───────┘
              │              │              │
              │    REST /api/v1/*  (D3, D7) │
              └──────────────┼──────────────┘
                             │
        ┌────────┬───────────┼───────────┬──────────┐
        │        │           │           │          │
   ┌────▼────┐ ┌─▼──────┐ ┌─▼───────┐ ┌─▼──────┐ ┌▼──────────┐
   │Catalog  │ │  RMS   │ │Finance  │ │Vetting │ │Social     │
   │(Rust)   │ │ (Go)   │ │(Rust)   │ │(Rust)  │ │(Node.js)  │
   └────┬────┘ └──┬─────┘ └──┬──────┘ └──┬─────┘ └──┬────────┘
        │         │           │           │          │
        │    gRPC (service-to-service where needed)  │
        │         │           │           │          │
   ┌────▼─────────▼───────────▼───────────▼──────────▼──┐
   │              PostgreSQL 16 (CloudNative-PG)         │
   │  ┌─────────┬──────────┬──────────┬──────────┐      │
   │  │ catalog │   rms    │ finance  │ vetting  │ ...  │
   │  │ schema  │  schema  │  schema  │  schema  │      │
   │  └─────────┴──────────┴──────────┴──────────┘      │
   └─────────────────────────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  Valkey 7.2     │
                    │  (rate limit,   │
                    │   cache, session)│
                    └─────────────────┘
```

### Key Patterns

- **Morgan speaks REST uniformly** to all backend services via MCP tools (D3)
- **Internal service-to-service** communication may use gRPC where both sides support it (e.g., Catalog→RMS availability checks)
- **API versioning** follows unified `/api/v1/{service-path}` with ingress routing (D7)
- **Schema isolation** within a single PostgreSQL cluster (D5 — pending final approval, default to no cross-schema FKs)
- **RBAC** with three roles: admin, service, read-only (D11)
- **GitOps** via ArgoCD with automatic rollbacks

### Explicitly Ruled Out

| Ruled Out | Reason | Decision |
|-----------|--------|----------|
| Standard Redis (non-Valkey) | Valkey already specified in infra YAML, full API compatibility, LF governance | D1 |
| AWS S3 | Cloudflare R2 has zero egress, native CDN integration, cluster already runs Cloudflare Operator | D4 |
| Payment orchestration (Paddle, Adyen) | Single-provider use case, no multi-provider routing need | D12 |
| ABAC for admin access control | Overengineering for small team with 3 clear roles | D11 |
| Managed Signal gateway | No production-grade managed option exists; Signal's protocol intentionally closed | D8 |
| Per-service versioned paths (e.g., `/catalog/v1/`) | PRD already establishes unified `/api/v1/` pattern | D7 |

---

## 6. Implementation Constraints

### Security Requirements

- **GDPR compliance** is mandatory across all services: support data export and customer deletion
- **RBAC enforcement** with three roles: `admin` (Mike), `service` (Morgan, inter-service accounts), `read-only` (reporting) — (D11)
- **Internal auth mechanism** is escalated (D6) — implementing agents should default to Cilium network policies + K8s ServiceAccount tokens until resolved
- **Secret rotation** must be automated for all external service credentials via External Secrets operator or equivalent (Task 10)
- **Security scanning** blocks merge on critical/high severity issues (Cipher agent, CI/CD pipeline)
- **Audit logging** required for Kubernetes API and all managed services, integrated with Loki/Grafana

### Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| Morgan simple query response | < 10 seconds | PRD Constraints |
| Equipment availability check | < 500ms | PRD Constraints |
| Invoice generation | < 5 seconds | PRD Constraints |
| Concurrent Signal connections | 500+ | PRD Constraints |
| Quote-to-invoice workflow | < 2 minutes | Success Criteria |
| Service uptime | 99.9% (≤ 43 min/month downtime) | PRD Constraints |
| Code coverage | ≥ 80% | QA Workflow |

### Operational Requirements

- **Signal-CLI state persistence**: PVC-backed storage with automated snapshots and documented re-registration runbook (D8 caveats). This is a hard requirement for 99.9% uptime.
- **HA for PostgreSQL and Valkey**: Must be enabled in production (Task 9)
- **Observability**: All services must expose Prometheus metrics (`/metrics`), liveness (`/health/live`), and readiness (`/health/ready`) probes
- **GitOps**: All deployments via ArgoCD with automatic rollbacks on failure
- **Multi-language debugging surface**: The platform ships Rust, Go, Node.js, and TypeScript — operations team must maintain debugging capabilities across all four runtimes (Pessimist concern, acknowledged)

### Service Dependencies and Integration Points

| Service | Depends On | External APIs |
|---------|-----------|---------------|
| Equipment Catalog | PostgreSQL, Valkey, R2 | — |
| RMS | PostgreSQL, Valkey | Google Calendar API |
| Finance | PostgreSQL, Valkey, Stripe | Stripe API |
| Customer Vetting | PostgreSQL | OpenCorporates, LinkedIn, Google Reviews, Credit APIs |
| Social Engine | PostgreSQL, R2 | Instagram Graph, LinkedIn, Facebook Graph, OpenAI/Claude |
| Morgan | All backend services, Signal-CLI | ElevenLabs, Twilio |
| Website | Equipment Catalog API, Morgan (chat) | Cloudflare Pages |

### Organizational Preferences

- Prefer Cloudflare ecosystem services (R2, Pages, Tunnel, CDN) — cluster already runs Cloudflare Operator
- Prefer self-hosted solutions where no reliable managed alternative exists (Signal-CLI)
- Managed by 5D Labs — not self-hosted deployment

---

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`**: `true`
- **`frontendTargets`**: `web` and `mobile`
- **Mode**: `both` (design and implementation)
- **Provider mode**: `stitch`

### Design Generation Status

| Provider | Status | Notes |
|----------|--------|-------|
| Stitch | `generated` | Design artifacts generated successfully |
| Framer | `skipped` | Not requested |

### Supplied Design Artifacts and References

No explicit design artifacts (Figma files, screenshots, or reference URLs) were supplied in the design context beyond the Stitch generation. The PRD specifies the technology stack (shadcn/ui, TailwindCSS 4, React 19) which implicitly constrains the design system.

### Implications for Implementation

**Web (Task 8 — Blaze):**
- Next.js 15 App Router with React 19 and shadcn/ui provides the component foundation
- TailwindCSS 4 for styling with shadcn/ui's default design tokens
- Navigation: top nav bar (desktop) + bottom tab bar (mobile) + sidebar filtering on catalog pages (D9)
- Data tables: shadcn/ui DataTable wrapping TanStack Table for equipment catalog and quote builder (D10)
- Equipment catalog serves 533+ products across 24 categories — requires robust filtering, sorting, and potentially virtualized list rendering
- Self-service quote builder with Effect form validation
- Morgan web chat widget integration
- AI-native optimization: `/llms.txt`, `/llms-full`, Schema.org structured data
- Deployed to Cloudflare Pages

**Mobile:**
- PRD references Expo in the architecture diagram but no mobile-specific tasks were decomposed in Phase 1
- The web frontend should be responsive (bottom tab bar for mobile), which may serve as the initial mobile experience
- Mobile-specific app development appears to be deferred

### Stitch-Generated Design Candidates

Stitch design generation completed successfully. Implementing agents (particularly Blaze for Task 8) should reference any Stitch-generated design candidates when available in the design artifact store for visual direction on:
- Homepage hero and CTA layout
- Equipment catalog browse and filter patterns
- Product detail pages with availability calendar
- Quote builder multi-step flow
- Portfolio gallery layout

---

## 7a. Selected Design Direction

No design selections (`design_selections`) were provided. Implementing agents should follow shadcn/ui defaults and the navigation decisions resolved in D9 and D10, referencing Stitch-generated candidates where available.

---

## 7b. Design Deliberation Decisions

No design deliberation results (`design_deliberation_result`) were provided. Visual identity, design system, component library, layout patterns, and UX behavior decisions beyond D9 (navigation) and D10 (data table) are left to implementing agents' best judgment within the constraints of shadcn/ui + TailwindCSS 4.

---

## 8. Open Questions

The following items were not resolved in deliberation and are not blocking. Implementing agents should use their best judgment:

1. **Rust service consolidation (D2 — Escalated)**: Whether Catalog, Finance, and Vetting ship as three binaries or one modular binary. Default: follow the PRD's task structure (separate services) unless human decision directs consolidation.

