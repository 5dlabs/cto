# Enhanced PRD

## 1. Original Requirements

> # Project: Sigma-1 — Unified AI Business Platform
>
> - **Website:** https://sigma-1.com
> - **Existing Platform:** https://deployiq.maximinimal.ca
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

The initial task decomposition identified **10 tasks** spanning the full Sigma-1 platform:

| Task ID | Title | Agent | Stack | Priority | Dependencies |
|---------|-------|-------|-------|----------|--------------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | High | — |
| 2 | Implement Equipment Catalog Service | Rex | Rust 1.75+/Axum 0.7 | High | 1 |
| 3 | Develop Rental Management System | Grizz | Go 1.22+/gRPC | High | 1 |
| 4 | Implement Finance Service | Rex | Rust 1.75+/Axum 0.7 | High | 1 |
| 5 | Build Customer Vetting Service | Rex | Rust 1.75+/Axum 0.7 | High | 1 |
| 6 | Develop Social Media Engine | Nova | Node.js 20+/Elysia 1.x + Effect | Medium | 1 |
| 7 | Implement Morgan AI Agent | Angie | OpenClaw/MCP | High | 2, 3, 4, 5, 6 |
| 8 | Develop Web Frontend | Blaze | React 19/Next.js 15 + Effect | High | 2, 7 |
| 9 | Develop Mobile App | Tap | Expo (React Native) | Medium | 2, 7 |
| 10 | Production Hardening & Security | Bolt | Kubernetes/Helm | High | 2–9 |

### Key Services and Components

- **Infrastructure layer**: PostgreSQL 16 (CloudNative-PG), Valkey 7.2, Cloudflare R2, Signal-CLI, Cloudflare Tunnel, Grafana/Loki/Prometheus observability stack
- **Backend services** (4 languages): Equipment Catalog (Rust/Axum), RMS (Go/gRPC), Finance (Rust/Axum), Customer Vetting (Rust/Axum), Social Media Engine (Node.js/Elysia+Effect)
- **AI orchestrator**: Morgan (OpenClaw with 10+ MCP tools)
- **Frontends**: Next.js 15 website (Cloudflare Pages), Expo mobile app
- **QA pipeline**: 6 automated agents (Stitch, Cleo, Tess, Cipher, Atlas, Bolt)

### Agent Assignments

- **Rex** — 3 Rust/Axum services (Catalog, Finance, Vetting); shared Cargo workspace
- **Grizz** — Go/gRPC RMS service
- **Nova** — Node.js Social Media Engine
- **Angie** — Morgan AI agent configuration
- **Blaze** — Next.js website
- **Tap** — Expo mobile app
- **Bolt** — Infrastructure provisioning and production hardening

### Cross-Cutting Concerns Identified

- 13 decision points were raised across tasks covering platform choices, architecture patterns, API design, data modeling, security, service topology, UX behavior, design systems, and GDPR compliance
- All backend services share a single PostgreSQL cluster and Valkey instance
- All services reference a shared `sigma1-infra-endpoints` ConfigMap
- GDPR compliance spans all data-holding services
- Authentication/authorization is required across all services and frontends

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine for caching, rate limiting, and session storage?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4 (Infrastructure, Catalog, RMS, Finance)

**Context**: The cluster already has a Valkey operator (`redis.redis.opstreelabs.in`) deployed. Both debaters agreed that Valkey 7.2 is wire-compatible with Redis 6/7 and that all client libraries (redis-rs, go-redis, ioredis) work without modification. The workloads — rate limiting, sessions, currency rate caching — are vanilla key-value patterns.

**Decision**: Use the existing Valkey operator, deployed as `sigma1-valkey` with `valkey/valkey:7.2-alpine`.

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: Zero additional infrastructure cost; single caching topology to operate; all standard Redis client libraries work unchanged.
- **Negative**: None identified — both parties agreed without reservation.

---

### [D2] Which object storage provider for media assets?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 6 (Infrastructure, Catalog, Social Engine)

**Context**: The PRD targets Cloudflare Pages for the website and Cloudflare Tunnel for ingress. Cloudflare R2 provides zero-egress-fee reads and S3-compatible API. Both debaters agreed this was the obvious choice for a media-heavy platform (533+ product images, event photos).

**Decision**: Use Cloudflare R2 with S3-compatible API.

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: Zero egress costs; native CDN integration with existing Cloudflare infrastructure; S3-compatible API provides migration path if ever needed.
- **Negative**: None identified.

---

### [D3] PostgreSQL strategy: single cluster with schemas, or separate instances?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4, 5, 6 (all data-holding services)

**Context**: The PRD explicitly defines a single `sigma1` database with schemas (rms, crm, finance, audit, public). Both debaters agreed on single cluster with domain schemas. The Pessimist raised a critical operational concern: the PRD specifies `instances: 1`, which means a single PostgreSQL pod backs six services — no failover capability. The Pessimist also flagged connection exhaustion risk from six polyglot services each maintaining connection pools.

**Decision**: Single CloudNative-PG cluster with per-service schemas (rms, finance, catalog, vetting, social) and schema-level GRANT isolation. **Minimum `instances: 2`** for HA failover. Add PgBouncer sidecar for connection pooling.

**Consensus**: 2/2 (100%) on single cluster with schemas; Pessimist's HA amendment (`instances: 2`, PgBouncer) accepted as essential for 99.9% uptime requirement.

**Consequences**:
- **Positive**: Single backup/migration story; cross-service reporting possible; CloudNative-PG handles failover automatically with ≥2 instances.
- **Negative**: Shared blast radius (mitigated by HA). Connection pooling adds one more component.
- **Caveat (Pessimist)**: Cross-schema foreign key cascades for GDPR deletion require explicit design in Task 10 — "GDPR deletion is not just a WHERE clause + CASCADE."

---

### [D6] Row-level tenant isolation or separate databases per tenant?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4, 5, 6 (all data-holding services)

**Context**: This is a single-company platform (Sigma-1/Perception Events). Both debaters agreed separate databases per tenant is over-engineering. The Pessimist flagged that cross-schema FK cascades for GDPR deletion must be designed deletion-safe from day one.

**Decision**: Single database, domain schemas, `org_id`/`tenant_id` columns for row-level filtering.

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: No operational complexity of multiple databases; simple migration management.
- **Negative**: Must design cross-schema FK cascades for GDPR deletion during schema design, not as an afterthought.
- **Caveat (Pessimist)**: Task 10 (Production Hardening) must explicitly address cross-schema deletion patterns.

---

### [D8] Signal-CLI self-hosted or third-party SaaS?

**Status**: Accepted

**Task Context**: Tasks 1, 7 (Infrastructure, Morgan AI Agent)

**Context**: No credible Signal messaging SaaS exists due to Signal's protocol restrictions. Signal-CLI is the only viable path and is already referenced in the PRD.

**Decision**: Signal-CLI as a sidecar to the Morgan pod — self-hosted, open-source.

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: Only viable option; sidecar deployment minimizes network hops to Morgan.
- **Negative**: Signal-CLI is a Java process with known memory leaks under sustained load; Signal's anti-automation stance means accounts can be banned.
- **Caveat (Pessimist)**: This is a runtime risk requiring operational mitigation — memory monitoring, process restart policies, and account rotation strategy. Not an architectural choice but an operational concern for Task 7.

---

### [D9] Separate Finance and Catalog services or merge?

**Status**: Accepted

**Task Context**: Tasks 2, 4 (Equipment Catalog, Finance)

**Context**: Both debaters agreed these are distinct domains with different access patterns (Catalog: public-facing, read-heavy, cacheable; Finance: admin-only, write-heavy, Stripe-integrated) and different scaling profiles.

**Decision**: Separate microservices. Use a shared Rust Cargo workspace for code reuse without service coupling.

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: Independent scaling and deployment cycles; clear domain boundaries; Cargo workspace gives code sharing.
- **Negative**: One additional Deployment manifest to maintain.

---

### [D10] What interaction pattern for Morgan web chat?

**Status**: Accepted

**Task Context**: Tasks 8, 9 (Web Frontend, Mobile App)

**Context**: Industry standard pattern (Intercom, Drift, Crisp) for chat-driven conversion. Morgan's 80%+ autonomous handling target requires high discoverability.

**Decision**: Persistent bottom-right widget that expands to near-full-screen on engagement; dedicated screen on mobile.

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: Maximizes chat discoverability without interrupting browsing; proven conversion pattern.
- **Negative**: None identified.

---

### [D11] shadcn/ui, Radix UI, or custom component library?

**Status**: Accepted

**Task Context**: Tasks 8, 9 (Web Frontend, Mobile App)

**Context**: shadcn/ui is built on Radix UI primitives — it's not an either/or. Custom component library delays Task 8 with no brand justification.

**Decision**: shadcn/ui + TailwindCSS 4 as specified in the PRD.

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: Radix accessibility guarantees + Tailwind styling velocity; copy-paste components with full customization.
- **Negative**: None identified.

---

### [D13] URI-based, header-based, or no API versioning?

**Status**: Accepted

**Task Context**: All service tasks (2, 3, 4, 5, 6, 7, 8, 9)

**Context**: The PRD already uses `/api/v1/` across every endpoint definition. URI versioning is visible in access logs, CDN cache keys, and monitoring dashboards.

**Decision**: URI-based versioning (`/api/v1/...`).

**Consensus**: 2/2 (100%) — Unanimous agreement.

**Consequences**:
- **Positive**: Already specified in PRD; explicit, debuggable, cacheable; no custom middleware required.
- **Negative**: None identified.

---

## 4. Escalated Decisions

### [D4] Synchronous-only service communication or hybrid sync + async via NATS? — ESCALATED

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 6, 7 (all backend services and Morgan)

**Options**:
- **A (Optimist)**: Hybrid — gRPC for synchronous request/response paths, NATS (already in-cluster at `openclaw-nats.openclaw.svc.cluster.local`) for event-driven async workflows (quote→invoice, vetting pipelines, social approval).
- **B (Pessimist)**: Synchronous gRPC for all Morgan-orchestrated service calls; NATS only for social media publish pipeline (Task 6) and optional audit event streaming.

**Optimist argued**: NATS is already deployed with zero operational cost to adopt. PRD data flows (DF-1, DF-2) involve multi-step pipelines that are natural event chains. NATS adds resilience (retry on consumer failure) and decouples services for independent deployment.

**Pessimist argued**: Morgan already IS the orchestrator via MCP tools — each tool call is synchronous. Adding NATS between every service means building, testing, and debugging pub/sub contracts in four languages (Rust, Go, Node.js, OpenClaw) for workflows that are inherently request/response. The 10-second response constraint requires synchronous calls. NATS for six services across a polyglot stack is not "zero cost" — it's six new failure surfaces to debug. Who debugs a lost NATS message at 2am?

**Recommendation**: The Pessimist's position is more defensible for Phase 1. Morgan's MCP tool architecture already provides orchestration — adding NATS for Morgan-initiated flows creates redundant coordination. **Adopt Option B**: synchronous gRPC/REST for all Morgan-orchestrated paths, NATS only for the genuinely async social media publish pipeline (Task 6) and optionally for audit event streaming. Revisit broader NATS adoption in Phase 2 when operational maturity with the polyglot stack is established.

---

### [D5] What API paradigm for internal and external service communication? — ESCALATED

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 6, 7, 8, 9 (all services and frontends)

**Options**:
- **A (Optimist)**: gRPC (tonic for Rust, native for Go) for all internal service-to-service communication; REST via grpc-gateway/tonic-web for public APIs. Protobuf as single source of truth. No GraphQL.
- **B (Pessimist)**: gRPC for Go RMS only (Task 3, as PRD specifies); REST/Axum for Rust services (Tasks 2, 4, 5); REST for Node.js Social Engine (Task 6). OpenAPI specs generated from code, not protobuf.

**Optimist argued**: PRD already specifies gRPC for RMS. Extending to Rust via `tonic` gives strong typing, codegen, and streaming. Single protobuf definition generates both gRPC stubs and OpenAPI specs — single source of truth. No GraphQL needed since frontend data needs are well-defined CRUD.

**Pessimist argued**: The PRD specifies gRPC only for the Go RMS. Rust services are specified as Axum REST. Adding tonic to three Rust services means protobuf codegen pipelines, schema synchronization across Go and Rust, and a cited 4x latency penalty for unoptimized Rust protobuf parsing. Morgan's MCP tools call HTTP endpoints — gRPC adds a translation layer for no consumer benefit. Keep Go gRPC (designed for it), keep everything else REST.

**Recommendation**: The Pessimist's position better respects the PRD's explicit technology assignments and reduces cross-stack complexity. **Adopt Option B**: gRPC + grpc-gateway for the Go RMS (Task 3); REST/Axum for all Rust services (Tasks 2, 4, 5); REST for the Node.js Social Engine (Task 6). Each service generates OpenAPI specs from its own code. Morgan's MCP tools consume REST endpoints. This eliminates the protobuf synchronization burden across four language ecosystems and avoids the Rust protobuf overhead issue cited in the research memo. Both parties agreed: No GraphQL.

---

### [D7] JWT-based auth, session-based, or OAuth2/OIDC? — ESCALATED

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 6, 7, 8, 9, 10 (all services and frontends)

**Options**:
- **A (Optimist)**: JWT with RBAC — stateless, service-mesh friendly, short-lived tokens (15min), refresh tokens stored in Valkey. Lightweight auth service issues tokens.
- **B (Pessimist)**: JWT for web/mobile sessions (issued by API gateway), pre-shared API keys (Kubernetes Secrets) for service-to-service auth in Phase 1. Defer dedicated auth service to Phase 2.

**Optimist argued**: Multiple polyglot services (Rust, Go, Node.js) need to validate tokens independently. JWT avoids per-request Redis lookups. Stateless validation on hot paths. Refresh tokens in Valkey provide revocation.

**Pessimist argued**: One human user (Mike), one AI agent (Morgan), and service-to-service calls. Building an auth service for this is over-engineering. API keys in Kubernetes secrets for inter-service auth, JWT for external sessions. Phase 2 adds the auth service when multi-user is needed.

**Recommendation**: Both agree on JWT mechanism for external sessions and RBAC. The disagreement is scope. **Adopt a pragmatic hybrid**: JWT with RBAC for web/mobile sessions (issued by the API gateway or a thin token endpoint, not a full auth service). Pre-shared API keys (K8s Secrets) for service-to-service auth in Phase 1. This gives stateless validation for external clients and simple, auditable inter-service auth without building a new service. Plan the full auth service for Phase 2 multi-user expansion.

---

### [D12] Distributed per-service GDPR implementation or centralized orchestrator? — ESCALATED

**Status**: Pending human decision

**Task Context**: Tasks 2, 3, 4, 5, 6, 7, 10 (all data-holding services and production hardening)

**Options**:
- **A (Optimist)**: Centralized GDPR orchestrator that fans out to per-service deletion/export endpoints with unified audit log. Potentially a Morgan AI skill.
- **B (Pessimist)**: Centralized deterministic GDPR orchestrator — a simple Rust CLI or Kubernetes Job (NOT a Morgan AI skill) that calls per-service deletion endpoints with structured audit logging.

**Optimist argued**: GDPR requires proving complete deletion across all services. Per-service implementation without a coordinator creates audit gaps. A thin orchestrator calls each service's `DELETE /gdpr/customer/:id` endpoint and logs confirmation.

**Pessimist argued**: Agrees a centralized orchestrator is correct, but GDPR Article 17 has a 30-day legal deadline. AI agent orchestration introduces non-determinism. A deterministic script with structured logging provides auditable, repeatable compliance. Morgan can trigger it, but execution must be deterministic.

**Recommendation**: Both agree on centralized orchestration — the disagreement is whether it should be an AI skill or a deterministic process. **Adopt Option B**: Build a deterministic Rust CLI or Kubernetes CronJob/Job that calls each service's GDPR deletion endpoint, collects confirmations, and writes a structured audit log. Morgan can trigger it via MCP tool, but the actual fan-out and execution is deterministic code. Legal compliance must never depend on AI inference.

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Database | PostgreSQL (CloudNative-PG) | 16 | Single cluster, `instances: ≥2`, PgBouncer sidecar |
| Cache | Valkey | 7.2-alpine | Existing operator, wire-compatible with Redis |
| Object Storage | Cloudflare R2 | — | S3-compatible API, zero egress |
| CDN/Ingress | Cloudflare (Tunnel + Pages) | — | TLS, CDN, SSR hosting |
| Observability | Grafana + Loki + Prometheus | — | Existing OpenClaw stack |
| Messaging | NATS | — | Existing in-cluster; scoped to social publish pipeline only (pending D4 resolution) |

### Service Architecture

| Service | Language/Framework | API Paradigm | Agent |
|---------|-------------------|-------------|-------|
| Equipment Catalog | Rust 1.75+ / Axum 0.7 | REST (OpenAPI) | Rex |
| RMS | Go 1.22+ / gRPC + grpc-gateway | gRPC internal, REST external | Grizz |
| Finance | Rust 1.75+ / Axum 0.7 | REST (OpenAPI) | Rex |
| Customer Vetting | Rust 1.75+ / Axum 0.7 | REST (OpenAPI) | Rex |
| Social Media Engine | Node.js 20+ / Elysia 1.x + Effect 3.x | REST (Effect Schema) | Nova |
| Morgan AI Agent | OpenClaw / MCP | MCP tools → REST/gRPC | Angie |
| Website | Next.js 15 / React 19 / shadcn/ui / TailwindCSS 4 | — | Blaze |
| Mobile App | Expo (React Native) | — | Tap |

### Communication Patterns (pending D4/D5 resolution)

- **Morgan → Backend services**: Synchronous MCP tool calls (REST/gRPC)
- **Go RMS**: gRPC for internal, REST via grpc-gateway for public consumers
- **Rust services**: REST/Axum with OpenAPI specs generated from code
- **Node.js Social Engine**: REST with Effect Schema validation
- **Async**: NATS used only for social media publish pipeline (approve → publish) and optional audit event streaming
- **Frontend → Backend**: REST via URI-versioned endpoints (`/api/v1/...`)

### Key Patterns

- **Cargo workspace**: Rex's three Rust services (Catalog, Finance, Vetting) share code via Cargo workspace without runtime coupling
- **Schema isolation**: Per-domain PostgreSQL schemas (rms, finance, catalog, vetting, social, audit) with schema-level GRANTs
- **Row-level filtering**: `org_id`/`tenant_id` columns on all tenant-scoped tables
- **Connection pooling**: PgBouncer sidecar on the CloudNative-PG cluster to prevent connection exhaustion from six polyglot services
- **Health/Metrics**: Every service exposes `/health/live`, `/health/ready`, and `/metrics` (Prometheus)
- **Config injection**: All services read from `sigma1-infra-endpoints` ConfigMap via `envFrom`

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| GraphQL | Frontend data needs are well-defined CRUD; both debaters agreed it adds latency and complexity without justification |
| Separate Redis instance | Valkey is wire-compatible; second cache topology adds operational surface for zero gain |
| AWS S3 | Cloudflare R2 aligns with existing Cloudflare stack; zero egress saves costs for media-heavy workload |
| Separate databases per service | Single-tenant platform; operational complexity of multiple databases provides no isolation benefit |
| Custom component library | No brand requirement justifies the build cost; shadcn/ui already provides accessible, customizable components |
| Trading Desk (Phase 1) | Python not in core stack; explicitly deferred to Phase 2 |
| Multi-region deployment | Out of scope per PRD |
| OAuth2/OIDC | No third-party login requirement in PRD |

---

## 6. Implementation Constraints

### Security Requirements

- **Authentication**: JWT with RBAC for external sessions; pre-shared API keys (K8s Secrets) for inter-service auth in Phase 1 (pending D7 resolution)
- **GDPR compliance**: Every data-holding service must expose a `DELETE /gdpr/customer/:id` endpoint. A centralized deterministic orchestrator (Rust CLI / K8s Job) coordinates deletion across services with structured audit logging (pending D12 resolution)
- **Secret management**: All sensitive credentials stored in Kubernetes Secrets; automated rotation in Task 10
- **Network policies**: Restrict inter-service communication to required paths only
- **Security scanning**: Critical/high severity vulnerabilities block merge (Cipher agent)

### Performance Targets

| Metric | Target | Service |
|--------|--------|---------|
| Morgan simple query response | < 10 seconds | Morgan (Task 7) |
| Equipment availability check | < 500ms | Equipment Catalog (Task 2) |
| Invoice generation | < 5 seconds | Finance (Task 4) |
| Concurrent Signal connections | 500+ | Morgan (Task 7) |

### Operational Requirements

- **Uptime**: 99.9% for all production services
- **HA**: All critical services run multi-replica with anti-affinity; PostgreSQL `instances: ≥2`
- **Connection pooling**: PgBouncer sidecar required for shared PostgreSQL cluster
- **Observability**: Grafana + Loki + Prometheus (existing stack); all services emit Prometheus metrics
- **GitOps**: ArgoCD with automatic rollbacks on failure
- **Code coverage**: Minimum 80% on all services (enforced by Tess agent)

### Service Dependencies and Integration Points

| Service | External APIs | Infrastructure |
|---------|--------------|----------------|
| Morgan | Signal-CLI (sidecar), ElevenLabs, Twilio | All backend services |
| Equipment Catalog | — | PostgreSQL, Valkey, R2 |
| RMS | Google Calendar API | PostgreSQL, Valkey |
| Finance | Stripe API | PostgreSQL, Valkey |
| Customer Vetting | OpenCorporates, LinkedIn, Google Reviews, credit APIs | PostgreSQL |
| Social Engine | Instagram Graph API, LinkedIn API, Facebook Graph API, OpenAI/Claude | PostgreSQL, R2, NATS |
| Website | Equipment Catalog API, Morgan (web chat) | Cloudflare Pages |
| Mobile App | Equipment Catalog API, Morgan (chat) | — |

### Organizational Preferences

- **Prefer existing in-cluster operators** when available (Valkey, CloudNative-PG, NATS, Grafana stack)
- **API versioning**: URI-based (`/api/v1/...`) across all services — no exceptions
- **Single source of configuration**: `sigma1-infra-endpoints` ConfigMap consumed by all services via `envFrom`
- **Shared Cargo workspace** for Rex's Rust services; no service-coupling at runtime

---

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`**: `true`
- **`frontendTargets`**: `web` | `mobile`
- **Provider mode**: `stitch` (design generation attempted)

### Design Artifact Status

| Provider | Status | Reason |
|----------|--------|--------|
| Stitch | **Failed** | Generation did not complete successfully |
| Framer | Skipped | Not requested |

### Supplied Design Artifacts

No design artifacts (mockups, Figma files, brand guidelines) were supplied with the project intake.

### Reference URLs

- **Production website**: https://sigma-1.com
- **Existing platform**: https://deployiq.maximinimal.ca

### Implications for Frontend Tasks

1. **Web (Task 8)**: Next.js 15 + React 19 + shadcn/ui + TailwindCSS 4 stack is confirmed. Since Stitch design generation failed and no design artifacts were supplied, Blaze (web agent) should:
   - Reference the existing sigma-1.com and deployiq.maximinimal.ca sites for visual direction
   - Use shadcn/ui default design tokens as baseline
   - Implement a clean, modern aesthetic appropriate for a professional lighting/visual production company
   - Prioritize equipment catalog usability and Morgan chat widget prominence
   - Include AI-native features (llms.txt, Schema.org structured data)

2. **Mobile (Task 9)**: Expo (React Native) app should maintain visual consistency with the web frontend. Tap (mobile agent) should:
   - Adapt shadcn/ui patterns to React Native equivalents
   - Morgan chat becomes a dedicated screen/tab (per D10 resolution)
   - Equipment catalog and quote builder are core flows requiring mobile-optimized UX

3. **Design System Gap**: Without supplied design artifacts or successful Stitch generation, implementing agents should establish consistent design tokens (colors, typography, spacing) in Task 8 and propagate to Task 9. The existing sigma-1.com site serves as the primary visual reference.

### 7a. Selected Design Direction

No design selections were provided — `design_selections` was not present in the intake.

### 7b. Design Deliberation Decisions

No design deliberation was conducted — `design_deliberation_result` was not present in the intake.

---

## 8. Open Questions

The following items were not resolved in deliberation and are left to implementing agents' best judgment:

1. **Signal-CLI account rotation strategy** — How to handle Signal's anti-automation account bans. Pessimist flagged this as a runtime risk requiring monitoring and account rotation. Task 7 (Morgan) implementing agent should design a fallback/rotation plan.

2. **Signal-CLI memory management** — Signal-CLI is a Java process with known memory leaks under sustained load. Task 1 (Infrastructure) should set appropriate resource limits and restart policies on the sidecar container.

3. **Currency rate sync frequency** — The PRD mentions a "scheduled job" for currency rate sync in the Finance service. The implementing agent for Task 4 should determine appropriate sync interval (hourly is typical for rental pricing).

4. **Credit data API selection** — The Vetting service (Task 5) references "commercial credit APIs" without specifying which. The implementing agent should evaluate available options (Dun & Bradstreet, Experian Business, etc.) based on Canadian market availability and cost.

5. **Google Reviews integration method** — Scraping vs. official API. Task 5 implementing agent should assess Google Business Profile API availability vs. fallback scraping approach.

6. **

