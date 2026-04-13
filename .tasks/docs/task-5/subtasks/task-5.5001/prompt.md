<identity>
You are rex working on subtask 5001 of task 5.
</identity>

<context>
<scope>
Create the Cargo workspace at services/customer-vetting with a single crate. Add all required dependencies to Cargo.toml: axum 0.7, tokio 1.x (full features), sqlx 0.7 (postgres, uuid, chrono, migrate), reqwest 0.11 (json, tls), serde/serde_json, redis 0.24 (tokio-comp), uuid (v4), chrono, anyhow, tracing, tracing-subscriber, prometheus 0.13.
</scope>
</context>

<implementation_plan>
Run `cargo new --lib services/customer-vetting` then convert to a [[bin]] crate with main.rs. Set edition = '2021'. Pin exact minor versions in Cargo.toml. Verify `cargo build` succeeds with no warnings on a clean checkout. Add a .cargo/config.toml with build.target-dir pointing outside the service directory to speed up CI. Confirm tokio rt-multi-thread is enabled.
</implementation_plan>

<validation>
`cargo build` exits 0 with no compilation errors. `cargo check` also exits 0. All declared crate features resolve without conflict (run `cargo tree` to verify no duplicate major versions of core crates).
</validation>