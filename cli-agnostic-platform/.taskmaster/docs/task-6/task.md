# Task 6: Build MCP Transport Abstraction Layer

## Overview
Create unified MCP communication layer supporting streaming, buffered relay, and STDIO patterns across different CLI implementations. This abstraction enables consistent tool communication regardless of CLI transport mechanisms.

## Technical Specification

### 1. Transport Architecture
```typescript
abstract class Transport {
    abstract async call(request: MCPRequest): Promise<MCPResponse>;
    abstract supports(capability: TransportCapability): boolean;
    abstract getMetrics(): TransportMetrics;
}

class DirectStreamingTransport extends Transport {
    // HTTP streaming for Claude, Opencode, Gemini
}

class BufferedRelayTransport extends Transport {
    // Buffered communication for Codex STDIO
}

class StdioTransport extends Transport {
    // Direct STDIO for local MCP servers
}
```

### 2. Auto-Detection System
```typescript
class TransportDetector {
    static async detectForCLI(cliType: CLIType): Promise<Transport> {
        const capabilities = await this.probeCapabilities(cliType);

        if (capabilities.supportsStreaming) {
            return new DirectStreamingTransport(capabilities.endpoint);
        } else if (capabilities.supportsStdio) {
            return new BufferedRelayTransport(capabilities.bufferSize);
        }

        return new StdioTransport();
    }
}
```

## Implementation Steps

### Phase 1: Core Transport Framework
1. Create abstract Transport class with standard interface
2. Implement transport capability detection
3. Build transport factory with auto-selection
4. Add comprehensive error handling

### Phase 2: Transport Implementations
1. DirectStreamingTransport for HTTP streaming CLIs
2. BufferedRelayTransport for STDIO-based CLIs
3. StdioTransport for local MCP servers
4. Add retry logic with exponential backoff

### Phase 3: Performance and Reliability
1. Connection pooling for HTTP transports
2. Compression support for large payloads
3. Request/response correlation tracking
4. Comprehensive metrics collection

## Success Criteria
- Unified interface for all MCP communication patterns
- Auto-detection selects optimal transport per CLI
- Streaming, buffered, and STDIO patterns supported
- <500ms p99 latency for tool calls
- Retry logic handles transient failures
- Compression reduces bandwidth by >50% for large payloads