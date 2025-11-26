# Doc Server Configuration Examples

This directory contains example configuration files for setting up doc server tasks with proper secrets and environment variables.



## Files

### `example-requirements.yaml`
Template for the `requirements.yaml` file that can be placed at either:
- **Project-level** (recommended): `{docs_project_directory}/requirements.yaml` - **shared by all tasks**
- **Task-specific**: `{docs_project_directory}/task-{TASK_ID}/requirements.yaml` - **overrides project-level**

This file configures:
- **Secrets**: References the `doc-server-secrets` Kubernetes secret
- **Environment Variables**: Non-secret configuration for the doc server

### `doc-server-example.env`
Example `.env` file for **local development** containing:


- Database connection strings (both main DB and vector DB)


- OpenAI API key


- All configuration variables



## Usage

### For Production (Kubernetes Tasks)

**Option 1: Project-level (Recommended)**


1. Copy `example-requirements.yaml` to your docs project root as `requirements.yaml`


2. **All tasks** in the project will automatically use this configuration

**Option 2: Task-specific (For special cases)**


1. Copy `example-requirements.yaml` to specific task directories as `requirements.yaml`


2. Task-specific files override the project-level configuration

**Priority Order:**


1. `{docs_project_directory}/task-{TASK_ID}/requirements.yaml` (highest priority)


2. `{docs_project_directory}/requirements.yaml` (fallback)


3. No requirements (empty configuration)

### For Local Development


1. Copy `doc-server-example.env` to your project root as `.env`


2. Update any values as needed for your local setup

## Secret Management

The secrets are managed via External Secrets Operator:
- **Source**: Central `secret-store` namespace
- **Target**: `cto` namespace (where code jobs run)
- **Configuration**: `infra/secret-store/doc-server-external-secrets.yaml`

The following secrets are automatically available in your code jobs:


- `OPENAI_API_KEY` - OpenAI API key for embeddings


- `DATABASE_URL` - Main application database (app_db)


- `VECTOR_DATABASE_URL` - Vector database with pgvector (vector_db)

## Database Information

- **PostgreSQL Cluster**: `vector-postgres.databases.svc.cluster.local:5432`
- **Main Database**: `app_db` (for general application data)
- **Vector Database**: `vector_db` (pgvector extension enabled for embeddings)
- **Redis Cache**: `redis-auth-service.databases.svc.cluster.local:6379`
