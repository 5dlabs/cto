# Enhanced PRD

## 1. Original Requirements

> # Project: Sigma-1 — Unified AI Business Platform for Perception Events
>
> - **Website:** https://sigma-1.com
> - **Existing Platform:** https://deployiq.maximinimal.ca
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

The initial task decomposition identified **10 tasks** spanning infrastructure, backend services, an AI agent, a frontend website, and production hardening.

| Task ID | Title | Agent | Stack | Priority | Dependencies |
|---------|-------|-------|-------|----------|--------------|
| 1 | Bootstrap Core Infrastructure | Bolt | Kubernetes/Helm | High | None |
| 2 | Equipment Catalog Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 3 | RMS Service | Grizz | Go 1.22+/gRPC | High | Task 1 |
| 4 | Finance Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 5 | Customer Vetting Service | Rex | Rust 1.75+/Axum 0.7 | High | Task 1 |
| 6 | Social Media Engine | Nova | Node.js 20+/Elysia + Effect | Medium | Task 1 |
| 7 | Morgan AI Agent | Angie | OpenClaw/MCP | High | Tasks 2, 3, 4, 5, 6 |
| 8 | Website Frontend | Blaze | Next.js 15/React 19/Effect | High | Tasks 2, 7 |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | High | Tasks 2–8 |
| 10 | Production Hardening: RBAC, Secrets, Audit | Bolt | Kubernetes/Helm | High | Task 9 |

### Key Services & Components

- **3 Rust/Axum services** (Equipment Catalog, Finance, Customer Vetting) — all using the Rex agent
- **1 Go/gRPC service** (RMS) — Grizz agent
- **1 Node.js/Elysia service** (Social Media Engine) — Nova agent
- **1 AI Agent** (Morgan) — OpenClaw runtime with MCP tools
- **1 Next.js frontend** (Website) — Blaze agent
- **2 infrastructure tasks** (Bootstrap + Production Hardening) — Bolt agent

### Agent Assignments

| Agent | Role | Tasks |
|-------|------|-------|
| Bolt | DevOps/Infrastructure | 1, 9, 10 |
| Rex | Rust backend services | 2, 4, 5 |
| Grizz | Go backend service | 3 |
| Nova | Node.js backend service | 6 |
| Angie | AI Agent orchestration | 7 |
| Blaze | Frontend | 8 |

### Cross-Cutting Concerns

- **14 decision points** were identified across tasks covering: platform choices (Redis engine, object storage, CDN), API design (inter-service paradigm, versioning), data model (schema separation, currency), security (service auth, access control), service topology (Finance/Vetting separation), build-vs-buy (Signal integration), UX behavior (chat widget), design system, and component library choices
- **Shared infrastructure** (PostgreSQL, Valkey, S3/R2, Cloudflare) is a dependency for all backend services
- **Polyglot architecture** (Rust, Go, Node.js, TypeScript) creates coordination overhead for shared concerns like auth and proto definitions
- **GDPR compliance** touches multiple services (Finance, Vetting, audit logging)

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine should be used for caching, rate limiting, and session storage?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 3, 4, 9 (Bootstrap Infrastructure, Equipment Catalog, RMS, Finance, Production Hardening)

**Context:** Both debaters agreed immediately. The existing Valkey operator (`redis.redis.opstreelabs.in` CRD) is already deployed in-cluster. No debate was necessary.

**Decision:** Use the existing Valkey operator, provisioning a dedicated Valkey instance for Sigma-1 via a new `Redis` CR in the `sigma1` namespace. Image: `valkey/valkey:7.2-alpine`.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Zero new operator overhead; single CRD apply; operator-managed lifecycle (upgrades, reconciliation); Valkey 7.2 is fully Redis-API compatible
- *Negative:* None identified
- *Caveats:* None

---

### [D2] Which object storage provider should be used for product images and social media photos?

**Status:** Accepted

**Task Context:** Tasks 1, 2, 6 (Bootstrap Infrastructure, Equipment Catalog, Social Media Engine)

**Context:** Both debaters agreed. Cloudflare R2 offers zero egress fees, S3-API compatibility, and the Cloudflare operator is already deployed in-cluster.

**Decision:** Cloudflare R2 with native CDN integration.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Zero egress fees for an image-heavy catalog (533+ products + social photos); S3-compatible SDK support across Rust/Go/Node; single-vendor path with existing Cloudflare operator
- *Negative:* Vendor lock-in to Cloudflare ecosystem (mitigated by S3-compatible API)
- *Caveats:* None

---

### [D7] How should versioning be handled for public and internal APIs?

**Status:** Accepted

**Task Context:** Tasks 2, 3, 4, 5, 6, 8 (all API-serving services and website)

**Context:** The PRD already uses URI-based versioning (`/api/v1/...`) consistently across all service specifications. Both debaters agreed this should be formalized.

**Decision:** URI-based versioning (`/api/v1/...`) as already specified in the PRD.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Version visible in access logs, Prometheus metrics labels, curl commands, and network policies; most operationally transparent approach
- *Negative:* URL sprawl if many versions accumulate (unlikely for v1 platform)
- *Caveats:* None

---

### [D8] Self-hosted Signal-CLI or managed gateway?

**Status:** Accepted

**Task Context:** Tasks 1, 7 (Bootstrap Infrastructure, Morgan AI Agent)

**Context:** Hard constraint in PRD. Signal has no official managed gateway API. Third-party Signal gateways violate Signal's Terms of Service. Both debaters agreed.

**Decision:** Self-hosted Signal-CLI as sidecar to Morgan (Task 7), per PRD hard constraint.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* ToS-compliant; self-contained; already proven in cluster (cto/openclaw-morgan deployed)
- *Negative:* Operational overhead of managing Signal-CLI lifecycle (registration, updates)
- *Caveats:* None

---

### [D9] CDN and TLS termination solution?

**Status:** Accepted

**Task Context:** Tasks 1, 7, 8, 9 (Bootstrap Infrastructure, Morgan, Website, Production Hardening)

**Context:** Both debaters agreed. The Cloudflare operator is already deployed (`cloudflare-operator-system` namespace, `clustertunnels.networking.cfargotunnel.com` CRD). Cloudflare Tunnel eliminates public port exposure.

**Decision:** Cloudflare Tunnel + CDN (existing operator in-cluster).

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Zero public port exposure; unified Cloudflare stack with R2; operator-managed tunnel lifecycle
- *Negative:* Cloudflare dependency for all ingress
- *Caveats:* None

---

### [D11] How should multi-currency be modeled?

**Status:** Accepted

**Task Context:** Task 4 (Finance Service)

**Context:** Both debaters agreed. The PRD data model specifies `i64` cents with `currency: String`. This matches Stripe's representation, eliminating conversion errors. IEEE 754 floating-point issues are well-documented in financial systems.

**Decision:** Integer cents with explicit currency fields, exactly as PRD specifies. Use `rust_decimal` for arithmetic when needed for reports.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Matches Stripe's model; eliminates floating-point precision bugs; PRD-aligned
- *Negative:* Requires cent↔display conversion logic in all UIs
- *Caveats:* None

---

### [D12] Morgan web chat interaction pattern?

**Status:** Accepted

**Task Context:** Tasks 7, 8 (Morgan AI Agent, Website Frontend)

**Context:** Both debaters agreed. A persistent widget lets visitors browse equipment while chatting — critical for the quote-builder flow where Morgan assists in real-time.

**Decision:** Embedded persistent chat widget (Intercom-style) with session continuity via localStorage token + server-side session (Redis/Valkey-backed).

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Conversations survive page navigation; matches Signal experience; enables real-time quote assistance
- *Negative:* Additional frontend complexity for widget state management
- *Caveats:* None

---

### [D13] shadcn/ui usage strategy?

**Status:** Accepted

**Task Context:** Task 8 (Website Frontend)

**Context:** Both debaters agreed. shadcn/ui is copy-paste by design — meant to be customized. The quote builder and chat widget need custom components that don't exist in the standard library.

**Decision:** Extend shadcn/ui with custom tokens/variants for Sigma-1 branding, plus custom components for quote builder and chat widget. Use standard shadcn for tables, forms, navigation.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* Speed on standard UI; freedom on differentiated flows; follows shadcn/ui's intended usage pattern
- *Negative:* Custom components require maintenance independent of shadcn updates
- *Caveats:* None

---

### [D14] Data table and form libraries for website?

**Status:** Accepted

**Task Context:** Task 8 (Website Frontend)

**Context:** Both debaters agreed. shadcn/ui already ships with TanStack Table and React Hook Form examples. These are the ecosystem-standard choices.

**Decision:** TanStack Table v8 + React Hook Form v7 with Effect Schema integration, leveraging shadcn/ui's existing integrations.

**Consensus:** 2/2 (100%)

**Consequences:**
- *Positive:* TanStack Table handles filtering/sorting/pagination for 533+ products; React Hook Form's `useFieldArray` handles dynamic quote line items; largest community support
- *Negative:* None identified
- *Caveats:* None

---

### [D4] Should Finance and Customer Vetting be separate deployments or merged?

**Status:** Accepted

**Task Context:** Tasks 4, 5 (Finance Service, Customer Vetting Service)

**Context:** Both debaters agreed on separate deployments but the Pessimist strengthened the position by requiring a Rust workspace mono-repo with shared crates to avoid code duplication across the three Rex services.

**Decision:** Separate deployments, organized in a Rust workspace mono-repo with shared crates for auth, database connection pooling, and error handling — producing separate binaries while maintaining a single CI pipeline.

**Consensus:** 2/2 (100%) — Pessimist's refinement accepted

**Consequences:**
- *Positive:* Security boundary between financial data and third-party API calls; independent scaling; Cargo workspace keeps single CI pipeline
- *Negative:* Six separate backend deployments to monitor (Equipment Catalog, RMS, Finance, Vetting, Social Engine, Morgan) — operational surface area
- *Caveats:* The Pessimist raised the valid concern that six deployments means six health checks, six scaling policies, six alert rule sets. This is manageable with GitOps (ArgoCD already in cluster) but requires disciplined observability setup in Tasks 9-10.

---

## 4. Escalated Decisions

### [D3] What API paradigm should be used for inter-service communication? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 2, 3, 4, 5, 7 (Equipment Catalog, RMS, Finance, Vetting, Morgan)

**Options:**
- **Option A (Optimist):** gRPC for all internal service-to-service calls, with REST (via grpc-gateway or native Axum) only for public/edge APIs and MCP tool endpoints
- **Option B (Pessimist):** REST/HTTP everywhere, with RMS exposing REST via grpc-gateway as already specified. Internal gRPC only where streaming is genuinely needed (none identified in v1).

**Optimist argued:** gRPC gives binary serialization (~10x smaller payloads), bidirectional streaming for real-time inventory updates, and codegen for all three languages (tonic/Rust, native Go, nice-grpc/Node). The RMS is already gRPC-native. Persistent HTTP/2 connections and binary framing directly serve the sub-500ms availability SLA. A thin MCP-to-gRPC adapter gives typed contracts for Morgan's tool calls.

**Pessimist argued:** Three-language gRPC codegen creates a coordination tax (three CI pipelines for proto compilation). gRPC endpoints can't be debugged with `curl` — you need `grpcurl` plus local proto files. Binary payloads don't show in standard HTTP access logs. The 500ms SLA is database-bound, not serialization-bound (in-cluster latency is sub-millisecond). Morgan's MCP tools are HTTP-native, so gRPC→grpc-gateway→REST→MCP adds protocol translation layers. REST is simpler to debug, log, and operate for a small team.

**Recommendation:** The Pessimist's operational arguments are compelling for a small team. The 500ms SLA is unlikely to be serialization-bound. However, the RMS is already gRPC-native per PRD. A pragmatic middle ground: **RMS stays gRPC internally with grpc-gateway REST for external consumers; all other services expose and consume REST/HTTP.** This avoids multi-language proto coordination while respecting the RMS's existing design. Morgan calls all services via their REST endpoints. If streaming becomes necessary in v2, gRPC can be added incrementally.

---

### [D5] How should multi-tenancy and schema separation be handled in PostgreSQL? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 1, 2, 3, 4, 5, 6 (all data-persisting services)

**Options:**
- **Option A (Optimist):** Separate schemas within a single CNPG cluster, with per-service database users scoped to their schema. Allows cross-schema JOINs where needed (e.g., opportunity→invoice).
- **Option B (Pessimist):** Separate schemas in a single CNPG cluster, with per-service connection pool credentials with schema-scoped permissions, AND a strict rule that no cross-schema JOINs are allowed — cross-service data access goes through APIs only.

**Optimist argued:** Single CNPG cluster with schema isolation gives cross-service transactional consistency where needed, shared connection pooling, and one backup/restore lifecycle. Per-service Postgres users with `USAGE` grants enforce isolation. Separate databases would mean 5x the backup jobs, monitoring, and failover complexity.

**Pessimist argued:** The Optimist's "cross-service transactional consistency" is the footgun. The moment Finance JOINs against the RMS schema, you've created invisible coupling that makes independent schema migrations impossible. When RMS needs to alter the `opportunities` table, it breaks Finance queries silently. API-only cross-service access prevents this coupling while keeping the backup simplicity.

**Recommendation:** The Pessimist's position is architecturally sounder. Both agree on single CNPG cluster with separate schemas — the disagreement is only about cross-schema JOINs. **Adopt the Pessimist's position: single CNPG cluster, separate schemas, per-service credentials with schema-scoped permissions, and a strict no cross-schema JOIN rule.** Cross-service data access goes through APIs. This preserves migration independence, which is critical for a polyglot team where Go and Rust services evolve at different paces.

---

### [D6] What auth mechanism for internal service-to-service API calls? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 1, 2, 3, 4, 5, 6, 7, 9, 10 (all services, infrastructure)

**Options:**
- **Option A (Optimist):** mTLS via Cilium's built-in mutual authentication, supplemented by service identity headers for audit logging
- **Option B (Pessimist):** Network policies via Cilium for service isolation, plus lightweight JWT service tokens (issued at deploy time via External Secrets) for audit logging. No mTLS at the application layer.

**Optimist argued:** Cilium supports transparent mTLS via its service mesh layer — no sidecar proxies, no cert-manager complexity. Cryptographic service identity verification with zero application code changes. Stronger than JWTs (no shared secrets to leak). Network policies alone aren't sufficient for GDPR compliance.

**Pessimist argued:** Cilium's transparent mTLS adds an invisible encryption layer that makes packet captures and debugging significantly harder. mTLS failures surface as connection resets with no application-level error. For a small team, Cilium network policies provide namespace/pod-level isolation (CRDs already present). A simple shared JWT with service identity claims (issued at deploy time via External Secrets, also already in-cluster) provides the GDPR audit trail without the mTLS debugging tax.

**Recommendation:** Both positions provide adequate security for a single-tenant, in-cluster platform. The key question is operational: how frequently will the team need to debug inter-service calls? For a v1 launch with a small team, **the Pessimist's approach (Cilium network policies + JWT service tokens) is lower-risk operationally.** mTLS can be enabled later as a hardening step once the platform is stable. The JWT approach provides the GDPR-required audit trail while keeping debugging straightforward.

---

### [D10] Access control model for admin endpoints? — ESCALATED

**Status:** Pending human decision

**Task Context:** Tasks 2, 3, 4, 5, 6, 10 (all backend services, production hardening)

**Options:**
- **Option A (Optimist):** RBAC with fine-grained roles per service, implemented via a shared auth library (Rust crate for Rex services, Go package for Grizz)
- **Option B (Pessimist):** Simple RBAC with 3-4 roles (admin, operator, morgan-agent, readonly), implemented per-service with shared protobuf/schema definitions but NOT a shared library across languages

**Optimist argued:** A shared Rust crate and Go package keeps implementation consistent. RBAC with roles like `admin`, `operator`, `morgan-agent`, `readonly` covers all current needs. Consistency across services prevents authorization bugs.

**Pessimist argued:** A "shared auth library" across Rust and Go means maintaining the same authorization logic in two languages with two dependency trees. When a role definition changes, you're coordinating releases across all services. Instead: define roles in a shared protobuf or JSON schema, implement validation per-service. Fewer moving parts, same outcome.

**Recommendation:** Both agree on RBAC with 3-4 roles — the disagreement is about implementation strategy. **The Pessimist's approach (shared role schema, per-service implementation) is more practical for a polyglot codebase.** A shared JSON schema or protobuf enum defining the role taxonomy gives a single source of truth without the cross-language library maintenance burden. Each service validates roles using its own middleware but references the same role definitions.

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Decision Basis |
|-------|-----------|----------------|
| **Database** | PostgreSQL 16 via CloudNative-PG (single cluster, multi-schema) | PRD + D5 |
| **Cache** | Valkey 7.2 via existing Opstree operator | D1 |
| **Object Storage** | Cloudflare R2 | D2 |
| **CDN/Ingress** | Cloudflare Tunnel + CDN | D9 |
| **Signal** | Self-hosted Signal-CLI (sidecar) | D8 (hard constraint) |
| **API Versioning** | URI-based `/api/v1/...` | D7 |
| **Currency** | Integer cents (`i64`) with explicit currency field | D11 |
| **Web Chat** | Persistent Intercom-style widget, Valkey-backed sessions | D12 |
| **Frontend Components** | shadcn/ui (extended) + TanStack Table v8 + React Hook Form v7 | D13, D14 |

### Service Architecture

Six backend deployments organized by domain and language:

1. **Equipment Catalog** (Rust/Axum 0.7) — Rex agent — Product CRUD, availability, rate limiting
2. **RMS** (Go 1.22+/gRPC + grpc-gateway) — Grizz agent — Opportunities, projects, inventory, crew, delivery
3. **Finance** (Rust/Axum 0.7) — Rex agent — Invoicing, payments, payroll, Stripe
4. **Customer Vetting** (Rust/Axum 0.7) — Rex agent — Background checks, lead scoring
5. **Social Media Engine** (Node.js 20+/Elysia + Effect) — Nova agent — Content curation, publishing
6. **Morgan AI Agent** (OpenClaw) — MCP tools calling all services

**Rust services** (Equipment Catalog, Finance, Vetting) are organized as a **Cargo workspace mono-repo** with shared crates for auth middleware, database connection pooling, and error handling (per D4 refinement). Each produces a separate binary and is independently deployable.

### Communication Patterns (Pending D3)

The inter-service API paradigm is **escalated and pending human decision**. Until resolved:
- The RMS is definitively gRPC-native with grpc-gateway REST exposure (per PRD)
- Morgan's MCP tools are HTTP-based
- The recommended middle ground is REST for all cross-service calls, with RMS's grpc-gateway providing the REST interface

### Data Architecture (Pending D5)

Schema separation is **escalated and pending human decision**. Until resolved:
- Both positions agree: single CNPG cluster, separate schemas (`rms`, `crm`, `finance`, `audit`, `public`)
- The recommended position is: per-service credentials, schema-scoped permissions, no cross-schema JOINs

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| AWS S3 for object storage | Cloudflare R2 has zero egress, operator already in-cluster (D2) |
| Bitnami Redis Helm chart | Valkey operator already deployed; don't add a second lifecycle (D1) |
| NGINX ingress controller | Would duplicate TLS termination; Cloudflare Tunnel eliminates public ports (D9) |
| Managed Signal gateway | Violates Signal ToS; no official managed API exists (D8) |
| Header-based API versioning | Hides routing logic, complicates debugging (D7) |
| Decimal/float currency storage | IEEE 754 precision issues; Stripe uses cents (D11) |
| ABAC for access control | Over-engineered for single-tenant platform (debate consensus) |
| Trading Desk (Phase 1) | Python not in core stack; explicitly deferred to Phase 2 (PRD) |
| Multi-region deployment | Non-goal per PRD |

---

## 6. Implementation Constraints

### Security Requirements

- **GDPR compliance** is mandatory: data export capability, customer deletion, audit trails for all data access
- All services must implement RBAC with at minimum: `admin`, `operator`, `morgan-agent`, `readonly` roles (pending D10 implementation strategy)
- Service-to-service authentication is required for audit logging (pending D6 mechanism)
- Critical/high severity security vulnerabilities block merge (Cipher agent enforcement)
- Secrets must be managed via External Secrets operator (already in-cluster) with automated rotation

### Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| Morgan response time (simple queries) | < 10 seconds | PRD |
| Equipment availability check | < 500ms | PRD |
| Invoice generation | < 5 seconds | PRD |
| Concurrent Signal connections | 500+ | PRD |
| Service uptime | 99.9% | PRD |
| Quote-to-invoice workflow | < 2 minutes | Success Criteria |

### Operational Requirements

- **GitOps** via ArgoCD (already in cluster) with automatic rollbacks on failure
- **Observability** via existing Grafana + Loki + Prometheus stack
- **CI/CD pipeline**: Stitch (code review) → Cleo (quality) → Tess (testing, ≥80% coverage) → Cipher (security) → Atlas (merge gate) → Bolt (deploy)
- **HA for production**: CloudNative-PG scaled to 3 instances with synchronous replication; Valkey sentinel/clustering
- All services must expose `/metrics` (Prometheus), `/health/live` (liveness), `/health/ready` (readiness)

### Service Dependencies & Integration Points

| Service | External Dependencies |
|---------|----------------------|
| Morgan | Signal-CLI, ElevenLabs, Twilio, all backend APIs |
| Equipment Catalog | PostgreSQL, Valkey, Cloudflare R2 |
| RMS | PostgreSQL, Valkey, Google Calendar API |
| Finance | PostgreSQL, Valkey, Stripe API |
| Customer Vetting | PostgreSQL, OpenCorporates API, LinkedIn API, Google Reviews, Credit APIs |
| Social Engine | PostgreSQL, Cloudflare R2, Instagram Graph API, LinkedIn API, Facebook Graph API, OpenAI/Claude |
| Website | Cloudflare Pages, Equipment Catalog API, Morgan (web chat), Social Engine API (portfolio) |

### Organizational Preferences

- **Prefer existing in-cluster operators** when available (Valkey, Cloudflare, External Secrets, CNPG)
- **Single-vendor Cloudflare path** for edge services (R2 + CDN + Tunnel)
- **Managed by 5D Labs** — not self-hosted deployment (non-goal)

---

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`:** true
- **`frontendTargets`:** web, mobile
- The PRD specifies a **Next.js 15 website** (Task 8) as the primary web target and mentions **Expo (mobile)** in the architecture diagram, though mobile is not explicitly tasked in v1
- Web is the active implementation target for Phase 1

### Supplied Design Artifacts & References

- **Existing platform:** https://deployiq.maximinimal.ca (current platform being replaced)
- **Target website:** https://sigma-1.com
- No explicit design mockups, Figma files, or brand guidelines were supplied in the PRD

### Provider Generation Status

- **Stitch (design generation):** Failed — no generated design candidates available
- **Framer:** Skipped (not requested)
- No normalized design candidates are available from automated providers

### Implications for Frontend Implementation (Task 8)

1. **No visual reference exists** beyond the existing platform (deployiq.maximinimal.ca) — implementing agents should audit the existing platform for brand colors, typography, and layout patterns as a baseline
2. **shadcn/ui extended with custom branding** (per D13) means the implementing agent needs to define Sigma-1 brand tokens (colors, typography, spacing) as part of the Tailwind/shadcn configuration
3. **Custom components** are required for the quote builder (drag-and-drop item selection, date range pickers with availability overlays, real-time pricing) and chat widget (streaming text display)
4. **TanStack Table v8** will power the equipment catalog's filtering, sorting, pagination, and column visibility for 533+ products
5. **React Hook Form v7** with Effect Schema integration handles the quote builder's dynamic line items via `useFieldArray`
6. The **Morgan web chat widget** must be persistent across page navigation with session continuity (localStorage + Valkey-backed server session)
7. **Mobile (Expo)** is shown in the architecture but not explicitly tasked — implementing agents should ensure the API design supports mobile clients but should not build the mobile app in Phase 1

### 7a. Selected Design Direction

No design selections were provided. The implementing agent (Blaze) should:
- Reference the existing platform at https://deployiq.maximinimal.ca for current brand identity
- Define a modern, professional design system appropriate for a lighting/visual production company
- Use shadcn/ui defaults as the starting point, customizing tokens for Sigma-1 branding

### 7b. Design Deliberation Decisions

No design deliberation was conducted (no `design_deliberation_result` provided). Design decisions for visual identity, color palette, typography, and layout patterns are left to the implementing agent's judgment, constrained by:
- shadcn/ui as the component foundation (D13)
- TailwindCSS 4 for styling
- The hybrid approach of standard shadcn components + custom components for quote builder and chat (D13)

---

## 8. Open Questions

The following items are non-blocking but should be addressed by implementing agents using their best judgment:

1. **Morgan rollback strategy (raised by Pessimist):** Morgan is the single point of contact for all customer interactions. A bad deployment breaks all active customer conversations. Implementing agents should define a canary or blue-green deployment strategy for Morgan specifically, with conversation state preservation during rollbacks. This is the highest-risk deployment in the platform.

2. **Repo structure for Go service:** The Rust services have a defined Cargo workspace strategy (D4 refinement). The Go RMS service (Task 3) repo structure needs to be decided — standalone repo or mono-repo with proto definitions shared across all services.

3. **Proto file management (relevant if D3 resolves toward gRPC):** If gRPC is adopted

