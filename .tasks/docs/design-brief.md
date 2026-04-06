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
> |-------|---------|----|
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

The initial task decomposition identified **10 tasks** spanning infrastructure provisioning, 6 backend microservices, 2 frontend clients, and production hardening.

| Task ID | Title | Agent | Stack | Priority | Dependencies |
|---------|-------|-------|-------|----------|-------------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | High | None |
| 2 | Equipment Catalog Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 3 | Rental Management System (RMS) | Grizz | Go 1.22+/gRPC | High | Task 1 |
| 4 | Finance Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 5 | Customer Vetting Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 6 | Social Media Engine | Nova | Node.js 20+/Elysia + Effect | Medium | Task 1 |
| 7 | Morgan AI Agent | Angie | OpenClaw/MCP | High | Tasks 2–6 |
| 8 | Sigma-1 Website | Blaze | Next.js 15/React 19/Effect | High | Tasks 2, 7 |
| 9 | Mobile App | Tap | Expo (React Native) | Medium | Tasks 2, 7 |
| 10 | Production Hardening & Security | Bolt | Kubernetes/Helm | High | Tasks 2–9 |

### Key Services & Components

- **Infrastructure Layer**: PostgreSQL 16 (CloudNative-PG), Valkey 7.2 (Opstree operator), Cloudflare R2, Signal-CLI, Cloudflare Tunnel
- **Backend Services**: Equipment Catalog (Rust), RMS (Go), Finance (Rust), Customer Vetting (Rust), Social Engine (Node.js)
- **AI Agent**: Morgan (OpenClaw with 10+ MCP tools)
- **Frontend Clients**: Next.js 15 website (Cloudflare Pages), Expo mobile app
- **QA Pipeline**: Stitch (review), Cleo (quality), Tess (testing), Cipher (security), Atlas (merge), Bolt (deploy)

### Agent Assignments & Technology Stacks

- **Bolt** — Kubernetes/Helm infrastructure provisioning and production hardening
- **Rex** — Three Rust/Axum 0.7 services (Equipment Catalog, Finance, Customer Vetting)
- **Grizz** — Go 1.22+ gRPC service (RMS)
- **Nova** — Node.js/Elysia + Effect service (Social Engine)
- **Angie** — OpenClaw/MCP agent configuration (Morgan)
- **Blaze** — Next.js 15/React 19 website
- **Tap** — Expo/React Native mobile app

### Cross-Cutting Concerns

- **12 decision points** identified across tasks covering platform choices, API design, data modeling, security, service topology, UX behavior, design systems, and GDPR compliance
- All services share the same PostgreSQL cluster (separate schemas), Valkey instance, and Cloudflare R2 storage
- All services expose `/metrics` (Prometheus) and `/health/live` + `/health/ready` endpoints
- GDPR compliance (data export and deletion) affects every service
- API versioning convention (`/api/v1/`) is universal

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine should be used for caching, rate limiting, and session storage?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4 (infrastructure provisioning and all backend services needing cache)

**Context**: Both debaters immediately agreed. The Valkey operator (`redis.redis.opstreelabs.in`) is already deployed in-cluster, and Valkey 7.2 is a fully Redis-compatible Linux Foundation fork that passes the complete Redis test suite.

**Decision**: Use the existing Valkey operator with `valkey/valkey:7.2-alpine` image. Do not deploy a separate Redis instance.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Zero new operational surface; one cache operator pattern for all services; full Redis API compatibility confirmed
- **Negative**: None raised
- **Caveats**: None — this was unanimous

---

### [D2] Which object storage provider should be used for product images, event photos, and static assets?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 6, 8 (infrastructure, catalog images, social photos, website assets)

**Context**: Both debaters agreed. The PRD already specifies Cloudflare infrastructure (Pages, Tunnel), making R2 the natural fit. Zero egress fees vs. S3's $0.09/GB, S3-API-compatible, and automatic Cloudflare CDN integration.

**Decision**: Use Cloudflare R2 as the primary S3-compatible object storage. All S3 SDKs (aws-sdk-s3 in Rust, aws-sdk-go in Go, @aws-sdk/client-s3 in Node) work unchanged.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Zero egress costs; native CDN integration; consistent Cloudflare ecosystem
- **Negative**: None raised
- **Caveats**: None — this was unanimous

---

### [D3] Which PostgreSQL operator should be used for managing the main database cluster?

**Status**: Accepted

**Task Context**: Task 1 (infrastructure provisioning)

**Context**: Both debaters agreed. The PRD explicitly specifies CloudNative-PG, the CRD is already registered in-cluster, and CNPG is CNCF-adopted with 3.8k GitHub stars.

**Decision**: Use the existing CloudNative-PG operator (`postgresql.cnpg.io/v1`). Deploy a single PostgreSQL 16 cluster named `sigma1-postgres` in the `databases` namespace.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Mature operator; automated failover; continuous WAL archival; native Prometheus metrics
- **Negative**: None raised
- **Caveats**: None — this was unanimous

---

### [D4] What API paradigm should be used for inter-service communication?

**Status**: Accepted

**Task Context**: Tasks 2, 3, 4, 5, 6, 7, 8, 9 (all backend services, Morgan agent, and frontend clients)

**Context**: The Optimist proposed gRPC for all internal service-to-service calls with REST gateways for external clients, arguing protobuf definitions would be the single source of truth across Rust, Go, and TypeScript. The Pessimist strongly countered that polyglot gRPC across three runtimes (tonic for Rust, grpc-go for Go, nice-grpc for Node.js) creates unnecessary complexity: protobuf codegen pipelines for three languages, debugging difficulty (`curl` doesn't speak gRPC), and a shared proto repo that becomes a contention point. The Pessimist noted the PRD already specifies REST endpoints for Equipment Catalog, Finance, Vetting, and Social Engine, and that Morgan's MCP tools are HTTP-native. The RMS (Go) is the only service explicitly designed as gRPC. At <100 RPS for a single-company platform, the performance argument is irrelevant.

**Decision**: REST (HTTP/JSON) for all services **except** RMS, which exposes gRPC + grpc-gateway as the PRD specifies. Other services consume RMS via its REST gateway. No polyglot gRPC — services communicate via REST.

**Consensus**: 2/2 (100% — the Pessimist's position won; the Optimist's gRPC-everywhere proposal was effectively withdrawn given the strength of the debuggability and operational simplicity arguments)

**Consequences**:
- **Positive**: Debuggable with `curl`; observable in access logs; no protobuf codegen pipeline across three languages; MCP tools use native HTTP; matches PRD endpoint specifications
- **Negative**: Slightly less type-safe for internal calls than gRPC; slightly higher serialization overhead (irrelevant at this scale)
- **Caveats**: RMS retains its gRPC interface — any service calling RMS directly should use its REST gateway endpoints, not attempt gRPC interop. If internal traffic ever reaches scale where gRPC matters, it can be adopted service-by-service

---

### [D5] How should multi-tenancy and schema separation be handled?

**Status**: Accepted

**Task Context**: Tasks 1, 2, 3, 4, 5, 6 (all services sharing PostgreSQL)

**Context**: Both debaters agreed. For a single-tenant platform (one company — Sigma-1/Perception Events), separate databases per service adds operational overhead without benefit. The PRD explicitly states this approach.

**Decision**: Use separate schemas within a single PostgreSQL database (`sigma1`). Schemas: `rms`, `finance`, `catalog`, `vetting`, `social`, `audit`. Each service gets a dedicated database role with `USAGE` only on its own schema.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Logical isolation with independent migration tracks; cross-schema references possible when needed; single CNPG cluster to manage; per-service role-based access provides sufficient isolation
- **Negative**: None raised
- **Caveats**: If the platform ever becomes multi-tenant, this decision should be revisited

---

### [D6] What authentication mechanism should be used for backend service APIs?

**Status**: Accepted

**Task Context**: Tasks 1–10 (all services and production hardening)

**Context**: The Optimist proposed mTLS via Cilium's built-in mutual TLS for service-to-service communication plus JWT for external clients. The Pessimist countered that Cilium's mTLS requires enabling mutual authentication with SPIFFE identity management, which is not zero-configuration and may not already be enabled. The simpler alternative: Cilium NetworkPolicies (CRDs already available in-cluster) restrict pod-to-pod traffic at the network level — only authorized services can reach each other — without SPIFFE operational burden. Both agreed on JWT for external clients.

**Decision**: JWT for external client authentication (web, mobile, Morgan). Cilium NetworkPolicies for internal service isolation (network-level zero-trust). No application-level mTLS configuration required.

**Consensus**: 2/2 (100% — converged on Pessimist's NetworkPolicy approach as the pragmatic choice)

**Consequences**:
- **Positive**: Network-level isolation without SPIFFE complexity; JWT is well-understood for external auth; CiliumNetworkPolicy CRDs already available
- **Negative**: No cryptographic identity at the application layer between services (network-level isolation only)
- **Caveats**: If SPIFFE/mTLS is already configured in-cluster, it can be enabled opportunistically during Task 10. The decision here is to not *require* it. Task 10 (production hardening) must define and enforce CiliumNetworkPolicies restricting which services can talk to which

---

### [D7] How should API versioning be handled?

**Status**: Accepted

**Task Context**: Tasks 2, 3, 4, 5, 6, 8, 9 (all services exposing APIs, and frontend clients consuming them)

**Context**: Both debaters agreed. The PRD already specifies `/api/v1/` for every endpoint. This is the convention used by Stripe, GitHub, Google Cloud, and Twilio.

**Decision**: URI-based versioning (`/api/v1/...`) for all REST endpoints.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Visible in URLs and access logs; browser-testable; matches PRD specification exactly
- **Negative**: None raised
- **Caveats**: None — the PRD had already decided this

---

### [D8] Should Finance and Customer Vetting be separate services or merged?

**Status**: Accepted

**Task Context**: Tasks 4, 5 (Finance and Customer Vetting services)

**Context**: The Optimist argued for separate services citing different scaling profiles and failure domains — Vetting makes slow external API calls while Finance handles latency-sensitive Stripe webhooks. The Pessimist countered that Vetting has only 3 endpoints, handles single-digit daily requests, and merging would halve the operational footprint (deployments, HPA configs, CI pipelines, container images) with trivial refactoring risk since both use Rust/Axum and the Rex agent.

**Decision**: Keep Finance and Customer Vetting as **separate microservices** as decomposed in the initial task plan.

**Consensus**: This decision follows the initial decomposition. The Optimist's failure-domain argument (external API timeouts in Vetting should not impact invoice processing) is the deciding factor given the PRD's explicit separation and the 99.9% uptime requirement for Finance. The Pessimist acknowledged the refactoring path is easy if the merge is later desired.

**Consequences**:
- **Positive**: Independent deployability; failure isolation between latency-sensitive Finance and unreliable external-API-dependent Vetting; matches PRD service boundaries
- **Negative**: Two deployments, two CI pipelines, two container images for services sharing the same agent (Rex) and stack (Rust/Axum)
- **Caveats**: The Pessimist's point about operational overhead is valid — if Vetting remains at single-digit daily requests, consolidation can be revisited. Both services should share common Rust libraries (error handling, auth middleware, metrics) to minimize duplication

---

### [D9] How should Signal integration for Morgan be handled?

**Status**: Accepted

**Task Context**: Tasks 1, 7 (infrastructure provisioning and Morgan agent)

**Context**: Both debaters agreed there is no viable third-party Signal messaging SaaS — Signal's protocol is intentionally closed to commercial resale. Signal-CLI is the only production-grade option. They differed on deployment topology: the Optimist suggested sidecar or separate pod; the Pessimist argued for a separate pod because Signal-CLI maintains persistent state (registration keys) that must survive Morgan pod restarts.

**Decision**: Self-hosted Signal-CLI deployed as a **separate pod** (not sidecar) with a PersistentVolumeClaim for registration state. Accessed via JSON-RPC over cluster networking.

**Consensus**: 2/2 (100% on Signal-CLI; Pessimist's separate-pod argument prevailed on topology)

**Consequences**:
- **Positive**: Signal-CLI state persists across Morgan restarts; independent lifecycle management; JSON-RPC interface is simple to consume
- **Negative**: Additional pod to manage; network hop between Morgan and Signal-CLI
- **Caveats**: Signal-CLI must be properly registered with a phone number before first use — this is a manual setup step during Task 1

---

### [D10] Primary navigation paradigm for web and mobile?

**Status**: Accepted

**Task Context**: Tasks 8, 9 (website and mobile app)

**Context**: Both debaters agreed immediately. This is standard responsive design — a sidebar on mobile wastes screen real estate, and tabs on desktop waste vertical space.

**Decision**: Tab-based navigation for mobile (Expo bottom tabs), sidebar navigation for web (shadcn/ui sidebar component). Responsive hybrid pattern — same route structure, platform-appropriate chrome.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Platform-native UX on both web and mobile; shared navigation structure enables consistent user mental model
- **Negative**: None raised
- **Caveats**: None

---

### [D11] How should the design system be managed for web and mobile?

**Status**: Accepted

**Task Context**: Tasks 8, 9 (website and mobile app)

**Context**: Both debaters rejected Tamagui (still maturing, adds significant complexity) and converged on shadcn/ui for web with shared Tailwind design tokens consumed by NativeWind for mobile. NativeWind 4 brings TailwindCSS to React Native, so the same color, spacing, and typography tokens defined once in a shared Tailwind config can be consumed by both platforms.

**Decision**: shadcn/ui for web (Next.js); NativeWind 4 for mobile (Expo). Shared Tailwind config package defining design tokens (colors, spacing, radii, typography). No cross-platform component library — platform-appropriate rendering with shared design language.

**Consensus**: 2/2 (100%)

**Consequences**:
- **Positive**: Consistent visual identity across web and mobile without runtime code sharing; NativeWind 4 is stable enough for this scope; avoids Tamagui complexity
- **Negative**: Mobile components must be built separately (no component sharing with web)
- **Caveats**: The shared Tailwind config should be a separate package in the monorepo or published to a private registry so both apps import it

---

### [D12] What approach for GDPR data export and customer deletion?

**Status**: Accepted

**Task Context**: Tasks 2–10 (every service must support GDPR operations)

**Context**: The Optimist proposed a centralized compliance service orchestrating data export/deletion across all services, citing the risk of distributed GDPR endpoints where one failure means violation. The Pessimist agreed on centralized orchestration but argued Morgan already orchestrates cross-service workflows via MCP tools — adding `sigma1_gdpr_export` and `sigma1_gdpr_delete` MCP tools is zero new services, zero new deployments. The compliance operation runs maybe once a quarter.

**Decision**: GDPR orchestration via Morgan using dedicated MCP tools (`sigma1_gdpr_export`, `sigma1_gdpr_delete`). Each service exposes internal GDPR endpoints (data export and deletion for its schema). Morgan orchestrates calls to all services, logs completions, and reports failures. No new dedicated compliance service.

**Consensus**: 2/2 (100% — converged on Morgan-orchestrated approach)

**Consequences**:
- **Positive**: Zero new services or deployments; leverages existing Morgan orchestration infrastructure; audit trail through Morgan's standard logging; same centralized coordination the Optimist wanted
- **Negative**: Morgan becomes a dependency for GDPR compliance operations
- **Caveats**: Each service (Tasks 2–6) must implement internal GDPR endpoints as part of their API surface. Task 7 (Morgan) must add the two GDPR MCP tools. Task 10 (production hardening) should verify the full deletion pipeline works end-to-end and produces an audit log entry

---

## 4. Escalated Decisions

No decisions were escalated. All 12 decision points reached consensus.

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Version/Detail |
|-------|-----------|----------------|
| **Database** | PostgreSQL | 16 via CloudNative-PG operator |
| **Cache** | Valkey | 7.2-alpine via Opstree Redis operator |
| **Object Storage** | Cloudflare R2 | S3-compatible, zero egress |
| **CDN/Ingress** | Cloudflare | Pages (website), Tunnel (Morgan) |
| **Container Orchestration** | Kubernetes | Existing cluster with Cilium CNI |
| **Observability** | Grafana + Loki + Prometheus | Existing OpenClaw stack |

### Service Architecture

```
                 ┌─────────────────────────────────┐
                 │         External Clients          │
                 │   Signal │ Voice │ Web │ Mobile   │
                 └────────────────┬──────────────────┘
                                  │ JWT Auth
                 ┌────────────────▼──────────────────┐
                 │          Morgan (OpenClaw)          │
                 │       MCP Tools → HTTP/REST         │
                 └──┬──────┬──────┬──────┬──────┬────┘
                    │      │      │      │      │
          ┌────────▼┐ ┌───▼────┐ ┌▼─────┐ ┌───▼────┐ ┌▼──────┐
          │ Catalog  │ │  RMS   │ │Finance│ │Vetting │ │Social │
          │Rust/Axum │ │Go/gRPC │ │Rust/  │ │Rust/   │ │Node/  │
          │  REST    │ │+gateway│ │Axum   │ │Axum    │ │Elysia │
          │          │ │ REST   │ │REST   │ │REST    │ │REST   │
          └────┬─────┘ └───┬────┘ └──┬───┘ └───┬────┘ └──┬───┘
               │           │         │         │         │
     ┌─────────▼───────────▼─────────▼─────────▼─────────▼──────┐
     │              PostgreSQL 16 (CloudNative-PG)                │
     │   Schemas: catalog | rms | finance | vetting | social | audit │
     └──────────────────────────────────────────────────────────────┘
     ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
     │  Valkey 7.2   │  │ Cloudflare R2 │  │  Signal-CLI   │
     │  (cache/RL)   │  │ (object store)│  │ (separate pod)│
     └──────────────┘  └──────────────┘  └──────────────┘
```

### Communication Patterns

- **External → Services**: JWT-authenticated REST over HTTPS (via Cloudflare Tunnel/CDN)
- **Morgan → Backend Services**: HTTP/REST via MCP tool-server (Morgan calls each service's REST endpoints)
- **Service → Service**: HTTP/REST (direct in-cluster); RMS exposes both gRPC (native) and REST (via grpc-gateway) — all consumers use the REST gateway
- **Frontend → Backend**: REST over HTTPS, URI-versioned (`/api/v1/...`)
- **Network Isolation**: CiliumNetworkPolicies restrict pod-to-pod traffic

### Key Patterns

- **Single database, multiple schemas**: One CNPG cluster, per-service schemas with role-based access
- **REST-first API design**: All services expose REST endpoints; RMS additionally supports native gRPC
- **Centralized AI orchestration**: Morgan (OpenClaw) coordinates all cross-service workflows including GDPR
- **Platform-appropriate frontends**: shadcn/ui + TailwindCSS 4 for web; NativeWind 4 for mobile; shared Tailwind token config
- **GitOps deployment**: ArgoCD with automatic rollbacks

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| Polyglot gRPC across all services | Unnecessary complexity; debuggability concerns; <100 RPS internal traffic makes performance argument irrelevant |
| Separate PostgreSQL databases per service | Operational overhead without benefit for single-tenant platform |
| Application-level mTLS (SPIFFE) | Cilium NetworkPolicies provide network-level isolation without SPIFFE operational burden; mTLS can be added opportunistically if already configured |
| Tamagui cross-platform design system | Still maturing; NativeWind 4 with shared Tailwind tokens achieves the same design consistency |
| Dedicated GDPR compliance service | Morgan already orchestrates cross-service workflows; adding MCP tools is zero new deployments |
| Third-party Signal SaaS | Does not exist; Signal's protocol prevents commercial resale |
| Header-based API versioning | URI-based already specified in PRD; header-based adds debugging complexity |
| Merging Finance and Vetting services | Different failure domains; external API timeouts in Vetting should not impact invoice processing |

---

## 6. Implementation Constraints

### Security Requirements

- **External authentication**: JWT tokens for all client-facing APIs (web, mobile, Morgan web chat)
- **Internal isolation**: CiliumNetworkPolicies must be defined and enforced for every service (Task 10)
- **Secret management**: All API keys (Stripe, OpenCorporates, LinkedIn, Google, ElevenLabs, Twilio) stored as Kubernetes Secrets with documented rotation schedule
- **Security scanning**: Semgrep, CodeQL, Snyk/Dependabot — critical/high severity issues block merge
- **OWASP compliance**: Enforced by Cipher agent in CI pipeline

### Performance Targets

- Morgan response time: < 10 seconds for simple queries
- Equipment availability check: < 500ms
- Invoice generation: < 5 seconds
- Quote-to-invoice workflow: < 2 minutes end-to-end
- Signal connections: 500+ concurrent

### Operational Requirements

- **Uptime**: 99.9% for production services
- **Replicas**: ≥2 replicas for all production services (Task 10)
- **Observability**: All services expose `/metrics` (Prometheus), `/health/live`, `/health/ready`
- **Logging**: Grafana + Loki (existing OpenClaw stack)
- **Deployment**: GitOps via ArgoCD with automatic rollbacks
- **Test coverage**: Minimum 80% code coverage required (enforced by Tess agent)

### Service Dependencies and Integration Points

| Service | External Dependencies |
|---------|----------------------|
| Morgan | Signal-CLI (JSON-RPC), ElevenLabs API, Twilio API, All backend REST APIs |
| Equipment Catalog | PostgreSQL, Valkey, Cloudflare R2 |
| RMS | PostgreSQL, Valkey, Google Calendar API |
| Finance | PostgreSQL, Valkey, Stripe API |
| Customer Vetting | PostgreSQL, OpenCorporates API, LinkedIn API, Google Reviews, Credit APIs |
| Social Engine | PostgreSQL, Cloudflare R2, Instagram Graph API, LinkedIn API, Facebook Graph API, OpenAI/Claude |
| Website | Cloudflare Pages, Equipment Catalog API, Morgan (web chat) |
| Mobile App | Equipment Catalog API, Morgan (web chat) |

### Organizational Preferences

- Prefer existing in-cluster operators and services (Valkey, CloudNative-PG, Cilium) over deploying new instances
- Prefer Cloudflare ecosystem (R2, Pages, Tunnel) for CDN, storage, and ingress
- Self-hosted Signal-CLI (no SaaS alternative exists)
- Single-cluster deployment (multi-region explicitly a non-goal)
- Trading Desk / Python services deferred to Phase 2

### GDPR Compliance

- Every service (Tasks 2–6) must implement internal GDPR data export and deletion endpoints
- Morgan (Task 7) orchestrates compliance via `sigma1_gdpr_export` and `sigma1_gdpr_delete` MCP tools
- Task 10 must verify full deletion pipeline end-to-end and confirm audit log entries are generated

---

## 7. Design Intake Summary

### Frontend Detection

- **hasFrontend**: `true`
- **frontendTargets**: `web` | `mobile`
- **Provider mode**: Stitch
- **Stitch status**: `generated` — design artifacts have been produced
- **Framer status**: `skipped` (not requested)

### Supplied Design Artifacts & References

- **Existing website**: https://sigma-1.com (current site for visual reference)
- **Existing platform**: https://deployiq.maximinimal.ca (prior platform reference)
- No additional Figma files, screenshots, or brand guideline documents were supplied in the design context

### Provider Generation Status

- **Stitch**: Generated design artifacts. Implementing agents (Blaze for web, Tap for mobile) should reference Stitch outputs for visual direction on page layouts, component styling, and responsive breakpoints
- **Framer**: Not requested; skipped

### Implications for Implementation

**Web (Task 8 — Blaze)**:
- Next.js 15 App Router with React 19
- shadcn/ui component library + TailwindCSS 4 for styling
- Effect 3.x for type-safe data fetching and form validation
- TanStack Query for server state management
- Hosted on Cloudflare Pages (static + SSR)
- Must implement AI-native optimizations: `llms.txt`, `llms-full`, Schema.org structured data
- Sidebar navigation pattern for desktop; responsive collapse for smaller viewports
- Morgan web chat widget embedded on all pages

**Mobile (Task 9 — Tap)**:
- Expo with React Native and TypeScript
- NativeWind 4 for styling (consuming shared Tailwind tokens)
- Effect for validation and data fetching
- Bottom tab navigation pattern
- Feature parity with web for core flows (catalog browsing, quote requests, Morgan chat)
- Push notifications for quote status updates

**Shared Design Tokens**:
- A shared Tailwind configuration package must be created defining: color palette, spacing scale, border radii, typography (font families, sizes, weights)
- Both web (shadcn/ui) and mobile (NativeWind) consume this shared config
- This is the design system — not a shared component library, but shared design language

---

## 7a. Selected Design Direction

No `design_selections` were provided. Implementing agents should follow the Stitch-generated artifacts and the PRD's specified technology stack (shadcn/ui, TailwindCSS 4, React 19) as the visual baseline. If Stitch outputs include specific screen designs, those should be treated as the design target.

---

## 7b. Design Deliberation Decisions

No `design_deliberation_result` was provided. The following design decisions were resolved during the technical deliberation and apply to frontend implementation:

| Decision | Category | Resolution |
|----------|----------|------------|
| Navigation paradigm (D10) | `ux-behavior` | Sidebar for web, bottom tabs for mobile |
| Design system (D11) | `design-system` | shadcn/ui (web) + NativeWind 4 (mobile) with shared Tailwind token config |

