Implement subtask 3006: Implement InventoryService with atomic conflict detection and unit tests

## Objective
Implement InventoryService handlers (GetStockLevel, RecordTransaction, ScanBarcode) and the inventory conflict detection logic that returns ConflictError on overlapping checkout windows. Write unit tests for conflict detection.

## Steps
Create internal/inventory/handler.go and internal/inventory/conflict.go. RecordTransaction: before inserting a checkout transaction, execute SELECT COUNT(*) FROM rms.inventory_transactions WHERE inventory_id=$1 AND type='checkout' AND timestamp BETWEEN $2 AND $3 within the same pgx transaction using SELECT FOR UPDATE on the inventory row to prevent races. If count > 0, return gRPC status.Error(codes.AlreadyExists, 'inventory conflict') which grpc-gateway maps to HTTP 409. GetStockLevel: query distinct inventory_ids checked out vs checked in to compute availability. ScanBarcode: accept barcode string, resolve to inventory_id via product catalog lookup (use a stub returning fixed UUID if catalog service is not yet available). Unit tests in internal/inventory/conflict_test.go: mock pgx rows, assert ConflictError returned when overlapping window exists, no error when window is free.

## Validation
POST checkout for inventory_id X in window [T1,T2] succeeds (201). Second POST checkout for same inventory_id X in overlapping window returns HTTP 409. go test ./internal/inventory/... passes all conflict unit tests. GetStockLevel returns numeric availability count.