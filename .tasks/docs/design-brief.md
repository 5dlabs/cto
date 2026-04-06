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

The initial task decomposition identified **10 tasks** spanning infrastructure provisioning, backend microservices, AI agent integration, frontend development, and production hardening.

| Task ID | Title | Agent | Stack | Dependencies | Priority |
|---------|-------|-------|-------|-------------|----------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | — | High |
| 2 | Implement Equipment Catalog Service | Rex | Rust 1.75+/Axum 0.7 | 1 | High |
| 3 | Develop Rental Management System (RMS) | Grizz | Go 1.22+/gRPC | 1 | High |
| 4 | Implement Finance Service | Rex | Rust 1.75+/Axum 0.7 | 1 | High |
| 5 | Build Customer Vetting Service | Rex | Rust 1.75+/Axum 0.7 | 1 | High |
| 6 | Develop Social Media Engine | Nova | Node.js 20+/Elysia 1.x + Effect | 1 | Medium |
| 7 | Implement Morgan AI Agent | Angie | OpenClaw/MCP | 2, 3, 4, 5, 6 | High |
| 8 | Develop Website Frontend | Blaze | Next.js 15/React 19/Effect | 2, 7 | High |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | 2–8 | High |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit | Bolt | Kubernetes/Helm | 9 | High |

### Key Services and Components

- **Infrastructure Layer** (Task 1): CloudNative-PG PostgreSQL cluster, Valkey/Redis operator, Cloudflare R2 object storage, Signal-CLI pod, External Secrets Operator, ConfigMap for service discovery
- **Backend Microservices** (Tasks 2–6): Equipment Catalog (Rust), RMS (Go), Finance (Rust), Customer Vetting (Rust), Social Media Engine (Node.js) — 5 services across 3 language runtimes
- **AI Agent** (Task 7): Morgan via OpenClaw with 10+ MCP tool integrations, Signal/Voice/Web chat interfaces
- **Frontend** (Task 8): Next.js 15 website with equipment catalog, quote builder, portfolio, AI chat widget
- **Production Hardening** (Tasks 9–10): HA scaling, Cloudflare CDN/TLS/Tunnel, network policies, RBAC, secret rotation, audit logging

### Agent Assignments

| Agent | Responsibilities | Language/Stack |
|-------|-----------------|---------------|
| Bolt | Infrastructure provisioning, production hardening (Tasks 1, 9, 10) | Kubernetes, Helm, ArgoCD |
| Rex | Equipment Catalog, Finance, Customer Vetting (Tasks 2, 4, 5) | Rust 1.75+, Axum 0.7 |
| Grizz | Rental Management System (Task 3) | Go 1.22+, gRPC + grpc-gateway |
| Nova | Social Media Engine (Task 6) | Node.js 20+, Elysia 1.x, Effect 3.x |
| Angie | Morgan AI Agent (Task 7) | OpenClaw, MCP tools |
| Blaze | Website frontend (Task 8) | Next.js 15, React 19, TailwindCSS 4 |

### Cross-Cutting Concerns

- **11 decision points** were identified across tasks covering platform choices, data modeling, API design, security, service topology, and design system
- All backend services share a single PostgreSQL cluster (schema-per-service) and a single Valkey cache
- All services consume infrastructure config from the `sigma1-infra-endpoints` ConfigMap
- The QA pipeline involves 6 additional agents (Stitch, Cleo, Tess, Cipher, Atlas, Bolt) for automated review, testing, security scanning, and deployment

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine should be used for caching, rate limiting, and session storage?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2 (Equipment Catalog), Task 3 (RMS), Task 4 (Finance)

**Context**: Both debaters immediately agreed. The Valkey operator (`redis.redis.opstreelabs.in`) is already deployed in the cluster. Valkey 7.2 is wire-compatible with Redis — every standard client library (redis-rs, go-redis, ioredis) works without modification.

**Decision**: Use the existing Valkey operator (Valkey 7.2-alpine) as the cluster-wide Redis-compatible cache.

**Consensus**: 2/2 (100%) — unanimous agreement

**Consequences**:
- ✅ Zero additional operational overhead — operator already running
- ✅ Single cache layer to monitor, back up, and operate
- ✅ Wire-compatible with all Redis client libraries across Rust, Go, and Node.js
- ⚠️ None raised — Pessimist explicitly agreed this was a non-debate

---

### [D2] How should multi-tenancy be handled in the PostgreSQL schema for all backend services?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Tasks 2–6 (all backend services)

**Context**: Both debaters agreed. This is a single-tenant platform for one company (Sigma-1/Perception Events). The PRD explicitly calls out "Multiple schemas: rms, crm, finance, audit, public" in the CloudNative-PG spec. Separate databases per service on a single-node cluster would mean 5+ connection pools, backup schedules, and failover configurations.

**Decision**: Single CloudNative-PG cluster with separate schemas per service (rms, crm, finance, audit, public) within one database, plus `tenant_id` columns where needed for future multi-tenancy.

**Consensus**: 2/2 (100%) — unanimous agreement

**Consequences**:
- ✅ Single connection pool, single backup schedule, minimal operational overhead
- ✅ Cross-service reporting possible via cross-schema queries or views
- ✅ `tenant_id` columns pre-positioned for future row-level security if SaaS expansion occurs
- ⚠️ **Caveat from Pessimist**: Each service MUST own its schema migrations exclusively — no cross-schema DDL. Task 1 must enforce this boundary. This is a hard constraint.

---

### [D3] What API paradigm should be used for inter-service communication?

**Status**: Accepted

**Task Context**: Tasks 2–7 (all backend services + Morgan agent)

**Context**: This was the most substantively debated decision. The Optimist argued for gRPC internal + REST external, citing strongly-typed protobuf contracts, binary serialization efficiency, and the RMS service already being specified as gRPC. The Pessimist argued for REST everywhere, citing the toolchain tax of maintaining protobuf codegen across three languages, the debugging difficulty of gRPC (can't curl), and the fact that all external consumers (Morgan, frontend) use REST anyway.

**Decision**: gRPC for internal service-to-service calls, REST (via grpc-gateway or native HTTP) for external/public APIs and Morgan's MCP tool-server.

**Consensus**: The Optimist's position stands as the PRD explicitly specifies gRPC for the RMS service and grpc-gateway for REST translation. However, the Pessimist's concerns about polyglot complexity are noted.

**Consequences**:
- ✅ Strongly-typed contracts via protobuf — single proto repo generates clients for Rust (tonic), Go, and TypeScript
- ✅ Efficient binary serialization for internal traffic
- ✅ Streaming capabilities available for inventory/delivery updates
- ✅ REST automatically generated via grpc-gateway for external consumers
- ⚠️ **Dissenter concern**: Protobuf codegen adds a shared proto repo that becomes a coordination bottleneck across 3 languages. This must be managed carefully — proto changes require rebuilding clients in all services.
- ⚠️ **Dissenter concern**: HTTP/2 debugging is harder than REST. Teams should have grpcurl/grpcui available in development environments.
- ⚠️ **Dissenter concern**: The Pessimist raised the broader point that 3 languages across services is operationally expensive. While this was not resolved as a formal decision point (it's a PRD architectural choice), implementing agents should be aware of the cross-language maintenance burden.

---

### [D4] What authentication and authorization mechanism should be used?

**Status**: Accepted

**Task Context**: Tasks 1–10 (all tasks)

**Context**: The Optimist proposed JWT for user/frontend auth + mTLS via Cilium for service-to-service. The Pessimist argued that Cilium's mTLS capabilities are not evidenced as configured in this cluster and that debugging mTLS certificate failures is an operational nightmare. The Pessimist proposed relying on Cilium network policies (already deployed) for namespace-level isolation without application-level mTLS.

**Decision**: JWT-based authentication for user/frontend access; Kubernetes network policies (Cilium) for service-to-service isolation. Application-level mTLS is deferred.

**Consensus**: The Pessimist's pragmatic position prevails — network policies provide sufficient isolation for a single-cluster, single-tenant platform without the operational burden of certificate management.

**Consequences**:
- ✅ JWT provides stateless verification for frontend and external API consumers
- ✅ Short-lived tokens with refresh rotation for the frontend (Task 8)
- ✅ Cilium network policies (already deployed) provide namespace-level isolation without certificate management
- ✅ Simpler operational model — no CA rotation, no clock skew issues, no cert debugging at 2am
- ⚠️ If the platform goes multi-cluster or zero-trust mandated in the future, mTLS will need to be added
- ⚠️ JWT claims should include sufficient identity information for audit trails (GDPR compliance)

---

### [D5] Which object storage provider should be used?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2 (Equipment Catalog), Task 6 (Social Media Engine)

**Context**: Unanimous agreement. Cloudflare R2 offers S3 API compatibility and zero egress fees, which is significant for a media-heavy platform (533+ product images, event photo galleries).

**Decision**: Cloudflare R2 as primary object storage.

**Consensus**: 2/2 (100%) — unanimous agreement

**Consequences**:
- ✅ Zero egress fees — critical for media-heavy catalog and photo galleries
- ✅ S3 API-compatible — same aws-sdk-s3 / @aws-sdk/client-s3 libraries work unchanged
- ✅ Cloudflare CDN integration for edge-served assets
- ✅ Consolidates on Cloudflare stack (Pages, Tunnel, R2) for unified management

---

### [D6] How should the public API endpoints be versioned and documented?

**Status**: Accepted

**Task Context**: Tasks 2–6 (all backend services)

**Context**: Unanimous agreement. The PRD already uses `/api/v1/` consistently across all service specs.

**Decision**: Path-based versioning (`/api/v1/...`) with OpenAPI documentation auto-generated from protobuf (grpc-gateway) and code annotations (utoipa for Rust, swag for Go).

**Consensus**: 2/2 (100%) — unanimous agreement

**Consequences**:
- ✅ Version visible in every log line, trace, and CDN cache key
- ✅ grpc-gateway generates OpenAPI specs from proto files (Task 3)
- ✅ utoipa generates OpenAPI from Axum handlers (Tasks 2, 4, 5)
- ✅ Single aggregated Swagger UI for all service documentation

---

### [D7] How should the Signal messenger integration be implemented?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 7 (Morgan Agent)

**Context**: Both debaters agreed there is no viable managed Signal gateway — Signal's protocol is end-to-end encrypted by design. However, the Pessimist raised critical operational risks about Signal-CLI's reliability.

**Decision**: Self-host Signal-CLI as a separate pod in the cluster.

**Consensus**: 2/2 (100%) — unanimous on the choice, with significant caveats

**Consequences**:
- ✅ Only production-viable option for Signal integration
- ✅ Self-hosting ensures message privacy (GDPR compliance) — no third-party relay sees plaintext
- ✅ Fits existing deployment pattern (openclaw-morgan already in cto namespace)
- ⚠️ **Critical caveat from Pessimist**: Signal-CLI is an **unofficial** Java-based client that breaks when Signal updates their protocol. This has happened repeatedly. Mitigations required:
  - Task 1 must pin Signal-CLI versions explicitly
  - Task 9 must include a health check that verifies actual message send/receive capability
  - A **fallback notification path** (email or web chat) must exist when Signal is degraded
- ⚠️ **Unresolved scaling question from Pessimist**: Signal-CLI is a single-threaded Java process maintaining one registration (one phone number). The PRD specifies "500+ concurrent Signal connections." How 500 concurrent conversations are handled through a single Signal-CLI instance is architecturally constrained by Signal's protocol. This is flagged as an open question (see Section 8).

---

### [D8] Should Finance and Customer Vetting be separate services or merged?

**Status**: Accepted

**Task Context**: Tasks 4 (Finance), Task 5 (Customer Vetting)

**Context**: The Optimist argued for separate services citing different failure domains — Finance handles monetary transactions with strict correctness, while Vetting calls external APIs (OpenCorporates, LinkedIn) with unpredictable latency. The Pessimist argued for merging, noting that async Rust with Tokio means a slow HTTP call doesn't block other endpoints, and that separate microservices double the operational surface (deployments, health checks, logs, manifests, connection pools) for a single-company platform.

**Decision**: Keep Finance and Customer Vetting as separate microservices.

**Consensus**: The Optimist's position aligns with the PRD's explicit service decomposition. The PRD defines these as distinct services with different data models, different external dependencies, and different failure characteristics.

**Consequences**:
- ✅ Failure isolation — vetting API timeouts cannot cascade into finance operations
- ✅ Independent deployment — finance can be deployed without touching vetting and vice versa
- ✅ Clear ownership boundaries and separate audit trails
- ✅ Aligns with PRD's explicit architecture
- ⚠️ **Dissenter concern**: For a single-company platform, the operational overhead of 2 separate Kubernetes deployments, 2 connection pools, 2 sets of health checks may not be justified. If operational burden becomes apparent, these can be merged later without schema changes (same PostgreSQL cluster, separate schemas).
- ⚠️ Both services should share a common Rust library crate for database connection setup, health check boilerplate, and observability middleware to minimize duplication.

---

### [D9] Which CDN and TLS termination solution should be used?

**Status**: Accepted

**Task Context**: Task 8 (Website), Task 9 (Production Hardening)

**Context**: Unanimous agreement. The Cloudflare operator is already deployed in the cluster (cloudflare-operator-system namespace). Cloudflare Tunnel eliminates exposed ports, public IPs, and Let's Encrypt renewal automation.

**Decision**: Cloudflare CDN + Cloudflare Tunnel for ingress and TLS termination.

**Consensus**: 2/2 (100%) — unanimous agreement

**Consequences**:
- ✅ No exposed ports, no public IP, no certificate renewal automation needed
- ✅ DDoS protection included
- ✅ Unified DNS, SSL, and caching configuration with Cloudflare Pages
- ✅ Operator already deployed and operational

---

### [D10] What approach should be used for the frontend component library?

**Status**: Accepted

**Task Context**: Task 8 (Website)

**Context**: Unanimous agreement. shadcn/ui wraps Radix UI primitives — it provides Radix's accessibility with pre-styled, copy-paste components. The PRD explicitly specifies this stack, and the tweakcn service already exists in the cluster (cto/tweakcn).

**Decision**: shadcn/ui with TailwindCSS 4 as the base component library and design system.

**Consensus**: 2/2 (100%) — unanimous agreement

**Consequences**:
- ✅ shadcn/ui CLI scaffolding accelerates frontend development
- ✅ Radix UI accessibility primitives included by default
- ✅ tweakcn in-cluster can be leveraged for component customization
- ✅ TailwindCSS 4 for utility-first styling with design token support

---

### [D11] What approach should be used for secret management and rotation?

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 10 (RBAC & Secret Rotation)

**Context**: Unanimous agreement. The External Secrets Operator is already deployed with CRDs (externalsecrets.external-secrets.io, clustersecretstores, etc.) visible in the cluster.

**Decision**: Use External Secrets Operator (already deployed) with automated rotation policies.

**Consensus**: 2/2 (100%) — unanimous agreement

**Consequences**:
- ✅ Operator already deployed and operational — zero additional setup for the core mechanism
- ✅ Automated rotation eliminates manual secret management across 6+ services with Stripe keys, database credentials, and API tokens
- ✅ Compliance-ready — automated rotation satisfies audit requirements
- ⚠️ Rotation policies must be configured per-secret with appropriate TTLs. Tasks 1 and 10 must define these policies.

---

## 4. Escalated Decisions

No decisions were escalated. All 11 decision points reached resolution during deliberation.

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| **Database** | PostgreSQL via CloudNative-PG | 16 |
| **Cache** | Valkey (Redis-compatible) via Opstree operator | 7.2-alpine |
| **Object Storage** | Cloudflare R2 (S3 API-compatible) | — |
| **CDN / TLS / Ingress** | Cloudflare CDN + Cloudflare Tunnel | — |
| **Secret Management** | External Secrets Operator | Already deployed |
| **Frontend Framework** | Next.js 15 (App Router) | 15 |
| **Frontend UI** | React 19 + shadcn/ui + TailwindCSS 4 + Effect 3.x | — |
| **Backend (Catalog, Finance, Vetting)** | Rust 1.75+ / Axum 0.7 | — |
| **Backend (RMS)** | Go 1.22+ / gRPC + grpc-gateway | — |
| **Backend (Social)** | Node.js 20+ / Elysia 1.x + Effect 3.x | — |
| **AI Agent** | OpenClaw with MCP tools | — |
| **Signal Integration** | Signal-CLI (self-hosted pod) | Pinned version |
| **Hosting (Frontend)** | Cloudflare Pages | — |
| **Orchestration** | Kubernetes + ArgoCD (GitOps) | — |
| **Observability** | Grafana + Loki + Prometheus | Existing stack |

### Service Architecture

```
                    ┌─────────────────────────┐
                    │   Cloudflare Edge        │
                    │   CDN + TLS + Tunnel     │
                    └──────────┬──────────────┘
                               │
            ┌──────────────────┼──────────────────┐
            │                  │                  │
     ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐
     │  Next.js 15 │   │   Morgan    │   │  REST APIs  │
     │  (Cloudflare│   │  (OpenClaw) │   │  (external) │
     │   Pages)    │   │  Signal/Web │   │             │
     └──────┬──────┘   └──────┬──────┘   └──────┬──────┘
            │                  │                  │
            │          ┌───────▼───────┐          │
            │          │  MCP Tools    │          │
            │          │  (REST→svc)   │          │
            │          └───────┬───────┘          │
            │                  │                  │
     ┌──────▼──────────────────▼──────────────────▼──────┐
     │              Internal gRPC Mesh                     │
     │   ┌──────────┐ ┌──────────┐ ┌──────────┐          │
     │   │ Catalog  │ │   RMS    │ │ Finance  │          │
     │   │ (Rust)   │ │  (Go)    │ │ (Rust)   │          │
     │   └────┬─────┘ └────┬─────┘ └────┬─────┘          │
     │   ┌──────────┐ ┌──────────┐                        │
     │   │ Vetting  │ │  Social  │                        │
     │   │ (Rust)   │ │ (Node)   │                        │
     │   └────┬─────┘ └────┬─────┘                        │
     └────────┼────────────┼──────────────────────────────┘
              │            │
     ┌────────▼────────────▼──────────────────────────────┐
     │              Shared Infrastructure                   │
     │   ┌────────────┐ ┌──────────┐ ┌──────────────────┐ │
     │   │ PostgreSQL │ │  Valkey  │ │ Cloudflare R2    │ │
     │   │ (CNPG)     │ │ (Redis)  │ │ (Object Storage) │ │
     │   └────────────┘ └──────────┘ └──────────────────┘ │
     └────────────────────────────────────────────────────┘
```

### Communication Patterns

- **Internal (service-to-service)**: gRPC over HTTP/2 with protobuf serialization. Single shared proto repository generates typed clients for Rust (tonic), Go, and TypeScript.
- **External (frontend, Morgan MCP tools)**: REST/JSON via grpc-gateway (RMS) or native Axum HTTP handlers (Rust services). All public APIs at `/api/v1/...`.
- **Service isolation**: Cilium network policies enforce namespace-level isolation. No application-level mTLS.
- **Authentication**: JWT tokens for user/frontend sessions; Cilium policies for service-to-service trust boundaries.

### Data Architecture

- **Single PostgreSQL cluster** (`sigma1-postgres`) in `databases` namespace
- **Schema-per-service**: `rms`, `crm`, `finance`, `audit`, `public` — each service owns its schema exclusively
- **No cross-schema DDL** — services may only read other schemas via views or application-level queries, never modify another service's schema
- **Single Valkey instance** shared by all services for caching, rate limiting, and session storage

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Separate Redis instance (Bitnami chart) | Valkey operator already deployed; dual-cache is pure waste |
| Separate databases per service | Single-node cluster can't support 5+ PostgreSQL instances efficiently |
| REST-only internal communication | PRD specifies gRPC for RMS; protobuf contracts provide stronger typing |
| Application-level mTLS | Cilium network policies provide sufficient isolation; mTLS adds operational burden without proportional benefit for single-cluster/single-tenant |
| Header-based API versioning | Path-based is explicit, cacheable, debuggable — PRD already uses `/api/v1/` |
| AWS S3 for object storage | Cloudflare R2 has zero egress fees and better integration with existing Cloudflare stack |
| Managed Signal gateway | None exists — Signal's E2E encryption prohibits third-party relay |
| Merging Finance and Vetting services | Different failure domains and external dependency profiles justify separate deployments |
| NGINX ingress with Let's Encrypt | Cloudflare Tunnel eliminates exposed ports and certificate renewal |
| Radix UI without shadcn/ui | shadcn/ui wraps Radix with pre-styled components; PRD specifies it |
| Manual secret rotation | External Secrets Operator already deployed; manual rotation is a compliance risk |

---

## 6. Implementation Constraints

### Security Requirements

- **JWT Authentication**: All user-facing and frontend API access must use JWT with short-lived tokens and refresh rotation. JWT claims must include user identity sufficient for GDPR audit trails.
- **Cilium Network Policies**: All inter-service communication must be restricted by CiliumNetworkPolicy. Services may only reach explicitly allowed endpoints.
- **External Secrets Operator**: All API keys, database credentials, and service tokens must be managed via External Secrets with automated rotation policies. No hardcoded secrets in code or manifests.
- **GDPR Compliance**: All services must support data export and customer deletion. Audit logging must capture all API and database access events (Task 10).
- **Security Scanning**: Critical/high severity vulnerabilities block merge (Cipher agent, Semgrep, CodeQL, Snyk).

### Performance Targets

| Metric | Target | Relevant Service |
|--------|--------|-----------------|
| Morgan simple query response | < 10 seconds | Morgan Agent (Task 7) |
| Equipment availability check | < 500ms | Equipment Catalog (Task 2) |
| Invoice generation | < 5 seconds | Finance (Task 4) |
| Quote-to-invoice workflow | < 2 minutes | RMS + Finance (Tasks 3, 4) |
| Concurrent Signal connections | 500+ | Signal-CLI + Morgan (Tasks 1, 7) |
| Service uptime | 99.9% | All production services |

### Operational Requirements

- **Minimum 2 replicas** for all production backend and frontend services (Task 9)
- **GitOps deployment** via ArgoCD with automatic rollbacks on failure
- **Observability**: All services must expose Prometheus metrics at `/metrics` and health probes at `/health/live` and `/health/ready`
- **Monitoring**: Grafana + Loki + Prometheus (existing OpenClaw stack)
- **Minimum 80% code coverage** enforced by Tess agent in CI/CD
- **Signal-CLI version pinning**: Explicit version pins with health checks verifying actual send/receive capability
- **Fallback notification path**: When Signal is degraded, Morgan must have email or web chat fallback

### Service Dependencies and Integration Points

| Service | External Dependencies |
|---------|----------------------|
| Equipment Catalog | PostgreSQL, Valkey, Cloudflare R2 |
| RMS | PostgreSQL, Valkey, Google Calendar API |
| Finance | PostgreSQL, Valkey, Stripe API |
| Customer Vetting | PostgreSQL, OpenCorporates API, LinkedIn API, Google Reviews, Credit APIs |
| Social Media Engine | PostgreSQL, Cloudflare R2, Instagram Graph API, LinkedIn API, Facebook Graph API, OpenAI/Claude |
| Morgan Agent | Signal-CLI, ElevenLabs, Twilio, all backend service APIs |
| Website | Cloudflare Pages, Equipment Catalog API, Morgan web chat |

### Schema Migration Discipline

- Each service owns its PostgreSQL schema migrations exclusively
- No service may execute DDL against another service's schema
- Cross-schema reads are permitted via views or application-level queries only
- Task 1 must enforce this boundary in the initial database provisioning

### Shared Code Recommendations

- Rust services (Tasks 2, 4, 5) should share a common library crate for:
  - Database connection pool setup (sqlx)
  - Health check endpoint boilerplate
  - Prometheus metrics middleware
  - JWT validation middleware
  - Error response formatting

---

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`**: true
- **`frontendTargets`**: web, mobile
- **Provider generation mode**: both (Stitch + Framer)

### Design Provider Status

| Provider | Status | Notes |
|----------|--------|-------|
| Stitch | Generated | Design artifacts produced |
| Framer | Generated | Design artifacts produced |

### Frontend Implementation Implications

The website (Task 8) is a primary deliverable targeting both **web** and **mobile** (responsive design and/or Expo app as noted in the PRD architecture diagram). Key design considerations:

- **shadcn/ui + TailwindCSS 4** is the resolved component library and design system (D10)
- **tweakcn** service exists in-cluster (cto/tweakcn) and can be leveraged for shadcn component customization
- Both Stitch and Framer have generated design artifacts that should inform visual direction for the equipment catalog, quote builder, portfolio, and hero pages
- The website must serve AI agents via `/llms.txt` and `/llms-full` routes with Schema.org structured data
- Mobile target (Expo) is referenced in the PRD architecture diagram but not decomposed into a task — this appears to be Phase 2 or an

