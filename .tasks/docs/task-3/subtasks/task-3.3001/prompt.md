Implement subtask 3001: Initialize Go project and define gRPC protobuf schemas

## Objective
Initialize a new Go project and define all necessary gRPC service and message protobufs for Opportunity, Project, Inventory, Crew, and Delivery services.

## Steps
1. Run `go mod init <project-name>` targeting Go 1.22.2.2. Create `.proto` files for `OpportunityService`, `ProjectService`, `InventoryService`, `CrewService`, and `DeliveryService`, including their respective message definitions.

## Validation
Verify `go.mod` and `go.sum` are created. Validate `.proto` files for correct syntax using `protoc --verify_only`.