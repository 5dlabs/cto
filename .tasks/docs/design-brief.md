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

The initial task decomposition identified **10 tasks** spanning infrastructure, backend services, the AI agent, the website, and production hardening.

| Task ID | Title | Agent | Stack | Dependencies | Priority |
|---------|-------|-------|-------|-------------|----------|
| 1 | Bootstrap Core Infrastructure | Bolt | Kubernetes/Helm | — | High |
| 2 | Equipment Catalog Service API | Rex | Rust/Axum | 1 | High |
| 3 | RMS Service | Grizz | Go/gRPC | 1 | High |
| 4 | Finance Service | Rex | Rust/Axum | 1 | High |
| 5 | Customer Vetting Service | Rex | Rust/Axum | 1 | High |
| 6 | Social Media Engine | Nova | Node.js/Elysia + Effect | 1 | Medium |
| 7 | Morgan AI Agent | Angie | OpenClaw/MCP | 1, 2, 3, 4, 5, 6 | High |
| 8 | Sigma-1 Website | Blaze | Next.js/React/Effect | 1, 2, 6, 7 | High |
| 9 | Production Hardening: HA, CDN, TLS, Ingress | Bolt | Kubernetes/Helm | 2–8 | High |
| 10 | Production Hardening: RBAC, Secret Rotation, Audit | Bolt | Kubernetes/Helm | 9 | High |

### Key Services & Components

- **Infrastructure layer**: CloudNative-PG PostgreSQL 16, Valkey (Redis-compatible), Cloudflare R2 object storage, Cloudflare Tunnel ingress, Signal-CLI, External Secrets Operator
- **Backend services**: Equipment Catalog (Rust/Axum), RMS (Go/gRPC), Finance (Rust/Axum), Customer Vetting (Rust/Axum), Social Engine (Node.js/Elysia)
- **AI agent**: Morgan (OpenClaw with 11 MCP tools)
- **Frontend**: Next.js 15 website with React 19, shadcn/ui, TailwindCSS 4, Effect 3.x
- **Observability**: Grafana + Loki + Prometheus (existing OpenClaw stack)
- **QA pipeline**: Stitch (code review), Cleo (quality), Tess (testing), Cipher (security), Atlas (merge gate), Bolt (deployment)

### Cross-Cutting Concerns

- 14 decision points identified across tasks covering platform choices, API design, security, data model, service topology, build-vs-buy, UX behavior, design system, and component library
- GDPR compliance required across all data-bearing services
- Minimum 80% test coverage enforcement
- GitOps deployment via ArgoCD with automatic rollbacks

---

## 3. Resolved Decisions

### [D1] Which Redis-compatible engine should be used for caching, rate limiting, and session storage?

**Status**: Accepted  
**Task Context**: Tasks 1, 2, 3, 4 (infrastructure + all services needing cache)  
**Context**: The existing Valkey operator (`redis.redis.opstreelabs.in`) is already deployed in the cluster. Both debaters agreed unanimously that reusing it avoids adding a second operational surface.  
**Decision**: Use the existing Valkey operator, provisioning a dedicated Sigma-1 instance via the CRD (`valkey/valkey:7.2-alpine`).  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Zero new operator deployment; wire-compatible with all Redis client libraries (redis-rs, go-redis, ioredis); per-instance isolation via CRD
- **Negative**: None identified
- **Caveats**: None

---

### [D2] Which object storage provider should be used for product images and event photos?

**Status**: Accepted  
**Task Context**: Tasks 1, 2, 6, 8 (infrastructure, catalog images, social engine photos, website assets)  
**Context**: Cloudflare is already integrated in the cluster (operator, tunnels, CDN). R2 provides zero-egress-cost reads through the existing CDN.  
**Decision**: Cloudflare R2 with S3-compatible API.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Zero cross-cloud egress; S3-compatible SDKs (aws-sdk-s3 in Rust, Go, Node) work unchanged; single-hop path from R2 → Cloudflare CDN for website (Task 8)
- **Negative**: Vendor lock-in to Cloudflare (mitigated by S3-compatible API)
- **Caveats**: None

---

### [D4] How should multi-tenancy and schema separation be handled in PostgreSQL?

**Status**: Accepted  
**Task Context**: Tasks 1, 2, 3, 4, 5, 6 (all data-bearing services)  
**Context**: The PRD explicitly specifies this approach. Cross-service queries (e.g., opportunity-to-invoice joins for profitability reports) become trivial with schemas vs. federated queries.  
**Decision**: Single CloudNative-PG cluster with multiple schemas (`rms`, `crm`, `finance`, `audit`, `public`) in one database.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Simplified backup/failover (one CNPG cluster); cross-schema joins for reporting; reduced connection pooler configuration
- **Negative**: Blast radius — a database failure affects all services simultaneously
- **Caveats**: Per-service database roles with `SEARCH_PATH` isolation provide security boundaries; schema-level RBAC in Task 10

---

### [D8] How should Google Reviews and credit data be accessed for customer vetting?

**Status**: Accepted  
**Task Context**: Task 5 (Customer Vetting Service)  
**Context**: Both debaters agreed emphatically. Scraping violates Google ToS, breaks monthly, and poisons lead scoring when it fails silently.  
**Decision**: Commercial APIs for both — Google Places API for reviews, a credit data provider (Creditsafe or Dun & Bradstreet) for credit signals.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Reliable structured data; legally compliant; Google Places API costs ~$2/month at 50–100 leads/month
- **Negative**: Recurring API costs (negligible at projected volume)
- **Caveats**: None

---

### [D9] What versioning strategy should be used for APIs?

**Status**: Accepted  
**Task Context**: Tasks 2, 3, 4, 5, 6, 8 (all services with APIs)  
**Context**: The PRD already specifies `/api/v1/` paths throughout every service definition. Both debaters agreed.  
**Decision**: URI-based versioning (`/api/v1/...`) for all APIs, both public and internal.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Explicit in logs, metrics, and curl commands; consistent across all services; simplifies observability (Task 9)
- **Negative**: URI proliferation when v2 arrives (manageable)
- **Caveats**: None

---

### [D10] What approach for GDPR data export and deletion?

**Status**: Accepted  
**Task Context**: Tasks 2, 3, 4, 5, 6, 8, 10 (all data-bearing services + production hardening)  
**Context**: Both debaters agreed this is the standard pattern recommended by the UK ICO and used by production GDPR-compliant platforms.  
**Decision**: Soft-delete with 30-day retention window, then automated hard deletion via scheduled job. Immediate hard-delete available on explicit request.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Audit trail and recovery window for accidental deletions; satisfies "right to be forgotten"; scheduled purge is automatable
- **Negative**: 30-day window retains data beyond deletion request (mitigated by immediate hard-delete path for explicit requests)
- **Caveats**: The `audit` schema logs all deletion events

---

### [D11] What interaction pattern for Morgan web chat on the website?

**Status**: Accepted  
**Task Context**: Task 8 (Website)  
**Context**: Both debaters agreed. The PRD's success criterion #1 is "Morgan handles 80%+ of customer inquiries autonomously" — maximizing surface area is critical.  
**Decision**: Persistent floating chat widget accessible from all pages, with expand-to-fullscreen capability.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Zero-friction access from any page; dominant pattern for conversational AI (Intercom, Drift, ChatGPT); handles complex interactions via fullscreen expansion
- **Negative**: May obscure content on small viewports (mitigated by minimize/collapse behavior)
- **Caveats**: None

---

### [D12] Should the frontend use shadcn/ui as-is or extend it?

**Status**: Accepted  
**Task Context**: Task 8 (Website)  
**Context**: Both debaters agreed. tweakcn is already deployed in the `cto` namespace, suggesting the team already customizes shadcn/ui via theming.  
**Decision**: Use shadcn/ui with TailwindCSS 4 theme customization (CSS variables for Sigma-1 brand colors, typography, spacing) — no fork.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Velocity — no fork maintenance; brand consistency via theme layer; upstream improvements flow in naturally
- **Negative**: Limited to what CSS variables and Tailwind config can achieve (sufficient for branding)
- **Caveats**: None

---

### [D13] Which data table library for equipment catalog and finance reporting?

**Status**: Accepted  
**Task Context**: Task 8 (Website)  
**Context**: Both debaters agreed. shadcn/ui's own documentation recommends this exact pattern.  
**Decision**: TanStack Table for headless logic (sorting, filtering, pagination, column visibility) with shadcn/ui's `<Table>` component as the rendering layer.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: Server-side pagination for 533+ products; Effect Schema integration for type-safe column definitions; follows shadcn/ui's own recommended pattern
- **Negative**: None identified
- **Caveats**: None

---

### [D14] Which CDN and TLS termination solution for public-facing services?

**Status**: Accepted  
**Task Context**: Tasks 1, 9 (infrastructure, production hardening)  
**Context**: Cloudflare operator is deployed with Tunnel CRDs. Both debaters agreed that running a parallel NGINX ingress is pointless complexity.  
**Decision**: Cloudflare CDN and Tunnel for ingress and TLS termination.  
**Consensus**: 2/2 (100%)  
**Consequences**:
- **Positive**: No public port exposure; managed TLS (no cert-manager, no Let's Encrypt rate limits); global anycast CDN with DDoS protection; aligns with PRD's Cloudflare Pages for website
- **Negative**: Dependency on Cloudflare availability (mitigated by their 99.99% SLA)
- **Caveats**: None

---

### [D6] Should Morgan interact with backend services directly or through MCP tool-server abstraction?

**Status**: Accepted  
**Task Context**: Task 7 (Morgan AI Agent)  
**Context**: The PRD defines 11 specific MCP tools. Both debaters agreed on MCP-only access, but the Pessimist raised critical operational concerns about the tool-server becoming a single point of failure.  
**Decision**: Morgan uses MCP tool-server exclusively for all backend interactions. **The tool-server must be treated as Tier-0 infrastructure** with HA deployment, per-tool circuit breakers, per-tool timeout enforcement (5s per-tool cutoff to stay within the 10s response constraint), and health checks.  
**Consensus**: 2/2 (100% — Pessimist's operational conditions were adopted into the decision)  
**Consequences**:
- **Positive**: Centralized authorization and audit logging; decoupled from service implementation details; natural upgrade path when backend APIs change
- **Negative**: Single point of failure if not hardened — all Morgan capabilities fail simultaneously
- **Caveats from Pessimist (adopted)**: Tool-server must have graceful degradation (Morgan says "I can't check availability right now" rather than going silent); aggressive per-tool timeouts; HA deployment required before production

---

## 4. Escalated Decisions

### [D3] What API paradigm should be used for inter-service communication? — ESCALATED

**Status**: Pending human decision  
**Task Context**: Tasks 2, 3, 4, 5, 6, 7, 8 (all backend services + website)

**Options**:
- **Option A (Optimist)**: gRPC for internal service-to-service calls, REST (via grpc-gateway) for public/website access
- **Option B (Pessimist)**: REST (HTTP/JSON) for all service APIs, with gRPC only for the RMS service which already specifies it

**Optimist argued**: The RMS (Task 3) is already specified as gRPC with grpc-gateway. Extending this pattern to internal calls gives type-safe protobuf contracts, efficient binary serialization, and streaming capability. Rust has mature gRPC via tonic. MCP tool-server wraps gRPC calls for Morgan. This is a proven pattern at Discord, Cloudflare.

**Pessimist argued**: Who actually calls these services internally? Only Morgan (via MCP tool-server, which speaks HTTP) and the website (Next.js has zero native gRPC support). Every Rust service would need both tonic AND Axum, plus protobuf compilation in CI, plus grpc-gateway configuration — double the API surface for zero actual gRPC consumers. Let each service use its natural paradigm: RMS is gRPC (PRD-specified, Go's gRPC is first-class), Rust/Axum services are REST, Node.js/Elysia is REST. Type-safe contracts are achievable via OpenAPI specs validated at CI time.

**Recommendation**: Option B is the pragmatic choice for Phase 1. The actual consumers (MCP tool-server and Next.js website) both speak HTTP/JSON natively. The RMS already has gRPC with grpc-gateway providing REST access, so it's accessible to both consumers regardless. Adding gRPC to Rust/Axum and Node.js/Elysia services creates dual-stack complexity with no current consumer of the gRPC interface. If gRPC is needed in Phase 2, individual services can be migrated incrementally — the MCP tool-server abstraction (D6) insulates Morgan from backend protocol changes.

---

### [D5] What authentication and authorization mechanism should be used? — ESCALATED

**Status**: Pending human decision  
**Task Context**: Tasks 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 (platform-wide)

**Options**:
- **Option A (Optimist)**: mTLS between services for internal calls (leveraging Cilium's built-in identity), JWT for user-facing APIs and Morgan's external integrations
- **Option B (Pessimist)**: JWT-based service tokens for internal calls (signed by a shared secret from External Secrets Operator), JWT for user-facing APIs, Cilium network policies for coarse-grained access control only

**Optimist argued**: Cilium is already deployed (CRDs present). Transparent mTLS between pods with identity-based policies means zero code changes in services — service-to-service auth "for free." Combined with JWT for user-facing APIs and External Secrets for key management, this is a layered approach without the complexity of a full OAuth2 server.

**Pessimist argued**: Cilium CRDs being present means Cilium is the CNI, not that mTLS is active. mTLS requires explicit enablement (`encryption.enabled=true`, `encryption.type=wireguard` or `ipsec`). If we design assuming Cilium mTLS and it's not enabled, we have **zero service-to-service authentication** in production. For a platform handling financial data (Task 4) and customer PII (Task 5), "we think the CNI handles it" is not an acceptable security posture. JWT service tokens via External Secrets are explicit, verifiable, and work regardless of CNI configuration. Cilium network policies provide defense-in-depth but are authorization, not authentication.

**Recommendation**: Option B is safer unless the team can verify Cilium encryption is enabled today. The Pessimist's point about misconfiguration risk is the #1 concern in Cilium deployments per Cilium's own documentation. JWT service tokens via External Secrets are explicitly verifiable and testable. If Cilium mTLS is confirmed as active, it can be layered on as defense-in-depth without removing the JWT-based authentication. The Pessimist's framing — Cilium network policies for coarse-grained authorization, JWT for authentication — is the more robust default.

---

### [D7] Should Finance and Customer Vetting be separate services or modules in a single binary? — ESCALATED

**Status**: Pending human decision  
**Task Context**: Tasks 4, 5 (Finance Service, Customer Vetting Service)

**Options**:
- **Option A (Optimist)**: Separate microservices with a shared Rust workspace (cargo workspace with shared crates)
- **Option B (Pessimist)**: Single Rex binary with modular Axum routing, deployed as one service. Equipment Catalog stays separate (different scaling characteristics).

**Optimist argued**: Finance (Stripe webhooks, payment processing) and Vetting (external API calls, background checks) have fundamentally different failure domains. A Vetting crash should never take down invoicing. Cargo workspace gives code sharing without deployment coupling. Three small Axum binaries vs. one monolith — operational overhead is marginal in Kubernetes.

**Pessimist argued**: Both services are stateless Axum handlers backed by the same PostgreSQL instance (D4 — single database). Realistic crash scenarios (bad query, OOM, panic) either affect both or are caught by Axum's per-request panic handling. At 50–100 vettings/month and ~200 invoices/month, there's no scaling scenario justifying independent deployment. One binary = one deployment, one health check, one set of Kubernetes manifests = fewer things to go wrong at 2am. Equipment Catalog stays separate because it's genuinely high-read and CDN-cached.

**Recommendation**: Option B is pragmatic for Phase 1 given the low traffic volumes. The cargo workspace approach (shared crates) should be used regardless — modular code doesn't require separate deployments. A single Rex binary with separate Axum routers for `/api/v1/invoices/*`, `/api/v1/vetting/*`, etc. keeps code boundaries clean while reducing operational surface. If traffic patterns diverge significantly in Phase 2, splitting the binary along existing module boundaries is straightforward.

---

## 5. Architecture Overview

### Agreed Technology Stack

| Layer | Technology | Version | Decision |
|-------|-----------|---------|----------|
| Database | PostgreSQL (CloudNative-PG) | 16 | Single cluster, multi-schema (D4) |
| Cache | Valkey (Redis-compatible) | 7.2 | Existing operator (D1) |
| Object Storage | Cloudflare R2 | — | S3-compatible API (D2) |
| CDN/Ingress | Cloudflare CDN + Tunnel | — | Managed TLS, no public ports (D14) |
| Secret Management | External Secrets Operator | Existing | API keys, JWT signing keys |
| Observability | Grafana + Loki + Prometheus | Existing | OpenClaw stack |

### Service Architecture

| Service | Language/Framework | API Protocol | Deployment |
|---------|-------------------|-------------|-----------|
| Equipment Catalog | Rust 1.75+ / Axum 0.7 | REST | Separate (high-read, CDN-cached) |
| RMS | Go 1.22+ / gRPC + grpc-gateway | gRPC (internal) + REST (gateway) | Separate |
| Finance | Rust 1.75+ / Axum 0.7 | REST (pending D3) | Pending D7 |
| Customer Vetting | Rust 1.75+ / Axum 0.7 | REST (pending D3) | Pending D7 |
| Social Engine | Node.js 20+ / Elysia 1.x + Effect | REST (pending D3) | Separate |
| Morgan Agent | OpenClaw | MCP tool-server (D6) | Separate (Tier-0) |
| Website | Next.js 15 / React 19 / Effect 3.x | Consumes REST APIs | Cloudflare Pages |

### Communication Patterns

- **Morgan → Backend Services**: Exclusively via MCP tool-server (D6). Tool-server is Tier-0 with HA, circuit breakers, and per-tool timeouts
- **Website → Backend Services**: Direct REST API calls (Equipment Catalog, Social Engine for portfolio)
- **Inter-service communication**: Protocol pending D3 resolution; RMS is gRPC with REST gateway regardless
- **Service authentication**: Pending D5 resolution; JWT for user-facing APIs is agreed by both sides
- **External integrations**: Commercial APIs for vetting data (D8); Stripe for payments; social media platform APIs

### Key Patterns

- **API versioning**: URI-based `/api/v1/...` everywhere (D9)
- **Data deletion**: Soft-delete → 30-day retention → scheduled hard-delete; immediate hard-delete on explicit request (D10)
- **Design system**: shadcn/ui with TailwindCSS 4 theme customization, no fork (D12)
- **Data tables**: TanStack Table + shadcn/ui Table rendering (D13)
- **Chat widget**: Persistent floating widget on all pages with fullscreen expansion (D11)

### Explicitly Ruled Out

- **Bitnami Redis Helm chart** — Redundant with existing Valkey operator (D1)
- **AWS S3 as primary storage** — Cloudflare R2 avoids egress costs and aligns with existing CDN (D2)
- **Separate CNPG clusters per service** — Multiplies operational overhead without meaningful isolation gain (D4)
- **Custom scraping for Google Reviews** — Violates ToS, brittle, fails silently (D8)
- **Header-based API versioning** — Adds cognitive overhead with no benefit over URI-based (D9)
- **NGINX Ingress with cert-manager** — Redundant with Cloudflare Tunnel already deployed (D14)
- **shadcn/ui fork** — Maintenance burden; theming via CSS variables is sufficient (D12)
- **Direct Morgan → backend API calls** — Bypasses MCP abstraction, loses audit trail (D6)

---

## 6. Implementation Constraints

### Security Requirements

- All services must support JWT-based authentication for user-facing APIs (agreed by both sides in D5)
- Service-to-service authentication mechanism pending D5 resolution — **implementing agents must not assume Cilium mTLS is active without verification**
- GDPR compliance: soft-delete with audit logging mandatory on all data-bearing services (D10)
- All deletion events must be logged in the `audit` schema
- Critical/high severity security vulnerabilities block merge (Cipher agent)
- Secrets managed via External Secrets Operator — no hardcoded credentials

### Performance Targets

- Morgan response time: < 10 seconds for simple queries (per-tool timeout: 5 seconds in tool-server)
- Equipment availability check: < 500ms
- Invoice generation: < 5 seconds
- Equipment catalog: support for 533+ products with server-side pagination
- Website: LCP < 2 seconds

### Operational Requirements

- 99.9% uptime for production services
- 500+ concurrent Signal connections
- MCP tool-server: Tier-0 with HA deployment, per-tool circuit breakers, graceful degradation
- GitOps deployment via ArgoCD with automatic rollbacks on failure
- Minimum 80% code coverage enforced by CI
- All services must expose `/health/live` (liveness), `/health/ready` (readiness), and `/metrics` (Prometheus)

### Service Dependencies & Integration Points

- **External APIs (commercial, D8)**: Google Places API, Creditsafe/D&B, OpenCorporates, LinkedIn API
- **Payment processing**: Stripe API (Finance Service)
- **Social platforms**: Instagram Graph API, LinkedIn API, Facebook Graph API, TikTok API
- **Communication**: Signal-CLI, ElevenLabs (voice), Twilio (phone/SIP/PSTN)
- **Calendar**: Google Calendar API (RMS crew scheduling)
- **AI**: OpenAI/Claude (caption generation in Social Engine), GPT model for Morgan agent

### Organizational Preferences

- Prefer reusing existing in-cluster infrastructure (Valkey operator, Cloudflare operator, Cilium, External Secrets, CloudNative-PG) over deploying new operators
- Self-hosted/in-cluster solutions preferred for infrastructure; managed APIs accepted for external integrations
- Cargo workspace pattern for shared Rust crates regardless of deployment topology (D7)

---

## 7. Design Intake Summary

### Frontend Targets

- **`hasFrontend`**: true
- **`frontendTargets`**: web, mobile
- **Primary web deliverable**: Next.js 15 website (Task 8) — equipment catalog, quote builder, portfolio, Morgan chat widget
- **Mobile**: Expo (referenced in architecture diagram, not in Phase 1 task decomposition — likely Phase 2)

### Supplied Design Artifacts & References

- **Existing platform**: https://deployiq.maximinimal.ca (current platform being replaced)
- **Target domain**: https://sigma-1.com
- No explicit Figma files, design tokens, or brand guidelines were supplied in the design context

### Provider Generation Status

- **Stitch (design generation)**: Failed — no generated design artifacts available
- **Framer**: Skipped (not requested)
- No normalized design candidates are available from automated providers

### Implications for Implementation

1. **Website (Task 8)** must establish the visual identity from scratch using shadcn/ui + TailwindCSS 4 theming (D12). The implementing agent (Blaze) should reference the existing platform at https://deployiq.maximinimal.ca for functional patterns while establishing a fresh brand aesthetic for Sigma-1.

2. **Design system tokens** (colors, typography, spacing) must be defined as CSS variables and Tailwind theme configuration early in Task 8 to ensure consistency across all pages.

3. **Data-heavy views** (equipment catalog with 533+ products, finance reports) use TanStack Table + shadcn/ui Table (D13), requiring attention to responsive design for the data table layout on mobile viewports.

4. **Morgan chat widget** (D11) — persistent floating widget — needs z-index management, mobile-responsive sizing, and accessibility (keyboard navigation, screen reader support) across all pages.

5. Since no design artifacts were generated, the implementing agent should prioritize a clean, professional aesthetic appropriate for a B2B lighting/visual production company, with high-quality product photography presentation as a key differentiator.

---

## 8. Open Questions

The following items were not debated or resolved and should be handled by implementing agents using best judgment:

1. **Mobile (Expo) scope**: The architecture diagram references Expo for mobile but no task was decomposed for it. Implementing agents should treat mobile as Phase 2 unless directed otherwise, but ensure APIs are mobile-friendly (proper CORS, responsive data payloads).

2. **Morgan agent model version**: The PRD specifies `openai-api/gpt-5.4-pro`. If this model identifier is not available at implementation time, agents should use the closest equivalent high-capability model and document the substitution.

3. **Currency rate sync frequency**: The Finance Service (Task 4) requires scheduled currency rate sync. Agents should default to daily at midnight UTC and document the cron schedule, allowing runtime configuration.

4. **Social platform API rate limits**: The Social Engine (Task 6) publishes to Instagram, TikTok, LinkedIn, and Facebook. Agents should implement per-platform rate limiting with exponential backoff (Effect.retry is specified) and document known rate limit thresholds.

5. **Equipment Catalog data seeding**: 533+ products across 24 categories need to be loaded. The mechanism for initial data migration from the existing platform is not specified — agents should provide a migration script or seed mechanism and document the expected data format.

6. **Signal-CLI deployment topology**: The PRD mentions "sidecar or separate pod." Agents should choose based on operational simplicity — a separate pod is generally preferable for independent scaling and failure isolation, but a sidecar reduces network hops for Morgan.

7. **ElevenLabs voice configuration**: SIP/PSTN integration details (Twilio trunk configuration, ElevenLabs agent setup) are not fully specified. Agents should implement a pluggable voice adapter and document required environment variables.

8. **Tax calculation engine**: The Finance Service requires GST/HST, US sales tax, and international tax calculation. Agents should evaluate whether a tax API service (e.g., TaxJar, Avalara) is appropriate or if a simple lookup table suffices for Phase 1 given the company's primary markets.

