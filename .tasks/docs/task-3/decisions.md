## Decision Points

- Choice of ORM or database access library for Go (e.g., `sqlx`, GORM, or raw SQL).
- Initial mocking strategy for barcode scanning, crew scheduling, and delivery tracking external integrations.
- Specific data structures and relationships for Opportunity, Project, and InventoryTransaction.
- Error handling and retry mechanisms for gRPC and REST API calls.

## Coordination Notes

- Agent owner: grizz
- Primary stack: Go/gRPC