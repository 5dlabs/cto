Implement subtask 3002: Define protobuf schemas for all five RMS domains

## Objective
Write proto/sigma1/rms/v1/opportunity.proto, project.proto, inventory.proto, crew.proto, and delivery.proto with grpc-gateway HTTP annotations for REST mapping. Run buf generate to produce Go stubs.

## Steps
Each proto file must define the service, all request/response messages, and google.api.http option annotations for grpc-gateway. opportunity.proto: OpportunityService with CreateOpportunity (POST /api/v1/opportunities), GetOpportunity (GET /api/v1/opportunities/{id}), UpdateOpportunity (PUT /api/v1/opportunities/{id}), ListOpportunities (GET /api/v1/opportunities), ScoreLead (POST /api/v1/opportunities/{id}/score). project.proto: ProjectService with CreateProject, GetProject, UpdateProject, CheckOut (POST /api/v1/projects/{id}/checkout), CheckIn (POST /api/v1/projects/{id}/checkin). inventory.proto: InventoryService with GetStockLevel, RecordTransaction, ScanBarcode. crew.proto: CrewService with ListCrew, AssignCrew, ScheduleCrew. delivery.proto: DeliveryService with ScheduleDelivery, UpdateDeliveryStatus, OptimizeRoute. Run `buf generate` and verify generated files appear in gen/go/sigma1/rms/v1/.

## Validation
`buf generate` exits 0 with no errors. Generated *_grpc.pb.go and *.pb.go files exist for all five domains. `buf lint` returns zero violations. `go build ./...` still passes with generated code included.