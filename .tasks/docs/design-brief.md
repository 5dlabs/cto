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
> |-------|---------|--------------  |
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

The initial task decomposition identified **10 tasks** spanning infrastructure, backend services, the AI agent, frontend, and production hardening.

| Task ID | Title | Agent | Stack | Priority | Dependencies |
|---------|-------|-------|-------|----------|--------------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | High | — |
| 2 | Implement Equipment Catalog Service API | Rex | Rust 1.75+/Axum 0.7 | High | 1 |
| 3 | Develop RMS Service | Grizz | Go 1.22+/gRPC | High | 1 |
| 4 | Implement Finance Service | Rex | Rust 1.75+/Axum 0.7 | High | 1 |
| 5 | Build Customer Vetting Service | Rex | Rust 1.75+/Axum 0.7 | High | 1 |
| 6 | Develop Social Media Engine | Nova | Node.js 20+/Elysia 1.x + Effect | Medium | 1 |
| 7 | Implement Morgan AI Agent | Angie | OpenClaw/MCP | High | 2, 3, 4, 5, 6 |
| 8 | Develop Sigma-1 Website | Blaze | Next.js 15/React 19/Effect | High | 2, 6, 7 |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | High | 2–8 |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit Logging | Bolt | Kubernetes/Helm | High | 9 |

### Key Services and Components

- **Infrastructure layer**: PostgreSQL 16 (CloudNative-PG), Valkey 7.2 (Redis-compatible), Cloudflare R2, Signal-CLI, Cloudflare Tunnel
- **Backend services** (4): Equipment Catalog (Rust/Axum), RMS (Go/gRPC), Finance (Rust/Axum), Customer Vetting (Rust/Axum)
- **Application services** (2): Social Media Engine (Node.js/Elysia+Effect), Morgan AI Agent (OpenClaw/MCP)
- **Frontend**: Sigma-1 Website (Next.js 15, React 19, shadcn/ui, TailwindCSS 4, Effect 3.x)

### Agent Assignments

- **Bolt** (Infra): Tasks 1, 9, 10
- **Rex** (Rust services): Tasks 2, 4, 5
- **Grizz** (Go service): Task 3
- **Nova** (Node.js): Task 6
- **Angie** (AI Agent): Task 7
- **Blaze** (Frontend): Task 8

### Cross-Cutting Concerns Identified

- 15 decision points raised across tasks spanning platform choices, API design, service topology, data modeling, security, UX, component library, and design system
- 3 hard constraints (Signal-CLI self-hosting, Stripe direct integration, MCP tool-server mediation)
- Multi-language runtime (Rust, Go, Node.js) noted as operational risk by Pessimist
- Schema migration discipline across shared PostgreSQL instance

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine should be used? {#dp-1}

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4 (infrastructure and all cache-dependent services)

**Context**: Both debaters agreed immediately. The Valkey operator (`redis.redis.opstreelabs.in`) is already deployed in-cluster, and Valkey 7.2 is a Linux Foundation-maintained, fully Redis-compatible fork.

**Decision**: Use the existing Valkey 7.2 operator (`valkey/valkey:7.2-alpine`) as the Redis-compatible cache for all Sigma-1 services.

**Consensus**: 2/2 (100%)

**Consequences**:
- (+) Zero additional operator deployment; one cache layer, one set of alerts
- (+) Full Redis API compatibility — all existing SDKs work unchanged
- (−) Valkey is newer than Redis; some edge-case Redis module compatibility may differ (mitigated: no Redis modules are specified)

---

### [D2] Which object storage provider should be used? {#dp-2}

**Status**: Accepted

**Task Context**: Tasks 1, 2, 6, 8 (product images, event photos, portfolio gallery)

**Context**: Both debaters agreed. Cloudflare R2 aligns with the existing Cloudflare footprint (Pages, Tunnel, CDN) and provides zero-egress-cost serving.

**Decision**: Use Cloudflare R2 as the primary S3-compatible object storage for all media assets.

**Consensus**: 2/2 (100%)

**Consequences**:
- (+) Zero egress costs when served via Cloudflare CDN; S3-compatible API
- (+) Eliminates separate CDN configuration for media assets
- (+) Single billing relationship with existing Cloudflare account
- (−) Vendor concentration on Cloudflare (mitigated: S3-compatible API allows migration)

---

### [D3] Which PostgreSQL operator should be used? {#dp-3}

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4, 5, 6 (all services using PostgreSQL)

**Context**: Both debaters agreed. CloudNative-PG is explicitly specified in the PRD's K8s manifests, is a CNCF Sandbox project, and provides unified backup/restore for a single-tenant system.

**Decision**: Use the existing CloudNative-PG operator with a single PostgreSQL 16 cluster and schema-per-service isolation.

**Consensus**: 2/2 (100%)

**Consequences**:
- (+) One WAL stream, one backup policy, unified monitoring
- (+) Schema-per-service gives logical isolation without connection pool multiplication
- (−) Single-cluster failure domain (mitigated: CNPG supports HA replicas, addressed in Task 9)

---

### [D6] How should schema separation be handled in PostgreSQL? {#dp-6}

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4, 5 (all backend services sharing one database)

**Context**: Both debaters agreed on schema-per-service within a single database. The Pessimist added a critical constraint: **no cross-schema JOINs**, arguing that cross-schema queries couple migration paths and defeat the isolation purpose. The Optimist had cited cross-schema joins for reporting as a benefit. The Pessimist's position was stronger: if Finance needs RMS data, it should call the RMS API or read from a dedicated reporting materialized view.

**Decision**: Separate PostgreSQL schemas (`rms`, `finance`, `vetting`, `public`, `social`) within a single database instance. **Strict rule: no cross-schema JOINs in application code.** Cross-domain data access uses either service-to-service API calls or a dedicated `reporting` schema with materialized views owned by a reporting process.

**Consensus**: 2/2 on schema separation; the no-cross-schema-JOIN constraint is adopted from the Pessimist's stronger argument

**Consequences**:
- (+) Clean namespace isolation with independent migration paths per service
- (+) Prevents hidden coupling between service schemas
- (+) Materialized reporting views provide a controlled interface for cross-domain queries
- (−) Slightly more complex reporting setup (materialized views must be maintained)
- (−) Application-level joins may be marginally slower than SQL joins (irrelevant at this scale)

---

### [D11] What is the primary navigation paradigm for the website? {#dp-11}

**Status**: Accepted

**Task Context**: Task 8 (Sigma-1 website)

**Context**: Both debaters agreed immediately. Sigma-1 is a public-facing catalog/marketing site, not a dashboard application. Topbar navigation is the established convention for rental/catalog sites.

**Decision**: Topbar navigation with responsive mobile hamburger menu.

**Consensus**: 2/2 (100%)

**Consequences**:
- (+) Matches user expectations for catalog/marketing sites
- (+) shadcn/ui provides responsive navigation components out of the box
- (−) None noted

---

### [D12] Which data table component for equipment catalog and quote builder? {#dp-12}

**Status**: Accepted

**Task Context**: Task 8 (website — equipment catalog, quote builder)

**Context**: Both debaters agreed. shadcn/ui's own documentation recommends TanStack Table for advanced features. The equipment catalog has 533+ products requiring sorting, filtering, and pagination.

**Decision**: TanStack Table (v8) with shadcn/ui styling wrappers.

**Consensus**: 2/2 (100%)

**Consequences**:
- (+) Battle-tested, headless, framework-agnostic; 25k+ GitHub stars
- (+) Native integration path with shadcn/ui
- (−) None noted

---

### [D13] How should theming and design tokens be managed? {#dp-13}

**Status**: Accepted

**Task Context**: Task 8 (website and web chat widget)

**Context**: Both debaters agreed. shadcn/ui already uses CSS custom properties for theming; TailwindCSS 4 supports CSS variables natively. Adding a custom token layer would be unnecessary abstraction.

**Decision**: TailwindCSS 4 with shadcn/ui CSS variables, extended for Sigma-1 branding. The chat widget inherits tokens via CSS variable injection.

**Consensus**: 2/2 (100%)

**Consequences**:
- (+) Works with shadcn/ui's grain — no fighting the framework
- (+) Chat widget theming via CSS variable injection regardless of mount context
- (−) None noted

---

### [D15] How should API versioning be handled? {#dp-15}

**Status**: Accepted

**Task Context**: Tasks 2, 3, 4, 5, 6, 7, 8 (all services exposing APIs)

**Context**: Both debaters agreed. The PRD already specifies `/api/v1/` prefix across every service endpoint. URI-based versioning is explicit, cache-friendly, and debuggable.

**Decision**: URI-based versioning (`/api/v1/...`) for all public and internal APIs, as specified in the PRD.

**Consensus**: 2/2 (100%)

**Consequences**:
- (+) Consistent with PRD specification; visible in logs and proxies
- (+) Cache-friendly; no proxy/CDN configuration needed for version routing
- (−) URI pollution if many versions accumulate (irrelevant for v1)

---

### [D8] Signal integration: self-hosted Signal-CLI {#dp-8}

**Status**: Accepted (Hard Constraint)

**Task Context**: Tasks 1, 7 (infrastructure provisioning, Morgan AI agent)

**Context**: PRD prescribes Signal-CLI as a sidecar/separate pod. Both debaters acknowledged this as a hard constraint. The Pessimist raised a critical operational concern: Signal-CLI is an unofficial Java client that breaks when Signal updates its protocol.

**Decision**: Self-host Signal-CLI as a sidecar or separate pod, as specified in the PRD.

**Consensus**: Hard constraint — not debatable

**Consequences**:
- (+) No third-party SaaS dependency for core messaging channel
- (−) **Critical risk**: Signal-CLI breaks on Signal protocol updates without backward compatibility. Implementing agents must document a fallback communication path (web chat, voice) and establish monitoring for Signal-CLI health. See Section 8 (Open Questions) for the rollback plan requirement.

---

### [D9] Stripe integration: direct API {#dp-9}

**Status**: Accepted (Hard Constraint)

**Task Context**: Task 4 (Finance Service)

**Context**: PRD specifies direct Stripe API integration. Payment orchestration platforms add abstraction over a single provider — unnecessary at this stage.

**Decision**: Integrate directly with Stripe API for payments and invoicing.

**Consensus**: Hard constraint — not debatable

**Consequences**:
- (+) Simplest path for single-provider payments
- (−) Migration cost if a second payment provider is needed later (acceptable: Phase 2 concern)

---

### [D10] Morgan tool orchestration: MCP tool-server {#dp-10}

**Status**: Accepted (Hard Constraint)

**Task Context**: Task 7 (Morgan AI Agent)

**Context**: MCP tool-server mediation is the OpenClaw architecture pattern and a hard constraint per the PRD.

**Decision**: Morgan accesses all backend services via MCP tool-server abstraction, not direct API calls.

**Consensus**: Hard constraint — not debatable

**Consequences**:
- (+) Clean abstraction between AI agent and backend services; transport-agnostic
- (+) Tool definitions serve as a contract for Morgan's capabilities
- (−) Additional indirection layer (acceptable: negligible latency overhead)

---

## 4. Escalated Decisions

### [D4] What API paradigm for inter-service communication? — ESCALATED {#dp-4}

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 7 (all backend services and Morgan)

**Options**:
- **A (Optimist)**: Hybrid — gRPC for RMS internal calls, REST/JSON for all other services, with Morgan accessing everything via MCP tool-server
- **B (Pessimist)**: REST/JSON for all services, including RMS. Drop gRPC entirely.

**Optimist argued**: The PRD prescribes gRPC for RMS (Task 3) with grpc-gateway for REST exposure. The Rust/Axum services are specified with REST endpoints. Forcing gRPC onto Axum services adds unnecessary complexity. The hybrid approach matches the PRD specification.

**Pessimist argued**: The RMS already needs a REST gateway (grpc-gateway) because every consumer speaks HTTP — Morgan via MCP tool-server, the website, and other services. Maintaining proto definitions, generated Go stubs, *and* a REST translation layer triples the failure surface. gRPC's benefits (binary serialization, streaming, code generation) don't justify the cost at <500 concurrent users. The grpc-gateway is an admission that REST clients exist — so skip the middleman.

**Recommendation**: The Pessimist's argument is operationally stronger. Every consumer of the RMS is an HTTP client. However, the PRD explicitly specifies gRPC for the RMS. **If the PRD specification is authoritative**, go with Option A (hybrid). **If operational simplicity is prioritized**, go with Option B (all REST). This decision also interacts with dp-16 (RMS language choice) — if the RMS stays in Go, gRPC is more idiomatic in Go; if consolidated to Rust, REST/Axum is more natural.

---

### [D5] Rust service topology: microservices vs. modular monolith? — ESCALATED {#dp-5}

**Status**: Pending human decision

**Task Context**: Tasks 2, 4, 5 (Equipment Catalog, Finance, Customer Vetting)

**Options**:
- **A (Optimist)**: Deploy as separate microservices, each with its own Kubernetes Deployment
- **B (Pessimist)**: Deploy as a single Rust/Axum binary with modular internal structure (workspace crates), separate database schemas

**Optimist argued**: These services have fundamentally different operational profiles — Equipment Catalog is read-heavy/public-facing, Finance handles Stripe webhooks requiring strict reliability, Customer Vetting makes external API calls with unpredictable latency. A failure in vetting should never impact catalog availability. Separate deployments give independent scaling, independent rollbacks, and clear ownership boundaries. Shared code goes in a workspace crate.

**Pessimist argued**: None of these services need independent scaling at this volume — 533 products, dozens of invoices/week, single-digit daily vetting runs. A circuit breaker in the vetting module achieves isolation without three separate Deployments, health checks, resource limits, CI pipelines, and container images. This is textbook over-engineering for <500 daily requests across all three domains. Ship a modular monolith, extract later when numbers demand it.

**Recommendation**: Both arguments are valid for different time horizons. The Pessimist is correct about current scale. The Optimist is correct about long-term maintainability. A pragmatic middle ground: **start with a Rust workspace monorepo with separate crates per domain, deployed as a single binary** (Pessimist's preference), but **structure the code to allow extraction to separate binaries with minimal refactoring** (satisfying the Optimist's isolation concerns). The workspace crate boundary is the same either way — only the deployment topology differs.

---

### [D7] Authentication and authorization for internal and external access? — ESCALATED {#dp-7}

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 6, 7, 8, 10 (all services and infrastructure)

**Options**:
- **A (Optimist)**: mTLS for internal service-to-service authentication via Cilium, OAuth2/JWT for external API access
- **B (Pessimist)**: Cilium network policies for service isolation (no mTLS), pre-shared API keys for internal service-to-service calls (rotated via external-secrets operator), JWT for external API access

**Optimist argued**: Cilium provides transparent mTLS for pod-to-pod communication without application-level changes, securing internal calls with zero code overhead. Combined with JWT for external access, this gives defense-in-depth. Shared secrets are a single point of compromise.

**Pessimist argued**: Cilium mTLS requires kube-proxy-replacement mode with the identity-aware policy engine properly configured. If Cilium restarts or loses its identity cache, internal calls fail silently — a debugging nightmare that looks like a network partition. For <10 services in one namespace, Cilium network policies (already available) provide sufficient isolation. Pre-shared API keys (rotated via external-secrets operator) are simple, debuggable, and visible in logs.

**Recommendation**: The Pessimist's approach is lower risk for a small team and small service count. Cilium mTLS is the superior long-term architecture but introduces operational complexity that may not be justified for a single-tenant, <10 service platform. **Recommend Option B (network policies + API keys + JWT) for v1**, with mTLS as a documented upgrade path when the team has operational confidence with Cilium's identity system.

---

### [D14] What is the access control model for admin endpoints? — ESCALATED {#dp-14}

**Status**: Pending human decision

**Task Context**: Tasks 2, 4, 5, 10 (admin endpoints across services and RBAC hardening)

**Options**:
- **A (Optimist)**: RBAC with roles assigned per user (admin, operator, viewer)
- **B (Pessimist)**: Simple admin whitelist for v1, with RBAC as a documented Phase 2 upgrade path

**Optimist argued**: RBAC with 3–4 roles (admin, operator, viewer, agent) covers all needs. ABAC is over-engineered for <10 users, but a simple whitelist doesn't scale if crew members or contractors need limited access. RBAC maps cleanly to JWT claims.

**Pessimist argued**: The PRD names one person (Mike) as the admin. RBAC implies role management UI, role assignment flows, and role-checking middleware across all services — all for a team countable on one hand. A whitelist is one `if` statement. Migrate to RBAC when the team grows beyond 5.

**Recommendation**: The Pessimist's pragmatism is appropriate for launch, but the Optimist is correct that even small teams benefit from role differentiation (Mike vs. Morgan the agent vs. a contractor). **Recommend a lightweight RBAC with 2–3 hardcoded roles** (admin, agent, viewer) stored as JWT claims — no role management UI, no assignment flows. Roles are assigned in configuration/database seed, not through a self-service interface. This is barely more complex than a whitelist but provides the semantic foundation for later expansion.

---

### [D16] Should the RMS be written in Go or consolidated into Rust? — ESCALATED {#dp-16}

**Status**: Pending human decision (raised by Pessimist as cross-cutting concern)

**Task Context**: Tasks 2, 3, 4, 5 (all backend services)

**Options**:
- **A (PRD-specified)**: RMS in Go 1.22+ with gRPC, as specified in the PRD
- **B (Pessimist)**: Consolidate RMS into Rust/Axum, matching Equipment Catalog, Finance, and Customer Vetting

**Optimist argued** (implicitly, by defending PRD specification): The PRD prescribes Go for RMS with gRPC. The hybrid API approach (dp-4) keeps Go where it's specified.

**Pessimist argued**: Three backend languages (Rust, Go, Node.js) for a single-tenant platform means three compiler toolchains, three dependency ecosystems, three debugging skill sets. Go for the RMS is justified only if gRPC or Go-specific concurrency is needed — but if gRPC is dropped (dp-4 Option B), Go's advantage disappears. Four Rust/Axum services share one Dockerfile pattern, one CI template, and one set of observability instrumentation.

**Recommendation**: This decision is tightly coupled to dp-4. **If gRPC is kept** (dp-4 Option A), Go is the natural choice for the RMS and the PRD specification should be followed. **If gRPC is dropped** (dp-4 Option B), the argument for Go weakens significantly and consolidation to Rust/Axum reduces operational surface area. The human should resolve dp-4 and dp-16 together.

---

## 5. Architecture Overview

Based on resolved decisions and the PRD specification, the Sigma-1 platform follows this agreed architecture:

### Technology Stack

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Database | PostgreSQL | 16 | CloudNative-PG operator, single cluster |
| Cache | Valkey | 7.2-alpine | Opstree Redis operator, Redis-compatible |
| Object Storage | Cloudflare R2 | — | S3-compatible API, zero-egress CDN |
| Backend (Rust) | Rust + Axum | 1.75+ / 0.7 | Equipment Catalog, Finance, Customer Vetting |
| Backend (Go) | Go + gRPC* | 1.22+ | RMS (*pending dp-4/dp-16 resolution) |
| Backend (Node.js) | Node.js + Elysia + Effect | 20+ / 1.x / 3.x | Social Media Engine |
| AI Agent | OpenClaw + MCP | — | Morgan agent |
| Frontend | Next.js + React + Effect | 15 / 19 / 3.x | shadcn/ui, TailwindCSS 4 |
| UI Components | shadcn/ui + TanStack Table | v8 | Topbar nav, CSS variable theming |
| Hosting | Cloudflare Pages | — | Website static + SSR |
| Ingress | Cloudflare Tunnel | — | Morgan and web access |
| Payments | Stripe API | — | Direct integration (hard constraint) |
| Messaging | Signal-CLI | — | Self-hosted (hard constraint) |
| Observability | Grafana + Loki + Prometheus | — | Existing OpenClaw stack |

### Service Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Cloudflare Edge                                         │
│  ┌─────────┐  ┌──────────────┐  ┌────────────────┐    │
│  │  R2 CDN │  │ Pages (SSR)  │  │ Tunnel (Ingress)│   │
│  └────┬────┘  └──────┬───────┘  └───────┬────────┘    │
└───────┼──────────────┼──────────────────┼──────────────┘
        │              │                  │
┌───────┼──────────────┼──────────────────┼──────────────┐
│  K8s Cluster (sigma1 namespace)                         │
│       │              │                  │               │
│  ┌────▼────┐    ┌────▼────┐      ┌─────▼─────┐       │
│  │Equipment│    │ Website │      │  Morgan   │       │
│  │Catalog  │    │(Next.js)│      │(OpenClaw) │       │
│  │(Rust)   │    └─────────┘      │ + MCP     │       │
│  └────┬────┘                     └─────┬─────┘       │
│       │                                │              │
│  ┌────▼────┐  ┌─────────┐      ┌─────▼─────┐       │
│  │ Finance │  │   RMS   │      │Signal-CLI │       │
│  │ (Rust)  │  │(Go/Rust*)│     │ (sidecar) │       │
│  └────┬────┘  └────┬────┘      └───────────┘       │
│       │            │                                 │
│  ┌────▼────┐  ┌────▼────┐                           │
│  │Customer │  │ Social  │                           │
│  │Vetting  │  │ Engine  │                           │
│  │ (Rust)  │  │(Node.js)│                           │
│  └────┬────┘  └────┬────┘                           │
│       │            │                                 │
├───────┼────────────┼─────────────────────────────────┤
│  databases namespace                                    │
│  ┌────▼────────────▼────┐  ┌──────────┐              │
│  │  PostgreSQL 16       │  │ Valkey   │              │
│  │  (CloudNative-PG)    │  │ 7.2      │              │
│  │  schemas: rms,       │  └──────────┘              │
│  │  finance, vetting,   │                             │
│  │  public, social,     │                             │
│  │  reporting           │                             │
│  └──────────────────────┘                             │
└─────────────────────────────────────────────────────────┘
```

*\* RMS language pending dp-4/dp-16 resolution*

### Key Patterns and Constraints

1. **Schema isolation**: Each service owns its PostgreSQL schema. **No cross-schema JOINs.** Cross-domain data flows through service APIs or a dedicated `reporting` schema with materialized views.
2. **API versioning**: All endpoints use URI-based versioning (`/api/v1/...`)
3. **Theming**: TailwindCSS 4 + shadcn/ui CSS variables extended for Sigma-1 branding
4. **Navigation**: Topbar with responsive hamburger menu for mobile
5. **Data tables**: TanStack Table v8 with shadcn/ui styling
6. **Agent orchestration**: Morgan uses MCP tool-server exclusively — never direct API calls
7. **Messaging**: Signal-CLI self-hosted as sidecar/pod
8. **Payments**: Direct Stripe API integration

### Explicitly Ruled Out

- **Bitnami Redis / separate Redis instance**: Valkey is already deployed and fully compatible
- **AWS S3 as primary storage**: R2 preferred given existing Cloudflare footprint and zero-egress pricing
- **Separate PostgreSQL operators/instances per service**: Single CNPG cluster with schema isolation is sufficient for single-tenant platform
- **Custom token system for theming**: shadcn/ui CSS variables are the native approach; adding another layer is unnecessary
- **Custom data table implementation**: TanStack Table is recommended by shadcn/ui; building custom is vanity engineering
- **Payment orchestration platforms**: Single-provider (Stripe) doesn't warrant abstraction
- **Header-based API versioning**: URI-based is explicit, cache-friendly, and already specified in PRD
- **Sidebar navigation**: Inappropriate for a catalog/marketing site
- **ABAC for access control**: Over-engineered for <10 users

---

## 6. Implementation Constraints

### Security Requirements

- **GDPR compliance**: Data export and customer deletion capabilities required across all services storing personal data
- **Signal-CLI**: Self-hosted (hard constraint); must not use third-party Signal gateways
- **Stripe**: Direct API integration only (hard constraint); no payment orchestration middleware
- **External secrets**: All third-party API keys (Stripe, LinkedIn, OpenCorporates, ElevenLabs, Twilio, Instagram, Facebook) stored in Kubernetes secrets; rotation via external-secrets operator
- **Security scanning**: Critical/high severity vulnerabilities block merge (Cipher agent)
- **Authentication** (pending dp-7 resolution): At minimum, JWT for external API access; internal auth mechanism TBD

### Performance Targets

| Metric | Target | Service |
|--------|--------|---------|
| Morgan simple query response | < 10 seconds | Morgan (Task 7) |
| Equipment availability check | < 500ms | Equipment Catalog (Task 2) |
| Invoice generation | < 5 seconds | Finance (Task 4) |
| Concurrent Signal connections | 500+ | Morgan (Task 7) |
| Service uptime | 99.9% | All production services |
| Website Lighthouse score | > 90 | Website (Task 8) |
| Code coverage | ≥ 80% | All services |

### Operational Requirements

- **GitOps deployment**: ArgoCD with automatic rollbacks on failure
- **Observability**: Grafana + Loki + Prometheus (existing OpenClaw stack)
- **Health checks**: All services expose `/health/live` (liveness) and `/health/ready` (readiness) Kubernetes probes
- **Metrics**: All services expose Prometheus metrics at `/metrics`
- **HA**: PostgreSQL and Valkey

