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

The initial task decomposition identified **10 tasks** spanning infrastructure, backend services, AI agent, frontend, and production hardening.

### Task Summary

| ID | Title | Agent | Stack | Priority | Dependencies |
|----|-------|-------|-------|----------|--------------|
| 1 | Provision Core Infrastructure | Bolt | Kubernetes/Helm | High | — |
| 2 | Equipment Catalog Service | Rex | Rust/Axum | High | 1 |
| 3 | Rental Management System (RMS) | Grizz | Go/gRPC | High | 1 |
| 4 | Finance Service | Rex | Rust/Axum | High | 1 |
| 5 | Customer Vetting Service | Rex | Rust/Axum | High | 1 |
| 6 | Social Media Engine | Nova | Node.js/Elysia | Medium | 1 |
| 7 | Morgan AI Agent | Angie | OpenClaw/MCP | High | 2, 3, 4, 5, 6 |
| 8 | Web Frontend | Blaze | React/Next.js | High | 2, 7 |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | High | 2–8 |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit | Bolt | Kubernetes/Helm | High | 9 |

### Key Services and Components

- **Infrastructure Layer**: PostgreSQL 16 (CNPG), Valkey 7.2, Cloudflare R2, Signal-CLI, Cloudflare Tunnel
- **Backend Services (4)**: Equipment Catalog (Rust/Axum), RMS (Go), Finance (Rust/Axum), Customer Vetting (Rust/Axum)
- **Support Service (1)**: Social Media Engine (Node.js/Elysia + Effect)
- **AI Agent (1)**: Morgan (OpenClaw with 11 MCP tools)
- **Frontend (1)**: Next.js 15 website with equipment catalog, quote builder, and chat widget
- **Production Hardening (2)**: HA scaling, CDN/TLS/ingress, RBAC, secret rotation, audit logging

### Agent Assignments

| Agent | Responsibilities | Languages |
|-------|-----------------|-----------|
| **Bolt** | Infrastructure, production hardening (Tasks 1, 9, 10) | YAML, Helm, Terraform |
| **Rex** | Equipment Catalog, Finance, Customer Vetting (Tasks 2, 4, 5) | Rust 1.75+ |
| **Grizz** | RMS (Task 3) | Go 1.22+ |
| **Nova** | Social Media Engine (Task 6) | TypeScript/Node.js 20+ |
| **Angie** | Morgan AI Agent (Task 7) | OpenClaw/MCP config |
| **Blaze** | Website (Task 8) | TypeScript/Next.js 15 |

### Cross-Cutting Concerns

- **12 decision points** were identified across tasks, covering platform choices, API design, data modeling, security, service topology, UX behavior, and secret management
- All backend services share PostgreSQL (CNPG) and Valkey as common dependencies
- Authentication/authorization strategy applies across all services
- Secret management approach affects all deployments
- Schema separation strategy impacts all database-connected services

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine should be used for caching, rate limiting, and session storage? {#dp-1}

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2 (Catalog), Task 3 (RMS), Task 4 (Finance), Task 6 (Social)

**Context**: The Valkey operator (`redis.redis.opstreelabs.in`) is already deployed in-cluster. Both debaters agreed unanimously that deploying a second Redis-compatible engine would be operational waste.

**Decision**: Use the existing Valkey operator with Valkey 7.2-alpine as the single cache layer for all services.

**Consensus**: Unanimous (both debaters agreed without contention)

**Consequences**:
- ✅ Single operator = one monitoring surface, one backup strategy, one set of alerts
- ✅ Wire-compatible with all Redis client libraries (`fred`, `go-redis`, `ioredis`) — no application code changes
- ✅ Already deployed; no provisioning effort beyond namespace-level resource creation
- ⚠️ Single cache layer means a Valkey outage affects all services simultaneously (mitigated by HA in Task 9)

---

### [D2] Which object storage provider should be used for product images and social media photos? {#dp-2}

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 2 (Catalog), Task 6 (Social)

**Context**: The entire edge stack (CDN, Pages, Tunnel) is already Cloudflare. Both debaters agreed R2 is the correct choice.

**Decision**: Cloudflare R2 as primary S3-compatible object storage.

**Consensus**: Unanimous (both debaters agreed without contention)

**Consequences**:
- ✅ Zero egress fees — critical for image-heavy workloads (533+ product images, event galleries)
- ✅ Native Cloudflare CDN integration simplifies caching and DNS configuration
- ✅ S3-compatible API — `aws-sdk-s3` / `@aws-sdk/client-s3` clients work without modification
- ✅ Consolidates billing on one edge provider

---

### [D3] Which PostgreSQL operator should be used for managing the main database cluster? {#dp-3}

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Tasks 2–6 (all backend services)

**Context**: CloudNative-PG CRDs are already present on the cluster and explicitly specified in the PRD infrastructure YAML. Both debaters agreed switching operators is churn for zero benefit.

**Decision**: CloudNative-PG (already specified and CRDs present in cluster).

**Consensus**: Unanimous (both debaters agreed without contention)

**Consequences**:
- ✅ Already deployed; PRD-specified
- ✅ Native WAL archiving to S3/R2 for backups
- ✅ Declarative backup policies and seamless minor-version upgrades
- ✅ Clear upgrade path from single-instance to HA replicas (Task 9)

---

### [D4] Should Finance and Customer Vetting be separate services or merged? {#dp-4}

**Status**: Accepted

**Task Context**: Task 4 (Finance), Task 5 (Customer Vetting)

**Context**: The Optimist argued for separate services based on distinct bounded contexts and different failure modes (Stripe vs. external intelligence APIs). The Pessimist argued that at single-digit RPS with a single operator, the operational overhead of two additional deployments (health checks, alerts, connection pools, migration pipelines) isn't justified, and proposed a single binary with clean module boundaries that can be extracted later.

**Decision**: Separate Rust/Axum services with a shared `sigma1-common` crate.

**Consensus**: Not formally voted — the Pessimist's counter-argument was strong but the PRD explicitly describes these as separate services. Implementing agents should follow the PRD's service decomposition.

**Consequences**:
- ✅ Clean bounded context isolation — Finance failure modes (Stripe webhooks) don't affect vetting pipeline
- ✅ Independent deployment cadences — tax calculation changes don't risk vetting pipeline
- ✅ Shared `sigma1-common` crate for auth middleware, error types, telemetry eliminates code duplication
- ⚠️ Two additional deployments, health check configs, and migration pipelines (Pessimist's concern)
- ⚠️ At current scale (single-digit RPS), the operational overhead may exceed the isolation benefit — monitor and consider merging if operational burden is excessive

---

### [D5] What API paradigm should be used for Equipment Catalog and RMS? {#dp-5}

**Status**: Accepted

**Task Context**: Task 2 (Equipment Catalog), Task 3 (RMS)

**Context**: The Optimist proposed REST for Catalog and gRPC with grpc-gateway for RMS per the PRD. The Pessimist argued forcefully that gRPC adds protobuf toolchain complexity (`protoc-gen-grpc-gateway` → `protoc-gen-openapiv2` → `buf` version coupling), introduces a different debugging story (no `curl`), and provides no benefit since every consumer (frontend, Morgan, other services) uses REST. No streaming use cases exist — all RMS operations are request-response CRUD at low RPS. The Pessimist specifically named grpc-gateway version mismatches as a documented source of CI breakage.

**Decision**: REST/JSON for all services. RMS implemented in Go with a standard HTTP router (`chi` or stdlib), OpenAPI spec for typed contract generation.

**Consensus**: The Pessimist's argument prevailed. Every consumer uses REST; gRPC adds complexity without corresponding benefit at this scale.

**Consequences**:
- ✅ Uniform debugging story across all services — `curl`/`httpie` works everywhere
- ✅ No protobuf compilation pipeline or grpc-gateway version coupling in CI
- ✅ OpenAPI codegen provides typed clients comparable to protobuf-generated stubs
- ✅ Reduces cognitive overhead for the team maintaining heterogeneous services
- ⚠️ Deviates from PRD's explicit "gRPC with grpc-gateway" specification for RMS
- ⚠️ If streaming needs emerge later (unlikely per current requirements), gRPC can be added selectively

---

### [D6] How should multi-tenancy and schema separation be handled in PostgreSQL? {#dp-6}

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Tasks 2–6 (all backend services)

**Context**: Both debaters agreed on separate schemas within a single CNPG database. However, the Pessimist raised a critical caveat: the Optimist's mention of cross-schema joins for reporting creates implicit coupling that makes future service extraction impossible. The Pessimist proposed strict cross-schema isolation with reporting via service APIs or a dedicated read replica.

**Decision**: Separate schemas within a single CNPG database, with schema-scoped roles that **cannot** read other schemas. No cross-schema joins from application code. Reporting via service APIs or a dedicated read replica with a reporting-only role.

**Consensus**: Agreement on schema-per-service; the Pessimist's strict isolation caveat was adopted.

**Consequences**:
- ✅ Logical isolation with operational simplicity (one database, one backup, one connection pool config)
- ✅ Schema-scoped roles enforce access boundaries at the database level
- ✅ Preserves the ability to extract services to separate databases without rewriting queries
- ⚠️ Reporting queries may be slower when going through service APIs vs. direct joins — mitigated by a read replica with a reporting-only role if needed
- ⚠️ Implementing agents **must not** create cross-schema foreign keys or joins in application code

---

### [D7] What authentication and authorization mechanism should be used? {#dp-7}

**Status**: Accepted

**Task Context**: Tasks 1–10 (all services and infrastructure)

**Context**: The Optimist proposed Cilium-managed mTLS for service-to-service authentication plus JWT for user-facing endpoints. The Pessimist argued that Cilium mTLS adds a significant debugging surface — certificate rotation failures have cluster-wide blast radius, and at 3am you don't want to be distinguishing cert issues from service issues. The Pessimist proposed JWT for everything (service-to-service and user-facing), with signing key rotation via External Secrets Operator, plus Cilium network policies (without mTLS) for network segmentation.

**Decision**: JWT-based authentication for both service-to-service and user-facing endpoints, with signing key rotation via External Secrets Operator. Cilium network policies for network segmentation **without** mTLS.

**Consensus**: The Pessimist's argument prevailed based on operational simplicity and debuggability.

**Consequences**:
- ✅ Single auth mechanism debuggable with standard HTTP tooling (`curl`, `httpie`) across all languages
- ✅ Signing keys rotate automatically via the already-agreed External Secrets Operator
- ✅ Cilium network policies still enforce pod-level network segmentation
- ✅ Works identically across Rust (Axum extractors), Go (middleware), and Node.js (Elysia plugins)
- ⚠️ JWT for service-to-service means tokens must be validated on every request (mitigated by short validation, ~microseconds)
- ⚠️ No cryptographic service identity at the network layer — compensated by Cilium network policies and JWT claims

---

### [D8] Should Signal integration be self-hosted via Signal-CLI? {#dp-8}

**Status**: Accepted (Hard Constraint)

**Task Context**: Task 1 (Infrastructure), Task 7 (Morgan Agent)

**Context**: Both debaters agreed this is a hard constraint — no managed Signal gateway SaaS exists. The Pessimist raised an important operational concern: Signal-CLI is an unofficial, reverse-engineered client that depends on Signal's undocumented protocol. Signal has historically broken third-party clients without notice.

**Decision**: Self-hosted Signal-CLI as a separate pod.

**Consensus**: Unanimous (hard constraint)

**Consequences**:
- ✅ Only viable option for Signal integration
- ⚠️ **Critical risk**: Signal-CLI is unofficial and can break without notice when Signal updates their protocol. Task 7 must include a fallback communication path (see D9 on graceful degradation)
- ⚠️ Implementing agents should design Morgan's Signal integration with a clear abstraction layer to facilitate swapping to alternative Signal libraries if `signal-cli` breaks

---

### [D9] How should Morgan orchestrate backend service calls? {#dp-9}

**Status**: Accepted

**Task Context**: Task 7 (Morgan Agent)

**Context**: Both debaters agreed on the MCP tool-server abstraction. The Pessimist's critical addition was mandatory graceful degradation: when the tool-server is down, Morgan must detect the failure and fall back to human handoff (connecting the user to Mike) within the 10-second response SLA, rather than hanging or erroring silently.

**Decision**: MCP tool-server abstraction mediating all backend interactions, with mandatory graceful degradation — Morgan must detect tool-server failures and fall back to human handoff within the 10-second SLA.

**Consensus**: Unanimous on MCP; Pessimist's degradation requirement was adopted.

**Consequences**:
- ✅ Per-tool rate limiting, auditing, versioning, and testing without touching agent logic
- ✅ Composable and testable skill architecture aligned with PRD's 11 MCP tools
- ✅ Graceful degradation ensures customer experience is preserved even during backend outages
- ⚠️ Tool-server is a single point of failure for AI operations — health checks and circuit breakers are required
- ⚠️ Implementing agents must implement a health-check loop and a "connect you with Mike directly" fallback message

---

### [D10] What component library and design system for the web frontend? {#dp-10}

**Status**: Accepted

**Task Context**: Task 8 (Web Frontend)

**Context**: PRD-specified. Both debaters agreed without contention.

**Decision**: shadcn/ui with TailwindCSS 4, built on Radix UI primitives. Effect Schema integration for type-safe forms.

**Consensus**: Unanimous (PRD-specified)

**Consequences**:
- ✅ Owned, accessible components (Radix-based WCAG 2.1)
- ✅ TailwindCSS 4 native CSS layers for styling
- ✅ Full design flexibility for catalog and portfolio pages
- ✅ Effect Schema for consistent form validation across the frontend

---

### [D12] What approach for secret management and rotation? {#dp-12}

**Status**: Accepted

**Task Context**: Task 1 (Infrastructure), Task 9 (Production Hardening), Task 10 (RBAC/Secrets/Audit)

**Context**: `external-secrets.io` CRDs are already deployed in-cluster. Both debaters agreed that automated rotation is essential for GDPR compliance.

**Decision**: External Secrets Operator (`external-secrets.io`) with automated rotation via `ExternalSecret` resources pointing to a configured secret store (1Password, Vault, or AWS SSM).

**Consensus**: Unanimous (CRDs already deployed, GDPR requirement)

**Consequences**:
- ✅ Automated rotation eliminates human error and drift
- ✅ Auditable secret access satisfies GDPR compliance requirement
- ✅ Already deployed in-cluster — no new operator installation
- ⚠️ Requires a backing secret store to be configured (1Password, Vault, or AWS SSM) — Task 1 must include this configuration

---

## 4. Escalated Decisions

### [D11] How should Morgan's web chat be integrated into the website? — ESCALATED {#dp-11}

**Status**: Pending human decision

**Task Context**: Task 8 (Web Frontend)

**Options**:
- **Option A (Optimist)**: Floating chat widget on all pages AND a dedicated `/chat` route for full-page experience
- **Option B (Pessimist)**: Floating chat widget only for v1; dedicated `/chat` route deferred to post-launch based on user feedback

**Optimist argued**: The widget drives engagement for quick queries, while the full-page `/chat` route supports Morgan's richer workflows (quote building, vetting status). Both are low implementation cost with shadcn/ui components. The PRD says "embedded chat widget" but the platform's value proposition is AI-first — a dedicated route reinforces that.

**Pessimist argued**: Building both doubles the chat integration surface in Task 8 — state management, routing, responsive layouts, testing. The project already has 10 tasks across 6 languages. The widget satisfies the PRD's requirement. Ship simpler, measure demand, then invest.

**Recommendation**: Start with the floating chat widget (satisfying the PRD requirement) but **architect the chat component as a standalone React component** that can be rendered both inline and in a full-page layout. This costs minimal additional effort during initial development and makes the `/chat` route a trivial follow-up if demand warrants it. The implementing agent (Blaze) should extract the chat UI into a `<MorganChat />` component with a `mode: 'widget' | 'fullpage'` prop, but only render the widget mode for v1.

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| **Database** | PostgreSQL | 16 | CloudNative-PG operator, single cluster |
| **Cache** | Valkey | 7.2-alpine | OpsTree operator, Redis wire-compatible |
| **Object Storage** | Cloudflare R2 | — | S3-compatible, zero egress |
| **CDN/Edge** | Cloudflare | — | Pages, Tunnel, CDN, SSL |
| **Backend (Catalog)** | Rust/Axum | 1.75+ / 0.7 | REST/JSON |
| **Backend (RMS)** | Go | 1.22+ | REST/JSON (chi or stdlib), **not gRPC** |
| **Backend (Finance)** | Rust/Axum | 1.75+ / 0.7 | REST/JSON, Stripe integration |
| **Backend (Vetting)** | Rust/Axum | 1.75+ / 0.7 | REST/JSON, external API aggregation |
| **Backend (Social)** | Node.js/Elysia + Effect | 20+ / 1.x / 3.x | REST/JSON, multi-platform publishing |
| **AI Agent** | OpenClaw | latest | MCP tool-server, Signal-CLI, ElevenLabs |
| **Frontend** | Next.js 15 / React 19 | 15 / 19 | shadcn/ui, TailwindCSS 4, Effect 3.x |
| **Auth** | JWT | — | All endpoints, signing key rotation via ESO |
| **Secrets** | External Secrets Operator | — | Automated rotation |
| **Observability** | Grafana + Loki + Prometheus | — | Existing OpenClaw stack |

### Service Architecture

```
                    ┌─────────────┐
                    │  Cloudflare  │
                    │  (CDN/SSL/   │
                    │   Tunnel)    │
                    └──────┬──────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
    │  Next.js 15 │ │   Morgan    │ │  Signal-CLI │
    │  (Blaze)    │ │  (OpenClaw) │ │  (sidecar)  │
    │  Cloudflare │ │  MCP Tools  │ └──────┬──────┘
    │  Pages      │ └──────┬──────┘        │
    └──────┬──────┘        │        ┌──────▼──────┐
           │               │        │  ElevenLabs │
           │        ┌──────▼──────┐ │  Twilio     │
           │        │ Tool Server │ └─────────────┘
           │        └──────┬──────┘
           │               │
    ┌──────┴───────────────┼──────────────────────┐
    │                      │                       │
┌───▼──────┐  ┌───────────▼──────┐  ┌────────────▼───┐
│Equipment │  │      RMS         │  │    Finance     │
│Catalog   │  │   (Go/REST)      │  │  (Rust/Axum)   │
│(Rust/Axum)│ │   chi router     │  └────────────────┘
└───┬──────┘  └───────┬──────────┘           │
    │                  │              ┌──────▼──────┐
    │                  │              │  Customer    │
    │                  │              │  Vetting     │
    │                  │              │ (Rust/Axum)  │
    │                  │              └──────┬──────┘
    │                  │                     │
┌───▼──────────────────▼─────────────────────▼───┐
│           PostgreSQL 16 (CNPG)                  │
│  ┌────────┐ ┌─────┐ ┌────────┐ ┌───────┐      │
│  │catalog │ │ rms │ │finance │ │vetting│ ...  │
│  │ schema │ │schema│ │ schema │ │schema │      │
│  └────────┘ └─────┘ └────────┘ └───────┘      │
└────────────────────────────────────────────────┘
          ┌──────────┐  ┌──────────┐
          │  Valkey   │  │Cloudflare│
          │  7.2      │  │  R2      │
          └──────────┘  └──────────┘
```

### Communication Patterns

- **Client → Services**: REST/JSON over HTTPS (Cloudflare Tunnel/CDN termination)
- **Service → Service**: REST/JSON with JWT bearer tokens, Cilium network policies enforcing pod-level segmentation
- **Morgan → Backend**: MCP tool-server abstraction → REST calls to individual services
- **Morgan → Customer**: Signal-CLI (messages), ElevenLabs (voice), web chat widget (WebSocket/SSE)
- **All services → PostgreSQL**: Direct connection via schema-scoped roles; **no cross-schema access**
- **All services → Valkey**: Standard Redis protocol for caching, rate limiting, sessions

### What Was Explicitly Ruled Out

| Ruled Out | Reason |
|-----------|--------|
| **gRPC for RMS** | All consumers use REST; protobuf toolchain adds CI complexity without benefit at this scale (D5) |
| **Cilium mTLS** | Certificate rotation failures have cluster-wide blast radius; JWT is more debuggable (D7) |
| **Cross-schema database joins** | Creates implicit coupling preventing future service extraction (D6) |
| **Separate Redis instance** | Valkey operator already deployed; second cache layer is operational waste (D1) |
| **Alternative PG operators** | CNPG already deployed and PRD-specified; switching is churn (D3) |
| **Manual secret management** | GDPR non-compliant; External Secrets Operator already deployed (D12) |
| **Trading Desk (Phase 1)** | Python not in core stack; deferred to Phase 2 per PRD |

---

## 6. Implementation Constraints

### Security Requirements

- **JWT everywhere**: All service-to-service and user-to-service communication must use JWT with RBAC claims. Signing keys must be rotated automatically via External Secrets Operator.
- **Schema-scoped database roles**: Each service connects to PostgreSQL with a role that can **only** access its own schema. No cross-schema reads or writes.
- **Cilium network policies**: Must be applied to restrict inter-pod traffic. Only explicitly allowed service-to-service paths should be permitted.
- **GDPR compliance**: All services must support data export and customer deletion. Audit logging of all access and API events is mandatory (Task 10).
- **Security scanning**: Critical/high severity issues block merge (Cipher agent in CI pipeline).

### Performance Targets

| Metric | Target | Service |
|--------|--------|---------|
| Morgan response time (simple queries) | < 10 seconds | Morgan Agent |
| Equipment availability check | < 500ms | Equipment Catalog |
| Invoice generation | < 5 seconds | Finance Service |
| Concurrent Signal connections | 500+ | Morgan Agent / Signal-CLI |
| Uptime (production) | 99.9% | All services |

### Operational Requirements

- **Observability**: All services must expose Prometheus metrics at `/metrics` and health probes at `/health/live` and `/health/ready`. Integrate with existing Grafana + Loki + Prometheus stack.
- **GitOps**: All deployments via ArgoCD with automatic rollbacks on failure.
- **Code coverage**: Minimum 80% required (enforced by Tess agent in CI).
- **HA scaling**: PostgreSQL and Valkey must be scaled to multi-replica mode for production (Task 9).

### Service Dependencies and Integration Points

| Service | External Dependencies |
|---------|----------------------|
| Equipment Catalog | PostgreSQL, Valkey, Cloudflare R2 |
| RMS | PostgreSQL, Valkey, Google Calendar API |
| Finance | PostgreSQL, Valkey, Stripe API |
| Customer Vetting | PostgreSQL, OpenCorporates API, LinkedIn API, Google Reviews, Credit APIs |
| Social Media Engine | PostgreSQL, Cloudflare R2, Instagram Graph API, LinkedIn API, Facebook Graph API, OpenAI/Claude |
| Morgan Agent | All backend services (via MCP tool-server), Signal-CLI, ElevenLabs, Twilio |
| Website | Equipment Catalog API, Morgan Agent (chat), Cloudflare Pages |

### Organizational Preferences

- **Prefer existing in-cluster operators** over deploying new ones (Valkey, CNPG, External Secrets all already deployed)
- **Cloudflare-first for edge services**: R2, Pages, Tunnel, CDN — consolidate on one edge provider
- **Shared Rust crate** (`sigma1-common`): Auth middleware, error types, and telemetry shared across all Rust services (Catalog, Finance, Vetting)
- **REST everywhere**: Uniform API paradigm simplifies debugging, client generation, and team cognitive load

---

## 7. Design Intake Summary

### Frontend Detection

- **`hasFrontend`**: `true`
- **`frontendTargets`**: `web`, `mobile`
- **Mode**: `ingest_plus_stitch` (design ingest with Stitch provider attempted)

### Supplied Design Artifacts and References

- **Website reference**: https://sigma-1.com (existing site for Perception Events)
- **Existing platform reference**: https://deployiq.maximinimal.ca (current tooling being replaced)
- No additional design mockups, Figma files, or brand guidelines were supplied in the design context.

### Provider Generation Status

- **Stitch**: `failed` — Stitch design generation was attempted but failed (no reason provided). No Stitch-generated design candidates are available.
- **Framer**: `skipped` — Framer generation was not requested.

### Component Library / Design System

- **PRD-specified**: shadcn/ui + TailwindCSS 4 + Radix UI primitives (confirmed by D10)
- **Effect Schema** for type-safe form validation
- **TanStack Query + Effect** for data fetching
- No pre-built component-library artifacts (token files, theme configs) were supplied — implementing agents will initialize shadcn/ui from defaults and customize per brand.

### Implications for Implementation

1. **Web (Task 8)**: Blaze should initialize the Next.js 15 project with `shadcn/ui init`, configure TailwindCSS 4, and establish the design system tokens (colors, typography, spacing) based on the existing sigma-1.com branding. Since no Stitch/Framer designs were generated, the implementing agent should reference https://sigma-1.com for visual direction.

2. **Mobile (Expo)**: The PRD architecture

