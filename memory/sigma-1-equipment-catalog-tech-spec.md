# Sigma-1 Equipment Catalog Service Technical Specification

## Service Overview

**Service Name:** Equipment Catalog Service
**Team Lead:** Rex (Rust Implementation)
**Implementation Language/Framework:** Rust/Axum
**Priority:** High

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Equipment Catalog Service                        │
├─────────────────────────────────────────────────────────────────────┤
│  API Layer                                                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Axum HTTP Server                         │   │
│  │                                                             │   │
│  │  Routes: GET /api/v1/catalog/*                              │   │
│  │          POST /api/v1/catalog/*                             │   │
│  │          PATCH /api/v1/catalog/*                            │   │
│  │          Internal: GET /internal/equipment/*                │   │
│  └─────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│  Business Logic Layer                                               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Product Management Service                     │   │
│  │                                                             │   │
│  │  Functions: Product CRUD, Availability Checking,            │   │
│  │             Search & Filtering, Inventory Sync              │   │
│  └─────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│  Data Access Layer                                                  │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐      │
│  │   PostgreSQL   │  │     Redis      │  │    S3/R2 CDN   │      │
│  │   (Primary)    │  │   (Cache)      │  │   (Images)     │      │
│  │                │  │                │  │                │      │
│  │ Products       │  │ Product Cache  │  │ Product Images │      │
│  │ Categories     │  │ Availability   │  │                │      │
│  │ Inventory      │  │ Search Index   │  │                │      │
│  └────────────────┘  └────────────────┘  └────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Responsibilities

1. Provide high-performance API for equipment inventory management
2. Enable real-time availability checking for rental items
3. Serve machine-readable equipment API for AI agents
4. Manage product images and specifications
5. Support advanced search and filtering capabilities

## API Endpoints

### Public Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | /api/v1/catalog/categories | List all product categories | No |
| GET | /api/v1/catalog/products | List/filter products | No |
| GET | /api/v1/catalog/products/:id | Get product details | No |
| GET | /api/v1/catalog/products/:id/availability | Check date range availability | No |
| GET | /api/v1/equipment-api/catalog | Machine-readable equipment API | No |
| POST | /api/v1/equipment-api/checkout | Programmatic booking | Yes (API Key) |
| GET | /metrics | Prometheus metrics | No |
| GET | /health/live | Liveness probe | No |
| GET | /health/ready | Readiness probe | No |

### Admin Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | /api/v1/catalog/products | Add new product | Yes (Admin) |
| PATCH | /api/v1/catalog/products/:id | Update product | Yes (Admin) |
| DELETE | /api/v1/catalog/products/:id | Remove product | Yes (Admin) |
| POST | /api/v1/catalog/categories | Add new category | Yes (Admin) |
| PATCH | /api/v1/catalog/categories/:id | Update category | Yes (Admin) |

### Internal Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | /internal/equipment/reserve | Reserve equipment for booking | Yes (Service Auth) |
| POST | /internal/equipment/release | Release reserved equipment | Yes (Service Auth) |
| GET | /internal/equipment/bulk | Bulk product information | Yes (Service Auth) |

## Data Models

### Product Model

```rust
struct Product {
    id: Uuid,
    name: String,
    category_id: Uuid,
    description: String,
    day_rate: Decimal,
    weight_kg: Option<f32>,
    dimensions: Option<Dimensions>,
    image_urls: Vec<String>,
    specs: JsonB,
    barcode: Option<String>,
    sku: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    status: ProductStatus, // active, inactive, discontinued
}

struct Dimensions {
    length_cm: f32,
    width_cm: f32,
    height_cm: f32,
}

enum ProductStatus {
    Active,
    Inactive,
    Discontinued,
}
```

### Category Model

```rust
struct Category {
    id: Uuid,
    name: String,
    parent_id: Option<Uuid>,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### Availability Model

```rust
struct Availability {
    product_id: Uuid,
    date: NaiveDate,
    status: AvailabilityStatus, // available, booked, unavailable
    reservation_id: Option<Uuid>,
}

enum AvailabilityStatus {
    Available,
    Booked,
    Unavailable,
}
```

## Integration Points

### Upstream Dependencies

1. **S3/R2 Compatible Storage** - For product image hosting and CDN
2. **Redis Cache** - For frequently accessed product and availability data
3. **PostgreSQL Database** - For persistent product and category data
4. **Prometheus** - For metrics collection and monitoring

### Downstream Consumers

1. **Morgan AI Agent** - Via MCP tools for product search and availability checking
2. **Rental Management Service** - For equipment reservation and booking
3. **Finance Service** - For pricing and quote generation
4. **Web Frontend** - For product browsing and selection
5. **Mobile App** - For on-the-go equipment browsing

## Security Considerations

1. **API Key Authentication** - For programmatic access to booking APIs
2. **JWT-based Admin Authentication** - For administrative endpoints
3. **Rate Limiting** - Per-IP and per-API-key request limiting
4. **Input Validation** - Sanitization of all user inputs to prevent injection attacks
5. **Image Upload Validation** - File type and size restrictions for product images
6. **Audit Logging** - Track all admin actions and significant changes

## Performance Requirements

1. **Response Time:** < 100ms for 95th percentile of simple queries
2. **Throughput:** 5000 requests/second for read operations
3. **Complex Query Response:** < 300ms for filtered product listings
4. **Availability Check:** < 50ms for date range availability
5. **Concurrent Users:** 10000+ active browsing sessions
6. **Image Delivery:** < 500ms for product image loading

## Monitoring & Observability

### Metrics

1. **Request Rate** - Track requests per second by endpoint
2. **Response Time** - Measure 50th, 95th, 99th percentile response times
3. **Error Rate** - Monitor HTTP error codes and internal errors
4. **Cache Hit Ratio** - Track effectiveness of Redis caching
5. **Database Query Performance** - Monitor slow queries and connection pool usage
6. **Image Delivery Performance** - Track CDN performance and fallback usage

### Logging

1. **Request Logging** - Log all API requests with response codes
2. **Error Logging** - Detailed logging of all errors with stack traces
3. **Performance Logging** - Log slow queries and performance bottlenecks
4. **Security Events** - Log authentication attempts and suspicious activities
5. **Business Events** - Log significant events like new product additions

### Tracing

1. **Request Tracing** - End-to-end tracing of API requests
2. **Database Query Tracing** - Tracing of database operations
3. **Cache Operation Tracing** - Tracing of Redis cache operations
4. **External Service Calls** - Tracing of S3/R2 image operations

## Error Handling

### Common Error Scenarios

1. **Product Not Found** - 404 Not Found - Return helpful error message
2. **Invalid Date Range** - 400 Bad Request - Validate date format and logical consistency
3. **Database Connection Failure** - 500 Internal Server Error - Retry with backoff, failover to replica
4. **Cache Miss** - Load from database and populate cache
5. **Image Upload Failure** - 500 Internal Server Error - Retry upload, notify admin

### Retry Logic

1. **Database Failures** - Retry up to 3 times with exponential backoff (100ms, 500ms, 1s)
2. **Cache Failures** - Fall back to database, async retry cache population
3. **S3/R2 Failures** - Retry up to 3 times, serve placeholder images if needed
4. **External Service Timeouts** - Timeout after 5 seconds, return cached data if available

## Deployment Configuration

### Environment Variables

1. `DATABASE_URL` - PostgreSQL connection string - No default
2. `REDIS_URL` - Redis connection string - Default: redis://localhost:6379
3. `S3_ENDPOINT` - S3/R2 endpoint URL - No default
4. `S3_ACCESS_KEY_ID` - S3 access key - No default
5. `S3_SECRET_ACCESS_KEY` - S3 secret key - No default
6. `S3_BUCKET_NAME` - S3 bucket for image storage - Default: sigma-1-products
7. `API_RATE_LIMIT` - Requests per minute per IP - Default: 1000
8. `ADMIN_RATE_LIMIT` - Requests per minute for admin endpoints - Default: 100
9. `CACHE_TTL_SECONDS` - Default cache TTL - Default: 300 (5 minutes)
10. `LOG_LEVEL` - Verbosity of logging - Default: INFO

### Kubernetes Manifests

configs/charts/equipment-catalog/

## Testing Strategy

### Unit Tests

1. **Product CRUD Operations** - 95% coverage of product management logic
2. **Availability Checking** - 95% coverage of date range availability logic
3. **Search & Filtering** - 90% coverage of search and filter functionality
4. **Input Validation** - 100% coverage of input validation logic
5. **Error Handling** - 100% coverage of error scenarios

### Integration Tests

1. **Database Integration** - Test all database operations with real PostgreSQL
2. **Cache Integration** - Test Redis caching behavior and invalidation
3. **S3/R2 Integration** - Test image upload, retrieval, and deletion
4. **API Endpoints** - Test all HTTP endpoints with valid and invalid inputs
5. **Authentication** - Test admin endpoint protection and API key validation

### Load Testing

1. **Read Performance** - Support 5000 requests/second for product listing
2. **Availability Checking** - Handle 1000 concurrent availability checks
3. **Image Delivery** - Serve 10000 concurrent image requests
4. **Cache Effectiveness** - Maintain > 90% cache hit ratio under load
5. **Database Performance** - Maintain < 100ms query response under load

## Dependencies

### External Libraries

1. **Axum** - 0.7+ - Web framework
2. **Tokio** - 1.0+ - Async runtime
3. **SQLx** - 0.7+ - Database access
4. **Redis-rs** - Latest - Redis client
5. **Serde** - Latest - Serialization/deserialization
6. **UUID** - Latest - UUID generation
7. **Chrono** - Latest - Date/time handling
8. **Rust_decimal** - Latest - Decimal arithmetic for pricing

### Services

1. **PostgreSQL Database** - Version 16+ - Primary data storage
2. **Redis Cache** - Version 7+ - Caching layer
3. **S3/R2 Compatible Storage** - For image hosting
4. **Prometheus** - For metrics collection
5. **Morgan AI Agent** - Consumer of API endpoints
6. **Rental Management Service** - Consumer of internal reservation APIs

## Implementation Phases

### Phase 1 - MVP

1. **Basic Product CRUD** - Implement GET/POST/PATCH for products and categories
2. **Simple Availability Model** - Basic date-based availability tracking
3. **Core API Endpoints** - Implement main public API endpoints
4. **Database Integration** - Connect to PostgreSQL and implement data models
5. **Basic Testing Setup** - Unit tests for core functionality

### Phase 2 - Enhanced Functionality

1. **Advanced Search** - Implement filtering, sorting, and pagination
2. **Image Management** - Integrate with S3/R2 for product images
3. **Caching Layer** - Implement Redis caching for frequently accessed data
4. **Availability Optimization** - Improve availability checking performance
5. **Metrics Integration** - Add Prometheus metrics collection

### Phase 3 - Production Ready

1. **Rate Limiting** - Implement per-IP and per-API-key rate limiting
2. **Admin Authentication** - Add JWT-based admin endpoint protection
3. **Security Hardening** - Input validation, audit logging, security headers
4. **Comprehensive Monitoring** - Full observability with tracing and detailed metrics
5. **Load Testing** - Verify performance under production loads

## Acceptance Criteria

### Functional Requirements

1. **Product Management** - Admins can create, read, update, and delete products - Test with CRUD operations
2. **Availability Checking** - System accurately reports equipment availability - Test with various date ranges
3. **Search & Filter** - Users can find products using various criteria - Test with 1000+ product dataset
4. **Image Management** - Product images are properly stored and served - Test upload and retrieval

### Non-Functional Requirements

1. **Response Time** - 95% of requests under 100ms - Load testing verification
2. **Availability** - 99.9% uptime - Monitoring dashboard verification
3. **Scalability** - Support 5000 requests/second - Load testing verification
4. **Security** - Pass automated security scanning - Weekly security scans

## Rollback Plan

1. **Failed Deployment** - Condition: Error rate > 5% for 5 minutes - Steps: Immediately rollback to previous version using Helm - Verification: Monitor error rates return to normal
2. **Performance Degradation** - Condition: Response time > 500ms for 10 minutes - Steps: Rollback to previous version, investigate database queries - Verification: Response times return to normal ranges
3. **Data Corruption** - Condition: Invalid data detected in database - Steps: Stop service, restore from backup, deploy previous version - Verification: Data integrity checks pass

## Timeline

### Development

- **Start Date:** [Upon credential resolution]
- **MVP Complete:** [3 weeks after start]
- **Testing Complete:** [2 weeks after MVP]
- **Production Ready:** [1 week after testing]

### Dependencies

1. **Credential Resolution** - Required by: ASAP
2. **Database Setup** - Required by: Phase 1
3. **S3/R2 Access** - Required by: Phase 2
4. **Redis Deployment** - Required by: Phase 2

## Team Coordination

### Communication Channels

1. **Daily Standup:** 11:00 AM PST on #sigma-1-catalog Discord channel
2. **Weekly Planning:** Fridays 2:30 PM PST on #sigma-1-core Discord channel
3. **Ad-hoc Discussions:** Discord DMs and voice channels as needed

### Stakeholders

1. **edge_kase** - Project Sponsor - Weekly status update
2. **Rex** - Rust Implementation Lead - Daily technical coordination
3. **Cleo** - Code Quality Specialist - Code review sessions
4. **Cipher** - Security Lead - Security review and audit coordination

## Documentation

### To Be Created

1. **API Documentation** - OpenAPI/Swagger format in /docs/api/catalog.yaml
2. **Deployment Guide** - Markdown format in /docs/deployment/catalog.md
3. **Troubleshooting Guide** - Markdown format in /docs/troubleshooting/catalog.md
4. **Database Schema** - Markdown format in /docs/database/catalog.md

### Existing Resources

1. **Sigma-1 PRD** - ~/sigma-1/prd.md
2. **Sigma-1 Architecture** - ~/sigma-1/architecture.md
3. **Equipment Catalog Section** - Relevant sections from PRD and Architecture documents