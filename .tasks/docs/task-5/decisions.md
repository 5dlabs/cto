## Decision Points

- Which Redis-compatible cache should be used for rate limiting, session storage, and caching across services?
- Should the Finance and Customer Vetting services be implemented as separate Rust/Axum services or merged into a single multi-domain service?
- What API paradigm should be used for inter-service communication between Morgan (OpenClaw) and backend services?
- How should multi-tenancy and schema separation be handled in the PostgreSQL database?
- What authentication and authorization mechanism should be used for internal service-to-service API calls?
- How should the public-facing API endpoints (e.g., Equipment Catalog, RMS REST, Finance) be versioned and exposed?
- What access control model should be used for admin endpoints (e.g., product add/update, finance, vetting)?
- Which specific credit data API provider should be integrated for credit scoring (e.g., Experian, Equifax, Dun & Bradstreet, CreditSafe)?
- What weighting algorithm should be used to aggregate OpenCorporates, LinkedIn, Google Reviews, and credit scores into the final GREEN/YELLOW/RED classification?
- Should Google Reviews data be obtained via official Google Places API (paid, reliable) or web scraping (free, fragile)?

## Coordination Notes

- Agent owner: Rex
- Primary stack: Rust/Axum