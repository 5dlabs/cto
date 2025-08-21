# Task 1 Prompt — Initialize Rust Project

Goal: Create a new Rust HTTP API project with proper structure and dependencies.

Do exactly the following:
1. **Initialize Rust Project**:
   - Run `cargo init --name rust-api-server` to create the project
   - Verify the project structure is created correctly

2. **Configure Dependencies**:
   - Update `Cargo.toml` with required dependencies:
     - `tokio = { version = "1.0", features = ["full"] }`
     - `axum = "0.7"`
     - `serde = { version = "1.0", features = ["derive"] }`
     - `serde_json = "1.0"`
     - `tracing = "0.1"`
     - `tracing-subscriber = "0.3"`

3. **Create Basic Structure**:
   - Set up `src/main.rs` with an empty async main function
   - Add proper imports for the HTTP server framework
   - Include basic error handling setup
   - Create `.gitignore` with standard Rust entries:
     - `/target/`
     - `Cargo.lock`
     - `*.pdb`
     - `.env`
     - `.DS_Store`

4. **Documentation**:
   - Create `README.md` explaining the project purpose
   - Document how to build and run the project

5. **Verify Setup**:
   - Run `cargo check` to ensure project compiles
   - Fix any compilation issues

6. **Create PR**:
   - Commit changes with message: `feat(task-1): initialize Rust HTTP API project`
   - Open PR with title: `Task 1: Initialize Rust project structure`
   - Add label `task-1` to the PR

