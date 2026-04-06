## Decision Points

- How should multi-tenancy be handled in the PostgreSQL schema for all backend services?
- What API paradigm should be used for inter-service communication between backend services and the Morgan agent?
- What authentication and authorization mechanism should be used for API access between services and for the frontend?
- How should the public API endpoints be versioned and documented for external and internal consumers?
- Should the Finance and Customer Vetting services be implemented as separate microservices or merged into a single Rust/Axum service?

## Coordination Notes

- Agent owner: Rex
- Primary stack: Rust 1.75+/Axum 0.7