# Sigma-1 Morgan AI Agent Technical Specification

## Service Overview

**Service Name:** Morgan AI Agent
**Team Lead:** Angie (Agent Architecture)
**Implementation Language/Framework:** OpenClaw
**Priority:** Critical

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Morgan AI Agent                             │
├─────────────────────────────────────────────────────────────────────┤
│  Communication Layers                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  Signal  │  │   Voice  │  │   Web    │  │  Mobile  │        │
│  │(OpenClaw)│  │(ElevenLabs│ │(Next.js) │  │  (Expo)  │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       │             │             │             │                 │
├───────┴─────────────┴─────────────┴─────────────┴─────────────────┤
│  Core Agent Logic                                                   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    OpenClaw Agent (Morgan)                  │   │
│  │                                                             │   │
│  │  Skills: sales-qual, customer-vet, quote-gen, upsell,       │   │
│  │          finance, social-media, rms-*, admin                │   │
│  │                                                             │   │
│  │  MCP Tools: sigma1_catalog_search, sigma1_check_availability│   │
│  │             sigma1_generate_quote, sigma1_vet_customer,     │   │
│  │             sigma1_score_lead, sigma1_create_invoice,       │   │
│  │             sigma1_finance_report, sigma1_social_curate,    │   │
│  │             sigma1_social_publish, sigma1_equipment_lookup  │   │
│  └─────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│  Backend Service Integration                                        │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐      │
│  │   Equipment    │  │     RMS        │  │    Finance     │      │
│  │   Catalog      │  │   Service      │  │    Service     │      │
│  │   (Rust/Axum)  │  │   (Go/gRPC)    │  │   (Rust/Axum)  │      │
│  │     Rex        │  │     Grizz      │  │     Rex        │      │
│  └────────────────┘  └────────────────┘  └────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Responsibilities

1. Serve as the central AI agent handling all customer interactions via Signal, voice, and web chat
2. Route natural language queries to appropriate backend services through MCP tools
3. Execute predefined skills for common business workflows
4. Maintain conversation context and state across multiple channels
5. Provide a consistent user experience regardless of communication channel

## API Endpoints

### Public Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | /api/v1/morgan/message | Receive message from any channel | Yes (JWT) |
| POST | /api/v1/morgan/voice | Receive voice input for processing | Yes (JWT) |
| GET | /api/v1/morgan/status | Check agent health and status | No |
| GET | /api/v1/morgan/conversations/{id} | Retrieve conversation history | Yes (JWT) |

### Internal Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | /internal/morgan/tool-response | Receive response from MCP tools | Yes (Service Auth) |
| POST | /internal/morgan/skill-complete | Notification when skill completes | Yes (Service Auth) |

## Data Models

### Conversation Model

```typescript
interface Conversation {
    id: string;
    userId: string;
    channelId: string;
    channelType: 'signal' | 'voice' | 'web' | 'mobile';
    messages: Message[];
    context: Record<string, any>;
    createdAt: Date;
    updatedAt: Date;
    status: 'active' | 'closed' | 'escalated';
}
```

### Message Model

```typescript
interface Message {
    id: string;
    conversationId: string;
    role: 'user' | 'assistant' | 'system';
    content: string;
    contentType: 'text' | 'image' | 'audio' | 'document';
    timestamp: Date;
    metadata: Record<string, any>;
}
```

## Integration Points

### Upstream Dependencies

1. **Signal-CLI** - Message sending/receiving via Signal messenger
2. **ElevenLabs API** - Voice synthesis and transcription
3. **Twilio API** - Phone number management and PSTN/SIP calls
4. **Web Chat Widget** - Real-time messaging via web interface
5. **Mobile App** - Native mobile application integration

### Downstream Consumers

1. **Equipment Catalog Service** - Via MCP tools for product search and availability
2. **Rental Management Service** - Via MCP tools for booking and reservation
3. **Finance Service** - Via MCP tools for quotes and invoices
4. **Social Media Engine** - Via MCP tools for content curation and publishing (Phase 2)
5. **Customer Vetting Service** - Via MCP tools for background checks (Phase 2)

## Security Considerations

1. **JWT-based Authentication** - Secure API access with token expiration
2. **End-to-End Encryption** - For sensitive customer communications
3. **Role-Based Access Control** - Different permissions for customer vs admin interactions
4. **Audit Logging** - Comprehensive logs of all interactions for compliance
5. **PII Protection** - Automatic detection and protection of personally identifiable information

## Performance Requirements

1. **Response Time:** < 500ms for 95th percentile of simple queries
2. **Throughput:** 100 concurrent conversations
3. **Concurrent Users:** 1000+ active users
4. **Availability:** 99.9% uptime
5. **Voice Latency:** < 200ms round-trip for voice interactions

## Monitoring & Observability

### Metrics

1. **Conversation Volume** - Track number of conversations per hour/day
2. **Response Time** - Measure 50th, 95th, 99th percentile response times
3. **Error Rate** - Monitor failed requests and tool invocations
4. **Skill Success Rate** - Track successful completion of each skill
5. **Channel Distribution** - Monitor usage across Signal, voice, web, mobile

### Logging

1. **Conversation Flow** - Log conversation state transitions
2. **Tool Invocations** - Log all MCP tool calls with parameters and results
3. **Skill Executions** - Log skill start, progress, and completion
4. **Error Conditions** - Detailed logging of all errors and exceptions
5. **Performance Data** - Response times and resource utilization

### Tracing

1. **Conversation Traces** - End-to-end tracing of conversation flows
2. **Tool Execution Traces** - Tracing of MCP tool invocations
3. **Skill Execution Traces** - Tracing of skill workflows
4. **Cross-Service Calls** - Distributed tracing across all integrated services

## Error Handling

### Common Error Scenarios

1. **MCP Tool Failure** - 502 Bad Gateway - Retry with exponential backoff, escalate to human if persistent
2. **Authentication Failure** - 401 Unauthorized - Prompt user for re-authentication
3. **Rate Limiting** - 429 Too Many Requests - Queue request and retry after delay
4. **Service Unavailable** - 503 Service Unavailable - Retry with backoff, notify user of delay
5. **Invalid Input** - 400 Bad Request - Provide helpful error message to user

### Retry Logic

1. **MCP Tool Failures** - Retry up to 3 times with exponential backoff (1s, 2s, 4s)
2. **External API Failures** - Retry up to 5 times with exponential backoff
3. **Database Connection Issues** - Retry up to 3 times with linear backoff
4. **Network Timeouts** - Retry up to 2 times with immediate retry

## Deployment Configuration

### Environment Variables

1. `MORGAN_JWT_SECRET` - Secret for JWT token signing - No default
2. `ELEVENLABS_API_KEY` - API key for voice services - No default
3. `TWILIO_ACCOUNT_SID` - Twilio account identifier - No default
4. `TWILIO_AUTH_TOKEN` - Twilio authentication token - No default
5. `SIGNAL_CLI_URL` - URL for Signal-CLI service - Default: http://localhost:8080
6. `LOG_LEVEL` - Verbosity of logging - Default: INFO

### Kubernetes Manifests

configs/charts/morgan-agent/

## Testing Strategy

### Unit Tests

1. **Message Routing** - 95% coverage of message routing logic
2. **Skill Execution** - 90% coverage of skill workflows
3. **Tool Integration** - 90% coverage of MCP tool invocations
4. **Error Handling** - 100% coverage of error scenarios

### Integration Tests

1. **Signal Integration** - Test message sending/receiving via Signal
2. **Voice Integration** - Test voice-to-text and text-to-voice conversion
3. **Web Chat Integration** - Test real-time messaging via web widget
4. **MCP Tool Integration** - Test all 10 MCP tools with backend services
5. **Skill Workflows** - Test end-to-end execution of all 8 skills

### Load Testing

1. **Concurrent Conversations** - Support 100 simultaneous conversations with < 500ms response time
2. **Voice Traffic** - Handle 20 simultaneous voice calls with < 200ms latency
3. **Peak Load** - Scale to 1000 active users with graceful degradation

## Dependencies

### External Libraries

1. **OpenClaw Framework** - Latest stable version - Core agent functionality
2. **ElevenLabs SDK** - Latest version - Voice synthesis and recognition
3. **Twilio SDK** - Latest version - Phone and SMS integration
4. **JWT Library** - Industry standard version - Authentication tokens
5. **WebSocket Library** - Latest stable version - Real-time web communication

### Services

1. **Equipment Catalog Service** - Internal service - Product search and availability
2. **Rental Management Service** - Internal service - Booking and reservations
3. **Finance Service** - Internal service - Quotes and invoicing
4. **Signal-CLI Service** - Internal service - Signal messaging
5. **PostgreSQL Database** - Internal service - Conversation and message storage
6. **Redis Cache** - Internal service - Session and context caching

## Implementation Phases

### Phase 1 - MVP

1. **Basic Signal Integration** - Send/receive text messages via Signal
2. **Simple Message Routing** - Route basic queries to hardcoded responses
3. **Core MCP Tools** - Implement 3 essential MCP tools (catalog search, availability, quote gen)
4. **Basic Testing Setup** - Unit tests for message routing and tool integration

### Phase 2 - Enhanced Functionality

1. **Voice Integration** - Full voice calling with ElevenLabs
2. **Web Chat Widget** - Real-time web interface
3. **Complete MCP Tool Set** - All 10 MCP tools operational
4. **Skill Implementation** - First 4 skills functional (sales-qual, customer-vet, quote-gen, finance)
5. **Performance Optimizations** - Caching and asynchronous processing

### Phase 3 - Production Ready

1. **Full Skill Suite** - All 8 skills operational
2. **Monitoring Integration** - Comprehensive metrics and logging
3. **Security Hardening** - End-to-end encryption and access controls
4. **Documentation Completion** - Full API docs and user guides
5. **Load Testing** - Verified performance under production loads

## Acceptance Criteria

### Functional Requirements

1. **Multi-Channel Support** - User can interact via Signal, voice, web, and mobile - Test across all channels
2. **Natural Language Processing** - User queries are correctly interpreted - Accuracy > 90% on test suite
3. **Backend Service Integration** - All MCP tools function correctly - 100% of tool calls succeed in testing
4. **Skill Execution** - All skills complete successfully - 95% success rate on skill execution

### Non-Functional Requirements

1. **Response Time** - 95% of responses under 500ms - Load testing verification
2. **Availability** - 99.9% uptime - Monitoring dashboard verification
3. **Security** - No unauthorized access in penetration testing - Third-party security audit
4. **Scalability** - Support 1000+ concurrent users - Load testing verification

## Rollback Plan

1. **Failed Deployment** - Condition: Error rate > 5% for 5 minutes - Steps: Immediately rollback to previous version using Helm - Verification: Monitor error rates return to normal
2. **Performance Degradation** - Condition: Response time > 1s for 10 minutes - Steps: Rollback to previous version, investigate database queries - Verification: Response times return to normal ranges
3. **Security Breach** - Condition: Unauthorized access detected - Steps: Immediately shut down service, rotate all credentials, deploy patched version - Verification: Security audit confirms breach is contained

## Timeline

### Development

- **Start Date:** [Upon credential resolution]
- **MVP Complete:** [2 weeks after start]
- **Testing Complete:** [1 week after MVP]
- **Production Ready:** [1 week after testing]

### Dependencies

1. **Credential Resolution** - Required by: ASAP
2. **Backend Service APIs** - Required by: Phase 1
3. **Infrastructure Setup** - Required by: Phase 1

## Team Coordination

### Communication Channels

1. **Daily Standup:** 10:00 AM PST on #sigma-1-morgan Discord channel
2. **Weekly Planning:** Fridays 2:00 PM PST on #sigma-1-core Discord channel
3. **Ad-hoc Discussions:** Discord DMs and voice channels as needed

### Stakeholders

1. **edge_kase** - Project Sponsor - Weekly status update
2. **Angie** - Agent Architecture Lead - Daily technical coordination
3. **Stitch** - Code Review Specialist - Code review sessions
4. **Keeper** - Operations Lead - Deployment and monitoring coordination

## Documentation

### To Be Created

1. **API Documentation** - OpenAPI/Swagger format in /docs/api/morgan.yaml
2. **Deployment Guide** - Markdown format in /docs/deployment/morgan.md
3. **Troubleshooting Guide** - Markdown format in /docs/troubleshooting/morgan.md
4. **User Guide** - Markdown format in /docs/user/morgan.md

### Existing Resources

1. **Sigma-1 PRD** - ~/sigma-1/prd.md
2. **Sigma-1 Architecture** - ~/sigma-1/architecture.md
3. **Morgan Agent Analysis** - ~/5dlabs/cto/memory/sigma-1-morgan-agent-analysis.md