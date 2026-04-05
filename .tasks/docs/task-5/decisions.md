## Decision Points

- What API paradigm should be used for inter-service communication between backend services (e.g., Morgan agent, Equipment Catalog, RMS, Finance, Vetting, Social Engine)?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for service-to-service and user-to-service API calls?
- Should the Finance and Customer Vetting services be implemented as separate Rust/Axum services, or as modules within a single Rex service binary?
- How should the Google Reviews and credit signal data be accessed for customer vetting?
- What versioning strategy should be used for public and internal APIs?
- What approach should be used for GDPR compliance regarding data export and customer deletion?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum