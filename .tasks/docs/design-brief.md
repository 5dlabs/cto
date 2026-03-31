# Enhanced PRD

## 1. Original Requirements

> # Project: Sigma-1 — Unified AI Business Platform
>
> ## Vision
>
> Sigma-1 is a comprehensive AI-powered business platform that replaces fragmented tools, manual processes, and administrative overhead with a single intelligent agent — **Morgan** — accessible through Signal, phone, and web. Built for Sigma-1 / Perception Events, a lighting and visual production company.
>
> Instead of juggling rental software, spreadsheets, phone calls, accounting tools, and social media apps, everything runs through one interface: send Morgan a message, and it handles the rest.
>
> This is a microservices architecture demonstrating full CTO platform agent utilization across multiple tech stacks, similar to the AlertHub pattern.
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
> |-------|---------|--------------
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

The initial task decomposition identified **10 tasks** spanning infrastructure, backend services, an AI agent, a web frontend, and production hardening.

### Task Inventory

| ID | Title | Agent | Stack | Priority | Dependencies |
|----|-------|-------|-------|----------|--------------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | High | None |
| 2 | Equipment Catalog Service API | Rex | Rust 1.75+, Axum 0.7 | High | Task 1 |
| 3 | RMS Service | Grizz | Go 1.22+, gRPC | High | Task 1 |
| 4 | Finance Service | Rex | Rust 1.75+, Axum 0.7 | High | Task 1 |
| 5 | Customer Vetting Service | Rex | Rust 1.75+, Axum 0.7 | High | Task 1 |
| 6 | Social Media Engine | Nova | Node.js 20+, Elysia 1.x, Effect TS | Medium | Task 1 |
| 7 | Morgan AI Agent | Angie | OpenClaw/MCP | High | Tasks 2, 3, 4, 5, 6 |
| 8 | Web Frontend | Blaze | React 19, Next.js 15, Effect, shadcn/ui, TailwindCSS 4 | High | Tasks 2, 7 |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | High | Tasks 2–8 |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit Logging | Bolt | Kubernetes/Helm | High | Task 9 |

### Key Services and Components

- **5 backend microservices**: Equipment Catalog (Rust), RMS (Go), Finance (Rust), Customer Vetting (Rust), Social Media Engine (Node.js)
- **1 AI agent**: Morgan (OpenClaw/MCP) — the central orchestrator for all customer and admin interactions
- **1 web frontend**: Next.js 15 website with equipment catalog, quote builder, chat widget, and portfolio
- **2 infrastructure/hardening tasks**: Core provisioning and production-grade security/HA

### Agent Assignments

- **Bolt**: Infrastructure provisioning and production hardening (Tasks 1, 9, 10)
- **Rex**: Three Rust/Axum services — Equipment Catalog, Finance, Customer Vetting (Tasks 2, 4, 5)
- **Grizz**: Go/gRPC Rental Management System (Task 3)
- **Nova**: Node.js/Elysia Social Media Engine (Task 6)
- **Angie**: Morgan AI Agent integration and MCP tool configuration (Task 7)
- **Blaze**: Next.js 15 web frontend (Task 8)

### Cross-Cutting Concerns

- **15 decision points** were identified across tasks, covering architecture, platform choices, API design, data models, security, UX, design systems, and build-vs-buy trade-offs
- All backend services share a single PostgreSQL cluster (CloudNative-PG) with schema-level isolation
- All services share a Valkey (Redis-compatible) cache for rate limiting, sessions, and caching
- Service-to-service authentication and secret management span all tasks
- API versioning strategy (`/api/v1/...`) applies uniformly to all public endpoints
- The QA pipeline (Stitch, Cleo, Tess, Cipher, Atlas, Bolt) applies to all code changes

---

## 3. Resolved Decisions

### [D2] Which Redis-compatible cache should be used?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4 (core infrastructure and all services needing caching)

**Context**: Both debaters agreed immediately. The Valkey operator (redis.redis.opstreelabs.in) is already available in-cluster with CRDs deployed. Valkey 7.2 is Redis wire-compatible and BSD-licensed.

**Decision**: Use the existing Valkey operator with Valkey 7.2 as specified in the PRD YAML (`valkey/valkey:7.2-alpine`).

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Single operator handles lifecycle management; no additional Helm charts; wire-compatible with all Redis client libraries across Rust, Go, and Node.js stacks
- **Negative**: None identified
- **Caveats**: None

---

### [D3] Which object storage provider should be used?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 6 (infrastructure, product images, social media photos)

**Context**: Both debaters agreed. R2 has zero egress fees, S3-compatible API, and native Cloudflare CDN integration. The Cloudflare tunnel operator is already deployed in-cluster.

**Decision**: Use Cloudflare R2 for all object storage (product images, social media photos, portfolio assets).

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Zero egress fees (vs S3's $0.09/GB); native Cloudflare CDN integration for Tasks 2, 6, 8; S3-compatible SDK means no code changes if migrating later
- **Negative**: None identified
- **Caveats**: None

---

### [D4] Which PostgreSQL operator?

**Status**: Accepted (Hard Constraint)

**Task Context**: Tasks 1, 2, 3, 4, 5, 6 (all services using PostgreSQL)

**Context**: Hard constraint per PRD. Both debaters agreed without discussion.

**Decision**: CloudNative-PG as specified in the PRD.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: CNCF-adopted, production-proven, handles backup/recovery natively
- **Negative**: None
- **Caveats**: None

---

### [D6] Single database with multiple schemas vs separate databases per service?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4, 5, 6 (all services sharing the PostgreSQL cluster)

**Context**: Both debaters agreed on single PostgreSQL cluster with schemas per domain (rms, crm, finance, audit, public) as specified in the PRD. The Pessimist raised a valid caveat about cross-schema coupling.

**Decision**: Single PostgreSQL cluster with schemas per domain.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: One backup strategy, one failover; cross-schema access available for reporting; simpler operations than distributed databases
- **Negative**: Cross-schema joins create implicit coupling between services
- **Caveats (Pessimist)**: **Mitigate cross-schema coupling by using views at schema boundaries. Never join raw tables cross-schema.** Schema migrations in one service (e.g., RMS) can break queries in another (e.g., Finance) if raw tables are joined directly.

---

### [D8] RBAC or ABAC for web frontend and Morgan agent?

**Status**: Accepted

**Task Context**: Tasks 7, 8, 10 (Morgan agent, web frontend, production hardening)

**Context**: Both debaters agreed. The PRD describes exactly two human personas (Mike as admin, customers) plus Morgan as agent. Four roles cover all access patterns.

**Decision**: RBAC with a small, well-defined role set: `admin`, `operator`, `customer`, `agent`.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Simple role checks across all services; no policy engine overhead (OPA/Cedar); well-matched to the actual user base
- **Negative**: May need to extend to ABAC later if role requirements grow
- **Caveats**: Only extend to ABAC if requirements demand it — YAGNI applies

---

### [D9] Self-hosted Signal-CLI or third-party relay?

**Status**: Accepted (Hard Constraint)

**Task Context**: Tasks 1, 7 (infrastructure provisioning, Morgan agent)

**Context**: Hard constraint per PRD. No reliable third-party Signal SaaS exists. The Pessimist agreed but raised an operational risk.

**Decision**: Self-host Signal-CLI as a sidecar pod alongside Morgan.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Open-source, well-maintained; sidecar pattern keeps latency low for Morgan's 10-second response requirement
- **Negative**: Signal-CLI is a Java process that holds registration state
- **Caveats (Pessimist)**: **Task 1 MUST provision a PVC for Signal-CLI state.** If the pod crashes and restarts without persistent storage for the Signal identity, the registered phone number is lost. An operational runbook is needed for Signal-CLI deregistration events after updates.

---

### [D10] API versioning strategy?

**Status**: Accepted (Hard Constraint)

**Task Context**: Tasks 2, 3, 4, 5, 6 (all public-facing API services)

**Context**: Hard constraint per PRD. Both debaters agreed without discussion.

**Decision**: URI-based versioning (`/api/v1/...`) for all public endpoints.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Explicit, discoverable, and matches the endpoint patterns already defined in the PRD
- **Negative**: None
- **Caveats**: None

---

### [D11] Sidebar or top navigation for the web frontend?

**Status**: Accepted

**Task Context**: Task 8 (web frontend)

**Context**: Only the Optimist raised this point; no counter-position was offered. The Optimist argued top navigation is standard for customer-facing catalog/marketing sites, with contextual sidebar filters on the equipment browse page.

**Decision**: Top navigation bar with responsive hamburger menu for mobile. Equipment catalog uses sidebar filters contextually (page-specific, not global navigation).

**Consensus**: Uncontested (Optimist position, no objection from Pessimist)

**Consequences**:
- **Positive**: Matches customer-facing site conventions; avoids admin-panel UX feel; sidebar filters enhance equipment browse UX
- **Negative**: None identified
- **Caveats**: None

---

### [D12] shadcn/ui as-is, extended, or custom?

**Status**: Accepted

**Task Context**: Task 8 (web frontend)

**Context**: Both debaters agreed. A lighting/production company needs brand differentiation beyond defaults. 80% shadcn/ui primitives, 20% custom branded components.

**Decision**: Extend shadcn/ui with custom branded components (theme tokens, hero sections, quote builder).

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Accessible, composable primitives from shadcn/ui; brand differentiation via custom color palette, typography, and hero layouts; fast development for Task 8
- **Negative**: Custom components require additional design and maintenance effort
- **Caveats**: None

---

### [D13] Which data table component for equipment catalog and admin views?

**Status**: Accepted

**Task Context**: Task 8 (web frontend — equipment catalog with 533+ products)

**Context**: Both debaters agreed. TanStack Table is headless and provides filtering, sorting, pagination, and virtual scrolling. shadcn/ui provides the visual layer.

**Decision**: TanStack Table with shadcn/ui Table styling.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Full control over rendering; handles 533+ products with virtual scrolling; de facto standard (24k+ GitHub stars)
- **Negative**: None identified
- **Caveats**: None

---

### [D14] External Secrets Operator with automated rotation or static secrets?

**Status**: Accepted

**Task Context**: Tasks 1, 10 (infrastructure provisioning, production hardening)

**Context**: Both debaters agreed. ESO CRDs are already present and operational in-cluster. Static secrets are a compliance liability.

**Decision**: Use the already-deployed External Secrets Operator with automated rotation.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Transparent rotation for Stripe keys, API tokens, DB credentials; compliance-friendly; already operational
- **Negative**: None identified
- **Caveats (Pessimist)**: **Verify that a SecretStore backend is actually configured (e.g., Vault, AWS SSM), not just the CRDs installed.** Task 1 should validate this during provisioning.

---

### [D15] Direct Stripe API integration or payment gateway abstraction?

**Status**: Accepted

**Task Context**: Task 4 (Finance Service)

**Context**: Both debaters agreed. The PRD names Stripe exclusively with no mention of alternative providers. YAGNI on the abstraction layer.

**Decision**: Direct Stripe API integration using the official `stripe-rust` crate.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Direct access to Stripe's invoicing, subscription, and webhook APIs; no unnecessary abstraction layer; well-maintained official crate
- **Negative**: If a second payment provider is needed, an interface will need to be extracted
- **Caveats**: Extract an interface later only if requirements change — not preemptively

---

## 4. Escalated Decisions

### [D1] Should inter-service communication be purely synchronous REST/gRPC or include asynchronous event-driven messaging via NATS? — ESCALATED

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 6, 7 (all backend services and Morgan agent)

**Options**:
- **Option A (Optimist)**: Hybrid — synchronous gRPC/REST for queries (availability checks, catalog lookups) and NATS JetStream for event-driven workflows (lead→vet→score→opportunity, invoice→payment, social publish pipeline). NATS is already deployed in-cluster at `openclaw-nats.openclaw.svc.cluster.local`.
- **Option B (Pessimist)**: Synchronous REST/gRPC for all service-to-service calls in financial and rental workflows (Tasks 2–5, 7). NATS only for the social media pipeline (Task 6) where eventual consistency is acceptable.

**Optimist argued**: The quote-to-invoice flow (DF-2) crosses 3 services; coupling them synchronously means a Finance Service outage blocks quoting. NATS JetStream provides at-least-once delivery, replay, and decoupling. NATS is already deployed and running. Tasks 3, 4, 5, 6, 7 all participate in multi-step workflows that benefit from event decoupling.

**Pessimist argued**: This is a small business platform (~5 services, single cluster, one primary user) not a distributed systems showcase. NATS JetStream at-least-once delivery requires every consumer to be idempotent. The quote→invoice flow is financial and must have clear success/failure semantics. Without dead-letter queue strategies, consumer lag alerting, and idempotency keys (none of which appear in any task description), the risk of silent data loss or duplicate invoices is real. With 2 replicas and circuit breakers, synchronous calls are simpler to operate and debug. Critical question: how does Morgan respond to users within 10 seconds if the answer depends on an async event that hasn't been consumed yet?

**Recommendation**: The Pessimist's position is more operationally conservative and better matched to the platform's scale. However, the Optimist correctly identifies that NATS is already deployed and that multi-step workflows benefit from decoupling. A pragmatic middle ground:

1. **V1**: Synchronous REST/gRPC for all financial and rental workflows (quote, invoice, payment, vetting). NATS for the social media pipeline (Task 6) only.
2. **V1.1**: After operational maturity, add NATS for non-financial event workflows (lead score notifications, portfolio sync triggers) with explicit dead-letter queues and idempotency keys.
3. **Prerequisite for any NATS expansion**: Dead-letter queue strategy, consumer lag monitoring, and idempotency keys must be designed as a sub-task before NATS is used for financial flows.

**The human should decide**: full hybrid from day one (Option A) or conservative synchronous-first (Option B, recommended).

---

### [D5] What API paradigm for Morgan's tool-server to backend services? — ESCALATED

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 6, 7 (tool-server communicating with all backends)

**Options**:
- **Option A (Optimist)**: gRPC for tool-server→backend communication, with REST via grpc-gateway for external/web consumers. Protobuf gives compile-time safety across Rust/Go/Node boundaries.
- **Option B (Pessimist)**: REST (HTTP/JSON) for tool-server→backend calls. Use gRPC only for the RMS service (Task 3) which already has it.

**Optimist argued**: MCP tools are typed function calls — a perfect fit for gRPC's Protobuf contracts. gRPC provides compile-time safety across the polyglot stack (Rust/Go/Node). The RMS (Task 3) already uses gRPC. Protobuf is 5–10x faster than JSON serialization.

**Pessimist argued**: Three of five backend services are Rust/Axum REST (Tasks 2, 4, 5) and one is Node/Elysia REST (Task 6). Adding gRPC to all of them doubles the API surface for no operational gain. MCP tools are JSON-based. The tool-server is already a translation layer that serializes/deserializes JSON. The "5–10x faster" Protobuf claim is irrelevant when the bottleneck is LLM inference at seconds, not microseconds of serialization.

**Recommendation**: The Pessimist's position is more pragmatic. The tool-server is a translation layer between JSON-based MCP tool schemas and backend services. Adding gRPC to services that are natively REST (Tasks 2, 4, 5, 6) increases implementation scope for each of those tasks with minimal benefit — the LLM inference time dwarfs serialization overhead. The recommended approach:

- Tool-server calls RMS (Task 3) via **gRPC** (native to that service)
- Tool-server calls all other services via **REST/JSON** (native to those services)
- Web consumers use the REST endpoints directly
- If a unified Protobuf contract is desired later, it can be added incrementally

**The human should decide**: universal gRPC (Option A) or native-protocol-per-service (Option B, recommended).

---

### [D7] mTLS or JWT for service-to-service auth? — ESCALATED

**Status**: Pending human decision

**Task Context**: Tasks 1, 2, 3, 4, 5, 6, 7 (all service-to-service communication)

**Options**:
- **Option A (Optimist)**: mTLS via Cilium's built-in mutual authentication, supplemented by service identity headers for authorization context.
- **Option B (Pessimist)**: Start with Kubernetes NetworkPolicies (Cilium-backed) for service isolation + shared pre-shared keys (PSK) in headers for v1. Upgrade to Cilium mTLS when the mesh is operationally validated.

**Optimist argued**: Cilium is already deployed in-cluster (CRDs present). Cilium's mTLS is transparent to application code — no JWT validation middleware needed in each Rust/Go/Node service. This reduces per-service auth complexity.

**Pessimist argued**: Having Cilium CRDs installed is not the same as having mTLS enabled. Cilium mTLS requires enabling mutual authentication mode, managing SPIFFE identities, and understanding the certificate lifecycle. Misconfigured mTLS fails closed — services can't talk to each other. For v1 with 5 internal services in a single namespace, NetworkPolicies + PSK provides defense-in-depth without the operational surface area.

**Recommendation**: The Pessimist's phased approach reduces risk for initial delivery. Cilium NetworkPolicies are well-understood and immediately deployable. PSK headers provide a lightweight authentication mechanism. The recommended approach:

1. **V1**: Cilium-backed NetworkPolicies for service isolation + PSK headers for service identity
2. **V1.1**: Enable Cilium mTLS mutual authentication after operational validation of the Cilium mesh configuration
3. Task 1 should include a sub-task to verify Cilium's mutual authentication readiness (SPIFFE identity issuance, certificate lifecycle)

**The human should decide**: Cilium mTLS from day one (Option A) or phased NetworkPolicies+PSK first (Option B, recommended).

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Version | Decision Source |
|-------|-----------|---------|-----------------|
| Database | PostgreSQL via CloudNative-PG | 16 | D4 (hard constraint) |
| Cache | Valkey via Opstree operator | 7.2 | D2 (unanimous) |
| Object Storage | Cloudflare R2 | — | D3 (unanimous) |
| Web Framework (Rust services) | Axum | 0.7 | PRD constraint |
| Web Framework (Go service) | gRPC + grpc-gateway | — | PRD constraint |
| Web Framework (Node service) | Elysia + Effect | 1.x / 3.x | PRD constraint |
| Frontend Framework | Next.js 15, React 19 | 15 / 19 | PRD constraint |
| UI Components | shadcn/ui (extended) + TanStack Table | — | D12, D13 (unanimous) |
| Styling | TailwindCSS | 4 | PRD constraint |
| Type System (frontend/Node) | Effect + TypeScript | 3.x / 5.x | PRD constraint |
| AI Agent | OpenClaw (Morgan) | — | PRD constraint |
| Payment Processing | Stripe (direct, `stripe-rust` crate) | — | D15 (unanimous) |
| Signal Integration | Signal-CLI (self-hosted sidecar) | — | D9 (hard constraint) |
| Secret Management | External Secrets Operator | — | D14 (unanimous) |
| CDN / Hosting | Cloudflare Pages + Cloudflare Tunnel | — | PRD constraint |
| Observability | Grafana + Loki + Prometheus | — | PRD constraint (existing stack) |
| Access Control | RBAC (admin, operator, customer, agent) | — | D8 (unanimous) |
| API Versioning | URI-based (`/api/v1/...`) | — | D10 (hard constraint) |

### Service Architecture

**Single PostgreSQL cluster** with domain-specific schemas (`rms`, `crm`, `finance`, `audit`, `public`). Cross-schema access uses **views at schema boundaries** — never raw table joins (per D6 caveat).

**5 backend microservices** communicate synchronously for v1 (pending D1 human decision):
- Equipment Catalog (Rust/Axum) — REST
- RMS (Go/gRPC) — gRPC natively, REST via grpc-gateway
- Finance (Rust/Axum) — REST
- Customer Vetting (Rust/Axum) — REST
- Social Media Engine (Node/Elysia) — REST (NATS for publish pipeline per both debaters)

**Morgan AI Agent** (OpenClaw) orchestrates all workflows through a tool-server that translates MCP tool calls into backend API calls. The tool-server's protocol per backend is pending D5 human decision (recommended: native protocol per service).

**Web Frontend** (Next.js 15) deployed to Cloudflare Pages with top navigation, extended shadcn/ui components, and TanStack Table for the equipment catalog.

### Key Patterns

- **Schema-per-domain** in a single PostgreSQL cluster with view-based boundaries
- **URI-based API versioning** across all services
- **RBAC with 4 roles** for all access control
- **ESO-managed secrets** with automated rotation
- **Sidecar pattern** for Signal-CLI alongside Morgan
- **GitOps deployment** via ArgoCD with automatic rollbacks

### Explicitly Ruled Out

- **ABAC/policy engines** (OPA, Cedar) — unnecessary for <5 roles (D8)
- **Payment gateway abstraction** — YAGNI; Stripe is the only provider (D15)
- **Separate PostgreSQL databases per service** — single cluster with schemas is sufficient (D6)
- **Bitnami Redis Helm chart** — Valkey operator is already deployed (D2)
- **AWS S3** — R2 is S3-compatible with zero egress and native Cloudflare integration (D3)
- **Trading Desk (Python)** — explicitly Phase 2 per PRD
- **Multi-region deployment** — single cluster per PRD non-goals
- **Custom component library from scratch** — extend shadcn/ui instead (D12)

---

## 6. Implementation Constraints

### Security Requirements

- **RBAC enforcement**: All services must implement role checks for `admin`, `operator`, `customer`, `agent` roles
- **Service-to-service auth**: Pending D7 decision — implement whichever approach is selected (mTLS or NetworkPolicies+PSK); do not ship services without inter-service authentication
- **Secret management**: All API keys, DB credentials, and tokens must be managed via External Secrets Operator — no static secrets in manifests. **Verify SecretStore backend is configured, not just CRDs installed** (D14 caveat)
- **Signal-CLI state persistence**: A PVC must be provisioned for Signal-CLI registration state. Loss of persistent state means loss of the registered phone number (D9 caveat)
- **Security scanning**: Critical/high severity issues block merge (Cipher agent in CI pipeline)
- **GDPR compliance**: Data export and customer deletion capabilities required across all services

### Performance Targets

- Morgan response time: **< 10 seconds** for simple queries
- Equipment availability check: **< 500ms** (95th percentile)
- Invoice generation: **< 5 seconds**
- Quote-to-invoice workflow: **< 2 minutes** end-to-end
- Signal concurrent connections: **500+**

### Operational Requirements

- **99.9% uptime** for production services
- **Minimum 80% code coverage** enforced by Tess agent in CI
- **HA for databases and services** in production (Task 9)
- **GitOps with ArgoCD** for deployment; automatic rollbacks on failure
- **Observability**: All services must expose Prometheus metrics at `/metrics` and health probes at `/health/live` and `/health/ready`
- **Cloudflare CDN**: Static assets served with < 100ms latency

### Service Dependencies and Integration Points

| Service | Depends On | External APIs |
|---------|-----------|---------------|
| Equipment Catalog | PostgreSQL, Valkey, R2 | — |
| RMS | PostgreSQL, Valkey | Google Calendar API |
| Finance | PostgreSQL, Valkey, Stripe | Stripe API |
| Customer Vetting | PostgreSQL | OpenCorporates, LinkedIn, Google Reviews, Credit APIs |
| Social Media Engine | PostgreSQL, R2 | Instagram Graph, LinkedIn, Facebook Graph, OpenAI/Claude |
| Morgan Agent | All backend services, Signal-CLI | ElevenLabs, Twilio |
| Website | Equipment Catalog API, Morgan (chat) | Cloudflare Pages |

### Organizational Preferences

- Prefer **self-hosted / in-cluster services** when available (Signal-CLI, Valkey operator, ESO, CloudNative-PG)
- Prefer **Cloudflare ecosystem** for CDN, hosting, tunnels, and object storage
- Prefer **existing OpenClaw observability stack** (Grafana, Loki, Prometheus)
- Cross-schema database access through **views only** — no raw cross-schema joins

---

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`**: `true`
- **`frontendTargets`**: `web` and `mobile`
- The PRD specifies a Next.js 15 website (Task 8) and lists an Expo mobile client in the architecture diagram, though mobile is not detailed as a Phase 1 task

### Supplied Design Artifacts

- No design artifacts (mockups, Figma files, brand guidelines) were supplied in the design context
- No reference URLs were provided for crawling

### Stitch Generation Status

- **Status**: `failed`
- **Reason**: Not specified (empty reason field)
- No Stitch-generated design candidates are available

### Implications for Implementation

1. **Web (Task 8)**: The frontend must be built using the resolved design system decisions:
   - Extended shadcn/ui with custom branded components (D12)
   - TanStack Table for equipment catalog data display (D13)
   - Top navigation with responsive hamburger menu; contextual sidebar filters on equipment page (D11)
   - TailwindCSS 4 for styling
   - No visual references are available — Blaze agent should establish brand identity (color palette, typography, hero layouts) as the first sub-task of Task 8, with Mike's approval before proceeding

2. **Mobile (Expo)**: Listed in architecture but not decomposed into a Phase 1 task. Implementing agents should not build mobile unless explicitly instructed. The REST API design should remain mobile-friendly (JSON, versioned endpoints).

3. **Brand Identity Gap**: With no supplied design artifacts and failed Stitch generation, the web frontend lacks visual direction. **Task 8 should include an explicit design spike** to establish:
   - Color palette appropriate for a lighting/visual production company
   - Typography system
   - Hero section layouts
   - Component theme tokens for shadcn/ui customization

### 7a. Selected Design Direction

No design selections were provided (`design_selections` not present).

### 7b. Design Deliberation Decisions

No design deliberation results were provided (`design_deliberation_result` not present).

---

## 8. Open Questions

The following items were not resolved in deliberation and are not escalated — implementing agents should use their best judgment:

1. **Signal-

