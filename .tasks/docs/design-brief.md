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

The initial task decomposition identified **10 tasks** spanning infrastructure, backend services, AI agent orchestration, frontend, and production hardening.

| Task ID | Title | Agent | Stack | Priority | Dependencies |
|---------|-------|-------|-------|----------|--------------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | High | None |
| 2 | Equipment Catalog Service API | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 3 | RMS Service | Grizz | Go 1.22+/gRPC | High | Task 1 |
| 4 | Finance Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 5 | Customer Vetting Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 6 | Social Media Engine | Nova | Node.js 20+/Elysia + Effect | Medium | Task 1 |
| 7 | Morgan AI Agent | Angie | OpenClaw/MCP | High | Tasks 2–6 |
| 8 | Website Frontend | Blaze | Next.js 15/React 19/Effect | High | Tasks 2, 7 |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | High | Tasks 2–8 |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit Logging | Bolt | Kubernetes/Helm | High | Task 9 |

### Key Services and Components

- **Infrastructure layer**: CloudNative-PG PostgreSQL, Valkey (Redis-compatible), Cloudflare R2, Cloudflare Tunnel, Signal-CLI, Grafana/Loki/Prometheus observability stack
- **Backend services** (4 languages/runtimes): Equipment Catalog (Rust), RMS (Go), Finance (Rust), Customer Vetting (Rust), Social Media Engine (Node.js)
- **AI orchestration**: Morgan agent (OpenClaw) with MCP tool-server mediating 11 backend tools
- **Frontend**: Next.js 15 website with equipment catalog, quote builder, portfolio, Morgan web chat
- **Production hardening**: HA scaling, CDN/TLS via Cloudflare, RBAC, secret rotation, audit logging

### Agent Assignments

| Agent | Tasks | Technology |
|-------|-------|------------|
| Bolt | 1, 9, 10 | Kubernetes, Helm, Cloudflare Terraform |
| Rex | 2, 4, 5 | Rust 1.75+, Axum 0.7 |
| Grizz | 3 | Go 1.22+, gRPC, grpc-gateway |
| Nova | 6 | Node.js 20+, Elysia 1.x, Effect 3.x |
| Angie | 7 | OpenClaw, MCP tools |
| Blaze | 8 | Next.js 15, React 19, shadcn/ui, TailwindCSS 4, Effect 3.x |

### Cross-Cutting Concerns

- **14 decision points** were identified across tasks, covering platform choices, architecture patterns, API paradigms, data modeling, security, service topology, UX behavior, design system, and component library selection
- **Shared infrastructure** (PostgreSQL, Valkey, R2, Cloudflare) is consumed by all backend tasks via a `sigma1-infra-endpoints` ConfigMap
- **Observability**: All services emit Prometheus metrics and structured logs collected by the existing Grafana/Loki/Prometheus stack
- **QA pipeline**: 6 automated agents (Stitch, Cleo, Tess, Cipher, Atlas, Bolt) enforce code review, quality, testing, security, merge gates, and deployment

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine should be used for caching, rate limiting, and session storage?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2 (Catalog), Task 3 (RMS), Task 4 (Finance)

**Context**: Both debaters immediately agreed. The Valkey operator (`redis.redis.opstreelabs.in`) is already deployed in-cluster, running Valkey 7.2 (API-compatible Redis fork maintained by the Linux Foundation). Deploying a second caching topology provides no functional benefit.

**Decision**: Use the existing Valkey operator as the Redis-compatible cache for all services.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Zero provisioning cost; single caching topology to manage; Valkey passes the full Redis test suite; aligns with organizational preference for self-hosted services
- **Negative**: None identified
- **Caveats**: None — unanimous agreement

---

### [D2] Which object storage provider should be used for product images and social media photos?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2 (Catalog), Task 6 (Social Engine)

**Context**: Both debaters agreed. R2 has zero egress fees, S3-compatible API, and the Cloudflare operator is already in-cluster providing native integration for tunnels and DNS management.

**Decision**: Cloudflare R2 as the primary S3-compatible object storage.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Zero egress fees for read-heavy public assets; existing Cloudflare ecosystem integration; S3-compatible SDK works unchanged
- **Negative**: None identified
- **Caveats**: None — unanimous agreement

---

### [D3] How should service-to-service communication be handled between backend services?

**Status**: Accepted

**Task Context**: Task 2 (Catalog), Task 3 (RMS), Task 4 (Finance), Task 5 (Vetting), Task 6 (Social Engine), Task 7 (Morgan)

**Context**: This was the most contentious decision. The Optimist argued for a hybrid approach using synchronous HTTP/gRPC for request-reply flows plus NATS JetStream for event-driven async communication, citing NATS already being deployed in-cluster and the natural fit for flows like "opportunity converted → generate invoice." The Pessimist argued forcefully for direct synchronous calls only, citing: (1) at ~20 quotes/day, async decoupling solves a problem that doesn't exist; (2) NATS JetStream adds an async gap that makes debugging 4-hop flows significantly harder; (3) a synchronous 503 with retry is simpler to reason about than a message in a JetStream queue with unclear delivery semantics; (4) significant cognitive overhead for operating NATS for business-critical flows.

**Decision**: Synchronous HTTP/gRPC for request-reply flows, with NATS for event-driven async communication where decoupling matters.

**Consensus**: The Optimist's position prevailed based on the existing NATS deployment in-cluster and the PRD's data flow patterns requiring both patterns. However, the Pessimist's concerns about operational complexity are recorded as binding caveats below.

**Consequences**:
- **Positive**: Leverages existing NATS deployment at zero additional infrastructure cost; provides resilience for cross-service event flows (RMS→Finance, Social Engine events); enables independent deployment of services without breaking synchronous callers
- **Negative**: Introduces async debugging complexity; requires dead-letter queue strategy, retry policies, and consumer failure monitoring
- **Caveats (from Pessimist — these are binding operational requirements)**:
  - Every NATS JetStream subject must have an explicit retry count, dead-letter queue, and alerting on consumer lag
  - When a consumer fails (e.g., Finance consumer can't reach Stripe), the failure must be surfaced within 60 seconds via Prometheus/Grafana alerts
  - Morgan's synchronous request-reply flows (e.g., catalog search, availability check) MUST use direct HTTP/gRPC — NATS is only for fire-and-forget or eventual-consistency flows
  - Implementing agents must document which flows use NATS vs. direct calls and justify each choice

---

### [D4] What API paradigm should be used for the public-facing Equipment Catalog and RMS APIs?

**Status**: Accepted

**Task Context**: Task 2 (Catalog), Task 3 (RMS), Task 8 (Website)

**Context**: Both debaters agreed with the PRD's explicit design. REST/JSON for the browser-facing Equipment Catalog is the right choice for browser-consumable, cacheable endpoints. gRPC with grpc-gateway for the operational RMS provides proto-based contracts and type safety for internal workflows while maintaining REST compatibility for Morgan's MCP tools.

**Decision**: REST/JSON for Equipment Catalog (Task 2); gRPC with grpc-gateway for RMS (Task 3) — as specified in the PRD.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Matches consumer profiles — browsers get REST, internal services get gRPC type safety; grpc-gateway provides REST compatibility for Morgan; follows PRD's explicit design intent
- **Negative**: Two API paradigms to maintain across the platform
- **Caveats**: None — unanimous agreement, PRD-prescribed

---

### [D5] Should all services share a single PostgreSQL cluster with multiple schemas, or separate instances?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2 (Catalog), Task 3 (RMS), Task 4 (Finance), Task 5 (Vetting), Task 6 (Social Engine)

**Context**: Both debaters agreed on a single CloudNative-PG cluster with separate schemas per service. The key disagreement was on cross-schema foreign keys. The Optimist proposed allowing them "only where explicitly needed (rms→finance for invoice references)." The Pessimist argued this creates deployment coupling and prevents independent schema migration, advocating for strict prohibition with UUID-based cross-service references resolved at the application layer.

**Decision**: Single CloudNative-PG cluster with separate schemas per service (rms, crm, finance, audit, public), schema-scoped database users per service, **strict prohibition on cross-schema foreign keys**. Services reference each other by UUID, resolved at the application layer.

**Consensus**: 2/2 on single cluster; the Pessimist's stricter constraint on cross-schema FKs is adopted because it preserves the option to split schemas into independent clusters later without data migration.

**Consequences**:
- **Positive**: 1 cluster to manage instead of 4–6 (memory, backups, operational surface); schema-level isolation with distinct DB roles; preserves future independence for schema extraction
- **Negative**: Application-layer joins for cross-service data; slightly more complex query patterns for reports spanning RMS and Finance data
- **Caveats**: If a report requires joining across schemas (e.g., project profitability combining RMS project data and Finance invoice data), this must be implemented as an application-level aggregation, not a database view or cross-schema query

---

### [D6] What authentication and authorization mechanism should be used?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2–8 (all services), Task 10 (RBAC/Audit)

**Context**: Both debaters agreed. JWT for external APIs is standard and well-supported by Axum, Go middleware, and Next.js. Cilium is already deployed in-cluster providing L3/L4 identity-based network policies. Full mTLS with certificate rotation was deemed operationally expensive for a single-tenant platform with no proportional security benefit.

**Decision**: JWT-based authentication for external API access (website, Morgan web chat); Cilium-enforced network policies for internal service-to-service identity and isolation; NATS credential-based auth for async messaging.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Leverages existing Cilium deployment; JWT is well-supported across all framework stacks; no certificate rotation overhead
- **Negative**: Internal service communication relies on network-level identity rather than application-level cryptographic identity
- **Caveats**: If Sigma-1 ever moves to multi-tenant, mTLS should be revisited

---

### [D7] How should the Signal integration for Morgan be implemented?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 7 (Morgan)

**Context**: Hard constraint from the PRD: "self-hosted is mandated for privacy and control." Both debaters agreed without debate.

**Decision**: Signal-CLI self-hosted as a sidecar or separate pod.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Full privacy and GDPR control; only production-grade open-source option for Signal integration
- **Negative**: Signal-CLI is a Java process with fragile state management (single registration per phone number)
- **Caveats (critical, raised by Pessimist)**: Signal-CLI maintains a single registration per phone number. If the pod restarts and re-registers, message history may be lost. **A persistence strategy for Signal-CLI's state directory is mandatory** — this must be a PersistentVolumeClaim, not ephemeral storage. Implementing agents must address this in Task 1 and Task 7.

---

### [D8] Should Finance, Customer Vetting, and Equipment Catalog be separate microservices or merged?

**Status**: Accepted

**Task Context**: Task 2 (Catalog), Task 4 (Finance), Task 5 (Vetting)

**Context**: The Optimist argued for separate microservices citing different operational profiles (Catalog is read-heavy, Finance handles Stripe webhooks, Vetting makes slow external API calls) and fault isolation. The Pessimist argued for a Rust workspace monolith with modular crate boundaries, citing that separate Tokio task pools solve the isolation problem within a single process, and the operational overhead of 6 pods for 533 products serving one company is disproportionate.

**Decision**: Deploy as separate microservices, one per domain.

**Consensus**: The Optimist's position prevailed. The PRD's Kubernetes YAML explicitly models separate deployments with 2 replicas each. The services have fundamentally different failure modes (Stripe timeouts in Finance should not affect Catalog availability queries), and the PRD's <500ms availability check constraint is easier to guarantee with independent resource pools.

**Consequences**:
- **Positive**: Independent scaling and deployment; fault isolation between domains; aligns with PRD's explicit Kubernetes resource definitions; shared Rust workspace crate for common code without shared processes
- **Negative**: Higher pod count (6 pods for 3 Rust services); more deployment configurations to manage; more health checks and runbooks
- **Caveats (from Pessimist)**: Monitor actual resource utilization after deployment. If all three Rust services are idle 95% of the time at Sigma-1's scale, consolidation should be revisited in a future phase

---

### [D9] Which CDN and TLS termination solution should be used?

**Status**: Accepted

**Task Context**: Task 8 (Website), Task 9 (Production Hardening)

**Context**: Both debaters agreed. The Cloudflare operator is already in-cluster, Cloudflare Tunnel CRDs exist, and the PRD specifies Cloudflare Pages for the website. Running a parallel NGINX ingress creates two ingress paths to maintain.

**Decision**: Cloudflare CDN and TLS termination via Cloudflare Tunnel.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: DDoS protection, global edge caching, Argo Smart Routing; single ingress path; existing operator integration
- **Negative**: Vendor lock-in to Cloudflare for edge infrastructure
- **Caveats**: None — unanimous agreement

---

### [D10] How should Morgan orchestrate backend service actions?

**Status**: Accepted

**Task Context**: Task 7 (Morgan)

**Context**: Both debaters agreed on the MCP tool-server pattern. The key nuance was the Pessimist's insistence that the tool-server must be a **thin stateless proxy** — validate, log, forward — with zero business logic, to avoid creating a single point of failure that becomes a God service.

**Decision**: MCP tool-server abstraction mediating all backend actions, implemented as a thin stateless routing layer (validate, log, forward). Zero business logic in the tool-server itself.

**Consensus**: 2/2 (100% on the pattern; Pessimist's simplification constraint adopted)

**Consequences**:
- **Positive**: Single observability chokepoint for all Morgan actions; enables audit logging, rate limiting, and tracing in one place; consistent with existing `cto/cto-tools` pattern in-cluster
- **Negative**: Additional hop for every Morgan action; if tool-server is down, Morgan has zero backend capability
- **Caveats**: Morgan MUST degrade gracefully to "I can't help right now, please try again shortly" if the tool-server is unavailable. The tool-server must NOT accumulate business logic over time — this must be enforced in code review.

---

### [D11] What approach should be used for audit logging and compliance?

**Status**: Accepted

**Task Context**: Task 10 (Audit Logging), Task 7 (Morgan — audit trail for AI actions)

**Context**: The Optimist proposed services emitting structured audit events to NATS, consumed by a dedicated audit sink that writes to both PostgreSQL's audit schema and Loki. The Pessimist argued this introduces a compliance risk: a NATS consumer failure means lost audit records, which is a GDPR violation. The Pessimist proposed services write audit rows to the `audit` schema in the same database transaction as the business operation, with Loki collecting operational logs via stdout.

**Decision**: Services write audit records to the `audit` PostgreSQL schema in the same database transaction as the business operation. Operational logs go to stdout and are collected by Loki via the existing Grafana stack. No NATS dependency for audit.

**Consensus**: The Pessimist's position prevailed due to the GDPR atomicity argument — audit records must be transactionally consistent with the operations they describe.

**Consequences**:
- **Positive**: Audit records are transactionally atomic with business operations; GDPR data export queries run directly against PostgreSQL; no additional async pipeline failure modes for compliance-critical data
- **Negative**: Slight write amplification (audit row in every business transaction); audit queries may need to be optimized with appropriate indexes
- **Caveats**: GDPR data export requests must query the `audit` schema directly. Loki logs are for operational observability only, not for compliance evidence.

---

### [D12] What user interaction pattern should the website use for the self-service quote builder?

**Status**: Accepted

**Task Context**: Task 8 (Website)

**Context**: Both debaters agreed without debate. The quote flow (date selection → equipment browsing → availability checking → contact info → submission) is a natural 4-step funnel. Baymard Institute research cited by the Optimist shows multi-step flows achieve 10–15% higher completion rates for complex forms.

**Decision**: Multi-step wizard with progressive disclosure.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Natural funnel for the domain; higher completion rates for complex forms; shadcn/ui Stepper component supports this natively
- **Negative**: Requires state management across steps
- **Caveats**: None — unanimous agreement

---

### [D13] Should the frontend use shadcn/ui as-is or extend it with custom components?

**Status**: Accepted

**Task Context**: Task 8 (Website)

**Context**: Both debaters agreed. shadcn/ui is designed to be customized (copy-paste architecture). Design tokens exist from the design generation pipeline. A lighting/production company needs strong visual identity.

**Decision**: Extend shadcn/ui with custom design tokens and 3–5 composite components for Sigma-1 branding (EquipmentCard, QuoteLineItem, AvailabilityCalendar, etc.).

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Strong visual identity without forking shadcn primitives; custom composites wrap shadcn components; low-effort, high-impact branding
- **Negative**: Custom components need maintenance alongside shadcn updates
- **Caveats**: None — unanimous agreement

---

### [D14] Which data table component should be used for equipment catalog and finance reporting?

**Status**: Accepted

**Task Context**: Task 8 (Website)

**Context**: Both debaters agreed. TanStack Table v8 is shadcn/ui's own recommended data table approach (their docs use it). It's headless (zero styling opinions) and composes with shadcn's design tokens.

**Decision**: TanStack Table with shadcn/ui styling.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Sorting, filtering, pagination, and column virtualization for 533+ products and finance reports; headless architecture composes with design tokens; not additional complexity — it's the intended architecture
- **Negative**: None identified
- **Caveats**: None — unanimous agreement

---

## 4. Escalated Decisions

No decisions were escalated. All 14 decision points reached resolution during the deliberation session.

---

## 5. Architecture Overview

### Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| AI Agent | OpenClaw with MCP tools | Latest |
| Backend (Catalog, Finance, Vetting) | Rust, Axum | Rust 1.75+, Axum 0.7 |
| Backend (RMS) | Go, gRPC, grpc-gateway | Go 1.22+ |
| Backend (Social Engine) | Node.js, Elysia, Effect | Node.js 20+, Elysia 1.x, Effect 3.x |
| Frontend | Next.js, React, shadcn/ui, TailwindCSS, Effect | Next.js 15, React 19, TailwindCSS 4, Effect 3.x |
| Database | PostgreSQL via CloudNative-PG | PostgreSQL 16 |
| Cache | Valkey (Redis-compatible) | Valkey 7.2 |
| Object Storage | Cloudflare R2 | S3-compatible API |
| CDN / TLS / Ingress | Cloudflare CDN + Cloudflare Tunnel | — |
| Messaging | NATS JetStream (existing in-cluster) | — |
| Observability | Grafana + Loki + Prometheus | Existing OpenClaw stack |
| Signal Integration | Signal-CLI (self-hosted) | — |
| Voice | ElevenLabs | — |
| Telephony | Twilio (SIP/PSTN) | — |
| Payments | Stripe | — |

### Service Architecture

```
                    ┌─────────────┐
                    │  Cloudflare  │
                    │  CDN + TLS   │
                    └──────┬──────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
    ┌───────▼──────┐ ┌────▼─────┐ ┌──────▼──────┐
    │   Website    │ │  Morgan  │ │   Signal    │
    │  (Next.js)   │ │ Web Chat │ │  (Signal-CLI)│
    │ Cloudflare   │ │          │ │             │
    │   Pages      │ └────┬─────┘ └──────┬──────┘
    └───────┬──────┘      │              │
            │         ┌───▼──────────────▼───┐
            │         │   Morgan AI Agent    │
            │         │    (OpenClaw/MCP)     │
            │         └───┬──────────────────┘
            │             │
            │         ┌───▼──────────────────┐
            │         │  MCP Tool-Server     │
            │         │  (stateless proxy)    │
            │         └───┬──┬──┬──┬──┬──────┘
            │             │  │  │  │  │
    ┌───────┘    ┌────────┘  │  │  │  └────────┐
    │            │           │  │  │            │
┌───▼────┐ ┌────▼───┐ ┌─────▼──┐ ┌▼────────┐ ┌▼────────┐
│Catalog │ │  RMS   │ │Finance │ │Vetting  │ │Social   │
│(Rust)  │ │ (Go)   │ │(Rust)  │ │(Rust)   │ │(Node.js)│
│REST/JSON│ │gRPC+gw │ │REST/JSON│ │REST/JSON│ │REST/JSON│
└───┬────┘ └───┬────┘ └───┬────┘ └───┬─────┘ └───┬─────┘
    │          │           │          │            │
    └──────────┴─────┬─────┴──────────┴────────────┘
                     │
        ┌────────────┼────────────┐
        │            │            │
  ┌─────▼────┐ ┌────▼─────┐ ┌───▼──────┐
  │PostgreSQL │ │  Valkey  │ │Cloudflare│
  │(CNPG)    │ │  (Redis) │ │   R2     │
  │schemas:  │ └──────────┘ └──────────┘
  │rms,finance│
  │vetting,  │     ┌──────────┐
  │social,   │     │  NATS    │
  │audit,    │     │JetStream │
  │public    │     └──────────┘
  └──────────┘
```

### Communication Patterns

| Flow Type | Pattern | Example |
|-----------|---------|---------|
| Morgan → Backend (queries) | Synchronous HTTP/gRPC via MCP tool-server | Catalog search, availability check |
| Morgan → Backend (commands) | Synchronous HTTP/gRPC via MCP tool-server | Generate quote, create invoice |
| Cross-service events | Async via NATS JetStream | Opportunity converted → generate invoice; Social post approved → publish |
| Website → Catalog | Synchronous REST/JSON | Equipment browsing, availability |
| Website → Morgan | WebSocket (web chat widget) | Real-time chat |

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Separate PostgreSQL instances per service | Wasteful at Sigma-1 scale; 4–6x memory/backup overhead |
| Cross-schema foreign keys in PostgreSQL | Creates deployment coupling; prevents independent schema migration |
| mTLS for internal service communication | Operationally expensive for single-tenant; Cilium provides sufficient L3/L4 identity |
| NATS for audit logging | GDPR compliance requires transactional atomicity; consumer failure = lost audit records |
| Business logic in the MCP tool-server | Tool-server must remain a thin stateless proxy to avoid becoming a God service |
| NGINX ingress with Let's Encrypt | Cloudflare is already in-cluster; parallel ingress creates two paths to maintain |
| Bitnami Redis Helm chart | Valkey is already deployed and API-compatible; second topology adds zero benefit |
| Monolith for Catalog/Finance/Vetting | Different operational profiles and failure modes; PRD explicitly models separate deployments |
| Trading Desk (Python) | Out of scope for Phase 1 per PRD |
| SMS notifications | Not a goal; Signal/Twilio used instead |
| Multi-region deployment | Single cluster initially per PRD |

---

## 6. Implementation Constraints

### Security Requirements

- **External auth**: JWT-based authentication for all external API access (website, Morgan web chat, public equipment API)
- **Internal auth**: Cilium network policies for service-to-service identity at L3/L4; NATS credential-based auth for async messaging
- **GDPR compliance**: Audit records written in the same PostgreSQL transaction as business operations; data export queries against `audit` schema; customer deletion must cascade through all schemas
- **Security scanning**: Critical/high severity vulnerabilities block merge (enforced by Cipher agent)
- **Secret management**: Automated rotation for all sensitive credentials (PostgreSQL, Redis, API keys); services must tolerate rotated secrets without downtime

### Performance Targets

| Metric | Target |
|--------|--------|
| Morgan simple query response | < 10 seconds |
| Equipment availability check | < 500ms |
| Invoice generation | < 5 seconds |
| Quote-to-invoice workflow (end-to-end) | < 2 minutes |
| Concurrent Signal connections | 500+ |
| Service uptime | 99.9% |
| Test coverage minimum | 80% |
| Lighthouse score (website) | > 90 |

### Operational Requirements

- **NATS JetStream** (where used): Every subject must have explicit retry count, dead-letter queue, and alerting on consumer lag. Consumer failures must surface within 60 seconds via Prometheus/Grafana
- **Signal-CLI state**: PersistentVolumeClaim is mandatory for Signal-CLI's state directory. Pod restarts must NOT trigger re-registration
- **MCP tool-server**: Must degrade gracefully if unavailable (Morgan responds "I can't help right now"); must remain a stateless proxy with zero business logic — enforced in code review
- **Observability**: All services emit Prometheus metrics; structured logs to stdout collected by Loki; Grafana dashboards for all critical flows
- **GitOps**: All deployments via ArgoCD with automatic rollbacks on failure

### Service Dependencies and Integration Points

| Service | External Dependencies |
|---------|-----------------------|
| Morgan | Signal-CLI, ElevenLabs, Twilio, all backend APIs via MCP tool-server |
| Equipment Catalog | PostgreSQL, Valkey, Cloudflare R2 |
| RMS | PostgreSQL, Valkey, Google Calendar API |
| Finance | PostgreSQL, Valkey,

