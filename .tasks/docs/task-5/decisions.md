## Decision Points

- LinkedIn data retrieval strategy: use official LinkedIn Company API (requires partnership/OAuth), or fall back to HTTP scraping of public company pages, or use a third-party enrichment service (e.g., Proxycurl, PeopleDataLabs). This affects reliability and legal compliance.
- Google Reviews data source: Google Business Profile API vs Google Places API vs structured search scraping. Each has different pricing, rate limits, and ToS implications.
- Credit provider selection: which commercial credit API to integrate (e.g., Dun & Bradstreet, Experian Business, CreditSafe)? The trait abstraction is decided, but the initial concrete implementation beyond the stub needs a vendor choice.
- Scoring weight calibration: the 30/20/20/20/10 point allocation and GREEN≥70/YELLOW≥40/RED<40 thresholds are specified but may need business stakeholder validation before hardcoding.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust 1.75+/Axum 0.7