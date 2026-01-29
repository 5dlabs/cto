---
type: research
title: gRPC Best Practices for AlertHub Admin API
---

# gRPC Implementation Patterns

This document covers gRPC best practices for the AlertHub Admin API (Grizz agent).

## Service Definition

### Proto File Structure

```protobuf
service TenantService {
  rpc CreateTenant(CreateTenantRequest) returns (Tenant);
  rpc GetTenant(GetTenantRequest) returns (Tenant);
  rpc UpdateTenant(UpdateTenantRequest) returns (Tenant);
  rpc ListTenants(ListTenantsRequest) returns (ListTenantsResponse);
}
```

## Go Implementation Patterns

### Server Setup with grpc-gateway

```go
import (
    "google.golang.org/grpc"
    "github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
)

func main() {
    grpcServer := grpc.NewServer()
    pb.RegisterTenantServiceServer(grpcServer, &tenantServer{})
    
    // REST gateway
    mux := runtime.NewServeMux()
    pb.RegisterTenantServiceHandlerServer(ctx, mux, &tenantServer{})
}
```

### Error Handling

```go
import "google.golang.org/grpc/codes"
import "google.golang.org/grpc/status"

return nil, status.Errorf(codes.NotFound, "tenant %s not found", id)
```

## References

- https://grpc.io/docs/languages/go/basics/ - Go gRPC guide
- https://github.com/grpc-ecosystem/grpc-gateway - gRPC gateway
