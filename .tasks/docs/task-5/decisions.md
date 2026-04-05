## Decision Points

- Which PostgreSQL operator should be used for managing the main database cluster?
- Should the Finance and Customer Vetting services be implemented as separate Rust/Axum services or merged into a single multi-domain service?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for service-to-service and user-to-service communication?
- Which specific credit API provider should be used for credit signal checks (e.g., Dun & Bradstreet, Experian Business, CreditSafe)?
- How should LinkedIn data be accessed — via official LinkedIn API (Marketing/Community Management), a third-party enrichment service (e.g., Proxycurl, People Data Labs), or web scraping?

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum