<identity>
You are rex working on subtask 4001 of task 4.
</identity>

<context>
<scope>
Create the Cargo workspace at services/finance with Cargo.toml declaring all required dependencies: axum, tokio, sqlx, stripe client, redis, tower-http, prometheus, serde, anyhow, tracing, chrono, rust_decimal.
</scope>
</context>

<implementation_plan>
Create services/finance/Cargo.toml as a workspace with a single member crate finance-service. In finance-service/Cargo.toml, add all dependencies with pinned versions from the task details. Create src/main.rs with a minimal tokio::main that binds Axum to :3000 and returns 200 on GET /. Add a .cargo/config.toml with incremental = true for faster builds. Run `cargo build` to verify the dependency graph resolves. Add sqlx-cli to dev-dependencies or document the migration command using DATABASE_URL.
</implementation_plan>

<validation>
`cargo build` exits 0 with no compilation errors. `cargo check` passes. `cargo test` runs zero tests without panic.
</validation>