# Enhanced PRD

This document synthesizes the original project requirements with the outcomes of a structured design deliberation session. It serves as the single source of truth for all subsequent implementation tasks, detailing both the original vision and the resolved architectural decisions.

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

## 2. Project Scope

The initial task decomposition identified 10 tasks to bootstrap the Sigma-1 platform. The scope covers infrastructure provisioning, backend service creation, frontend website setup, and foundational CI/CD and testing pipelines.

-   **Total Tasks Identified**: 10
    -   `Task 1`: Provision Core Infrastructure (PostgreSQL, Redis, S3/R2).
    -   `Task 2`: Bootstrap Equipment Catalog Service (Rust/Axum).
    -   `Task 3`: Bootstrap RMS Service & gRPC Definitions (Go/gRPC).
    -   `Task 4`: Bootstrap Finance Service (Rust/Axum).
    -   `Task 5`: Bootstrap Customer Vetting Service (Rust/Axum).
    -   `Task 6`: Bootstrap Social Media Engine (Bun/Elysia).
    -   `Task 7`: Setup Website & Equipment Catalog Pages (React/Next.js).
    -   `Task 8`: Bootstrap Morgan AI Agent & Catalog Tool (OpenClaw).
    -   `Task 9`: Establish CI/CD Pipeline Foundations (GitHub Actions).
    -   `Task 10`: Create Integration Test for Lead Qualification.

-   **Key Services & Components**:
    -   **Backend**: Equipment Catalog, RMS, Finance, Customer Vetting, Social Media Engine.
    -   **Frontend**: Next.js Website.
    -   **AI**: Morgan Agent (OpenClaw).
    -   **Infrastructure**: PostgreSQL, Redis/Valkey, Cloudflare R2, Kubernetes.

-   **Agent Assignments & Technology Stacks**:
    -   **Bolt**: Kubernetes/Helm for infrastructure.
    -   **Rex**: Rust/Axum for Catalog, Finance, and Vetting services.
    -   **Grizz**: Go/gRPC for the RMS service.
    -   **Nova**: Bun/Elysia/Effect for the Social Media Engine.
    -   **Blaze**: React/Next.js for the website.
    -   **Angie**: OpenClaw/MCP for the Morgan agent.
    -   **Atlas**: CI/CD platforms for pipelines.
    -   **Tess**: Test frameworks for integration tests.

-   **Cross-cutting Concerns**:
    -   CI/CD pipelines will be established for all services (Task 9).
    -   End-to-end integration testing will validate core data flows (Task 10).

## 3. Resolved Decisions

The following decisions were resolved by unanimous agreement during the deliberation session.

### [D2] Which object storage provider should be used for storing product images and social media content?

-   **Status**: Accepted
-   **Task Context**: 1 (Provision Core Infrastructure), 2 (Bootstrap Equipment Catalog Service), 6 (Bootstrap Social Media Engine)
-   **Context**: Both Optimist and Pessimist agreed that Cloudflare R2 is the pragmatic choice. The S3-compatible API minimizes technical risk, and its integration with the Cloudflare-hosted frontend eliminates data egress costs.
-   **Decision**: Use Cloudflare R2 for all object storage needs.
-   **Consensus**: 2/2 (100%)
-   **Consequences**:
    -   **Positive**: Significant and compounding cost savings on data egress for image-heavy services. Aligns with existing Cloudflare infrastructure (Pages, CDN).
    -   **Negative**: None identified. The S3-compatible API makes it a low-risk choice.

### [D6] How will the Customer Vetting service source Google Reviews data?

-   **Status**: Accepted
-   **Task Context**: 5 (Bootstrap Customer Vetting Service)
-   **Context**: Both Optimist and Pessimist agreed that building and maintaining a custom web scraper against a target like Google is a fragile, low-value engineering task that creates significant maintenance debt.
-   **Decision**: Use a paid, third-party data provider API for reliable access to Google Reviews data.
-   **Consensus**: 2/2 (100%)
-   **Consequences**:
    -   **Positive**: Offloads maintenance burden, provides a stable API contract, and allows engineers to focus on core business logic. Trades a predictable operational expense for a massive reduction in engineering toil and incident risk.
    -   **Negative**: Introduces a new operational expense for the third-party API.

### [D7] How should the Signal-CLI dependency for the Morgan agent be deployed?

-   **Status**: Accepted
-   **Task Context**: 8 (Bootstrap Morgan AI Agent & Catalog Tool)
-   **Context**: Both Optimist and Pessimist agreed that the relationship between the Morgan agent and Signal-CLI is 1:1 and tightly coupled. The sidecar pattern is the textbook correct implementation for this use case.
-   **Decision**: Deploy Signal-CLI as a sidecar container within the same Kubernetes pod as the Morgan agent.
-   **Consensus**: 2/2 (100%)
-   **Consequences**:
    -   **Positive**: Co-locates the containers, simplifies networking to `localhost`, and ensures their lifecycles are managed together. Avoids unnecessary network and service discovery complexity.
    -   **Negative**: None identified.

## 4. Escalated Decisions

The following decision points resulted in a split opinion and require human review to resolve. Implementing agents **must not** proceed with tasks affected by these decisions until a final resolution is provided.

### [D1] How should data be isolated for the various backend services (RMS, Finance, Catalog, etc.) within the single PostgreSQL cluster? — ESCALATED

-   **Status**: Pending human decision
-   **Task Context**: 1, 2, 3, 4, 5, 6
-   **Options**:
    -   A: Use a single PostgreSQL database with logically separated schemas for each service.
    -   B: Provision separate, physically isolated PostgreSQL databases for each service.
-   **Optimist argued (A)**: A single database with schemas simplifies infrastructure provisioning and connection management, aligning with the PRD's hint. It reduces operational overhead.
-   **Pessimist argued (B)**: A single database is a single point of failure. Separate databases provide true service isolation, contain the blast radius of a bad migration or runaway query, and enforce strong boundaries. The CloudNative-PG operator makes provisioning new databases trivial.
-   **Recommendation**: **Option B (Separate Databases)**. For a V1 system, prioritizing reliability and blast radius containment is paramount. The operational cost of managing a few separate databases with the specified operator is low compared to the risk of a platform-wide outage caused by a single service's database error.

### [D3] What should be the primary communication protocol for internal service-to-service calls? — ESCALATED

-   **Status**: Pending human decision
-   **Task Context**: 2, 3, 4, 5, 6, 8, 10
-   **Options**:
    -   A: Standardize on gRPC for all internal communication.
    -   B: Use a mixed-protocol approach: gRPC for high-throughput services (RMS) and HTTP/REST for others.
-   **Optimist argued (A)**: A unified gRPC protocol provides performance, type safety, and a superior developer experience through a central repository of Protobuf definitions, preventing integration bugs.
-   **Pessimist argued (B)**: Forcing gRPC on simple CRUD services adds unnecessary complexity (Protobuf management, code generation, gateways). Simple REST/JSON is easier to implement, secure, and debug with standard tools like `curl` during an incident.
-   **Recommendation**: **Option B (Mixed Protocol)**. The PRD already implies this pattern. The operational benefit of simple, debuggable REST APIs for services like the Equipment Catalog outweighs the benefits of universal gRPC standardization at this stage. Use gRPC where its performance is critical (RMS) and REST where simplicity and debuggability are more valuable.

### [D4] How will authentication and authorization be handled for service-to-service calls and external API access from the frontend? — ESCALATED

-   **Status**: Pending human decision
-   **Task Context**: 2, 3, 4, 5, 6, 7, 8
-   **Options**:
    -   A: Use a service mesh with mTLS for service-to-service communication and JWTs for external clients.
    -   B: Implement service-to-service authentication using short-lived JWTs issued by a central identity service.
-   **Optimist argued (A)**: A service mesh pushes security to the infrastructure layer, freeing developers from implementing token validation. It's a modern, secure approach for a zero-trust network.
-   **Pessimist argued (B)**: A service mesh is a complex distributed system that adds an enormous layer of opaque complexity, increasing mean-time-to-resolution (MTTR) during incidents. JWTs are a well-understood, debuggable, and stateless technology that is sufficient for V1.
-   **Recommendation**: **Option B (JWTs)**. The operational burden and learning curve of a service mesh are too high for a V1 project. A simple, robust JWT-based system is easier to debug and operate, which is critical for a new platform. A service mesh can be introduced later if the need is proven.

### [D5] What architectural pattern should be used for orchestrating long-running, multi-step business workflows? — ESCALATED

-   **Status**: Pending human decision
-   **Task Context**: 5, 6, 8, 10
-   **Options**:
    -   A: Use an event-driven approach with the existing NATS message queue.
    -   B: Use direct orchestration from the Morgan agent, offloading long-running steps to a simple background job queue.
-   **Optimist argued (A)**: An event-driven architecture using NATS decouples services, creating a resilient, scalable, and observable system. This is a proven pattern for robust microservices.
-   **Pessimist argued (B)**: A fully event-driven system is difficult to trace and debug without extensive, perfectly configured distributed tracing. Direct orchestration with a simple Redis-backed job queue provides the necessary asynchronicity and resilience without the debugging nightmare.
-   **Recommendation**: **Option B (Direct Orchestration + Job Queue)**. Traceability is king in a new system. Understanding a full business process flow is critical for debugging. The direct orchestration model provides a clear, traceable path for core business logic, which is more valuable than the loose coupling of an event-driven system at this early stage.

### [D8] How will database schema migrations be managed across the multiple services using the PostgreSQL cluster? — ESCALATED

-   **Status**: Pending human decision
-   **Task Context**: 2, 3, 4, 5, 6
-   **Options**:
    -   A: Each service manages its own schema migrations independently using a language-specific tool.
    -   B: Use a single, language-agnostic migration tool managed from a central repository.
-   **Optimist argued (A)**: A core principle of microservices is autonomy. Empowering each service to manage its own migrations within its own CI/CD pipeline improves developer experience and velocity.
-   **Pessimist argued (B)**: Autonomy is not anarchy. A central tool creates a single, auditable source of truth and prevents uncoordinated, conflicting schema changes from taking down the database. This "bottleneck" is a feature that forces communication.
-   **Recommendation**: **Option A (Per-service Migrations)**. This recommendation is contingent on the acceptance of the recommendation for **[D1]** to use separate databases. If each service has its own database, the risk of collision is eliminated, and the principle of service autonomy can be safely applied. This aligns best with a microservices philosophy.

### [D9] Which Redis-compatible engine should be used for caching and rate-limiting? — ESCALATED

-   **Status**: Pending human decision
-   **Task Context**: 1, 2, 3, 4
-   **Options**:
    -   A: Use Valkey, as specified in the example Kubernetes manifest.
    -   B: Use a standard Redis deployment.
-   **Optimist argued (A)**: Valkey is the community-driven, open-source future of Redis. Adopting it aligns with the open-source community and insulates the project from future licensing changes by Redis Inc.
-   **Pessimist argued (B)**: Redis is battle-tested for a decade; Valkey is months old. The risk of running critical production caching on an unproven engine is an unforced error. We should use "boring technology" for core infrastructure.
-   **Recommendation**: **Option B (Standard Redis)**. For critical infrastructure like caching and rate-limiting, stability and operational history are more important than aligning with a new fork. The risk of unknown bugs or behavioral differences in Valkey is too high for a V1 launch.

### [D10] How will the Morgan web chat widget be implemented and integrated into the Next.js site? — ESCALATED

-   **Status**: Pending human decision
-   **Task Context**: 7, 8
-   **Options**:
    -   A: Build a custom React component for the chat widget from scratch.
    -   B: Source a third-party, embeddable chat widget library and connect it to the Morgan agent's backend.
-   **Optimist argued (A)**: Morgan is the centerpiece of the product. A custom component gives complete control over UX, branding, and unique features, which is a direct investment in product quality.
-   **Pessimist argued (B)**: Building a chat widget is a notorious time sink that involves many solved problems (history, typing indicators, accessibility). Using a library allows the team to ship faster and focus on the backend logic that makes Morgan smart.
-   **Recommendation**: **Option B (Third-party Library)**. The primary goal for V1 is to ship a functional end-to-end product. The engineering effort required to build a production-ready chat widget from scratch is significant and will delay the project. A well-maintained open-source library can be styled to match the brand and replaced with a custom version in a future phase if it becomes a competitive differentiator.

## 5. Architecture Overview

Based on the resolved decisions, the architecture will adhere to the following principles. Note that this overview is partial, pending the resolution of the escalated decisions above.

-   **Technology Stacks**: The project will use a polyglot microservice architecture as defined in the PRD:
    -   **Rust/Axum**: For high-performance REST services (Equipment Catalog, Finance, Customer Vetting).
    -   **Go/gRPC**: For the core RMS service.
    -   **Node.js/Elysia/Effect**: For the Social Media Engine.
    -   **Next.js/React**: For the frontend website.
    -   **OpenClaw**: For the Morgan AI agent.
-   **Service Architecture**: Services will be deployed as containers in a single Kubernetes cluster.
-   **Object Storage**: All services requiring object storage for images, photos, or other assets will use **Cloudflare R2** via its S3-compatible API.
-   **Third-Party Data**: The Customer Vetting service will integrate with a **paid, third-party API** for sourcing Google Reviews data.
-   **Key Patterns**:
    -   The **sidecar pattern** is mandated for deploying the Signal-CLI dependency alongside the Morgan agent.
-   **Explicitly Ruled Out**:
    -   Building and maintaining an in-house web scraper for Google Reviews data is not permitted.

## 6. Implementation Constraints

All implementing agents must adhere to the following hard constraints:

-   **Performance Targets**:
    -   Morgan must respond within 10 seconds for simple queries.
    -   Equipment availability check must be < 500ms.
    -   Invoice generation must be < 5 seconds.
-   **Operational Requirements**:
    -   The platform must support 500+ concurrent Signal connections.
    -   All production services must maintain 99.9% uptime.
-   **Security & Compliance**:
    -   The platform must be GDPR compliant, supporting data export and customer deletion requests.
-   **Service Dependencies & Integration Points**:
    -   All object storage must target the provisioned Cloudflare R2 bucket.
    -   The Customer Vetting service must use a commercial API for Google Reviews data, not a custom scraper.
    -   The Morgan agent's Kubernetes deployment must include Signal-CLI as a sidecar container.

## 7. Design Intake Summary

The project includes a significant frontend component with opportunities for UI modernization.

-   **Frontend Presence**: `hasFrontend` is **true**.
-   **Frontend Targets**: The project must support both **web** (`Next.js`) and **mobile** (`Expo`) clients as specified in the PRD.
-   **Supplied Artifacts**: No design artifacts (e.g., Figma files, wireframes) were supplied.
-   **Reference URLs**: No reference URLs were supplied.
-   **Stitch Analysis**: The `stitch_status` is **generated**, indicating that the codebase was analyzed and concrete UI modernization opportunities were identified.
-   **Implications**: Frontend implementation tasks for the web (Task 7) and the future mobile app must account for both targets. The absence of design artifacts means implementing agents will rely on the specified component library (`shadcn/ui`) and their own judgment to create a clean, functional UI, pending further design deliberation.

## 8. Open Questions

All initial open questions identified during task parsing were elevated to formal decision points during the deliberation session. There are no remaining non-blocking items for which implementing agents should use their own judgment. The unresolved items are now classified as **Escalated Decisions** in Section 4 and require a formal resolution before work can proceed.
