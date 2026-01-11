# Web App Development Guide

This guide covers local development setup for the Next.js web application, including database access and configuration.

## Overview

The web app is a Next.js 16 application with:
- **Database**: PostgreSQL (CloudNative-PG in Kubernetes)
- **Auth**: Better Auth with GitHub OAuth
- **ORM**: Drizzle ORM
- **Styling**: Tailwind CSS 4

## Local Development Setup

### Prerequisites

- Node.js 20+
- kubectl configured to access the cluster
- Access to the `databases` namespace

### Starting the Development Server

```bash
cd apps/web
npm install
npm run dev
```

The app will be available at `http://localhost:3000`

### Environment Variables

Create `apps/web/.env.local` with the following variables:

```bash
# Database URL (optional for basic pages, required for auth/features)
# See "Database Connection" section below
DATABASE_URL="postgresql://web_user:password@localhost:5432/web_app"

# Better Auth secret (generate with: openssl rand -base64 32)
BETTER_AUTH_SECRET="your-secret-here"

# GitHub OAuth credentials (optional for auth features)
# Create OAuth app at: https://github.com/settings/developers
# Callback URL: http://localhost:3000/api/auth/callback/github
GITHUB_CLIENT_ID="your-client-id"
GITHUB_CLIENT_SECRET="your-client-secret"

# Public app URL (defaults to localhost:3000 in dev)
NEXT_PUBLIC_APP_URL="http://localhost:3000"

# Disable telemetry
NEXT_TELEMETRY_DISABLED=1
```

## Database Connection

### Using Cluster Database (Recommended)

Connect to the PostgreSQL database in the Kubernetes cluster via port-forward:

#### Option 1: Using the Helper Script

```bash
./scripts/port-forward-web-db.sh
```

This script will:
- Port-forward the `web-postgres-rw` service to `localhost:5432`
- Display connection details and connection string
- Handle port conflicts automatically

#### Option 2: Manual Port-Forward

```bash
# Port-forward PostgreSQL service
kubectl port-forward svc/web-postgres-rw -n databases 5432:5432

# In another terminal, get connection details
kubectl get secret web-postgres-app -n databases -o jsonpath='{.data.uri}' | base64 -d
```

#### Connection String Format

```
postgresql://web_user:<password>@localhost:5432/web_app
```

To get the password:
```bash
kubectl get secret web-postgres-app -n databases -o jsonpath='{.data.password}' | base64 -d
```

### Database Status

Check if the PostgreSQL cluster is ready:

```bash
# Check cluster status
kubectl get cluster web-postgres -n databases

# Check pods
kubectl get pods -n databases -l app.kubernetes.io/name=web-postgres

# Check service
kubectl get svc web-postgres-rw -n databases

# View cluster logs
kubectl logs -n databases -l app.kubernetes.io/name=web-postgres --tail=50
```

### Database Migrations

The web app uses Drizzle ORM for database migrations.

#### Running Migrations Locally

```bash
cd apps/web

# Generate migration files (after schema changes)
npm run db:generate

# Apply migrations (requires DATABASE_URL)
npm run db:migrate

# Or use Drizzle Kit directly
npx drizzle-kit push
```

#### Running Migrations in Kubernetes

Migrations should be run as an init container or job before the app starts. This is a TODO for future implementation.

## Authentication Setup

### GitHub OAuth

1. Create a GitHub OAuth App:
   - Go to https://github.com/settings/developers
   - Click "New OAuth App"
   - **Application name**: `CTO Web App (Local)`
   - **Homepage URL**: `http://localhost:3000`
   - **Authorization callback URL**: `http://localhost:3000/api/auth/callback/github`
   - Click "Register application"

2. Copy the Client ID and generate a Client Secret

3. Add to `.env.local`:
   ```bash
   GITHUB_CLIENT_ID="your-client-id"
   GITHUB_CLIENT_SECRET="your-client-secret"
   ```

### Better Auth Secret

Generate a secure random secret:

```bash
openssl rand -base64 32
```

Add to `.env.local`:
```bash
BETTER_AUTH_SECRET="generated-secret-here"
```

## Development Workflow

### Making Changes

1. Start the dev server: `npm run dev`
2. Make code changes - Next.js will hot reload
3. Test locally at `http://localhost:3000`

### Testing Database Changes

1. Update Drizzle schema in `apps/web/src/lib/db/schema.ts`
2. Generate migration: `npm run db:generate`
3. Apply migration: `npm run db:migrate`
4. Test the changes

### Building for Production

```bash
cd apps/web
npm run build
npm run start  # Test production build locally
```

## Troubleshooting

### Database Connection Issues

**Error**: `Connection refused` or `timeout`

**Solutions**:
- Verify port-forward is running: `kubectl port-forward svc/web-postgres-rw -n databases 5432:5432`
- Check cluster is ready: `kubectl get cluster web-postgres -n databases`
- Verify secret exists: `kubectl get secret web-postgres-app -n databases`

**Error**: `password authentication failed`

**Solutions**:
- Verify you're using the correct password from the secret
- Check the secret is up-to-date: `kubectl get secret web-postgres-app -n databases -o yaml`

### Next.js Build Issues

**Error**: Missing environment variables during build

**Solutions**:
- Ensure all required env vars are set in `.env.local`
- For CI builds, dummy values are provided (see `.github/workflows/web-ci.yaml`)

### Port Already in Use

If port 5432 is already in use:

```bash
# Find process using port 5432
lsof -ti:5432

# Kill the process
kill -9 $(lsof -ti:5432)

# Or use a different local port
LOCAL_PORT=5433 ./scripts/port-forward-web-db.sh
```

Then update your `DATABASE_URL` to use port 5433.

## Kubernetes Deployment

The web app is deployed to Kubernetes via ArgoCD. See the main deployment documentation for details.

### Database Secret in Kubernetes

The web app deployment references the `web-database-url` secret in the `cto` namespace. This secret is synced from OpenBao via External Secrets Operator.

### Populating OpenBao with Database URL

After the PostgreSQL cluster is created, populate OpenBao with the connection string:

```bash
# Get the connection URI from CNPG secret
DB_URI=$(kubectl get secret web-postgres-app -n databases -o jsonpath='{.data.uri}' | base64 -d)

# Get OpenBao root token (from 1Password)
ROOT_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --format=json | \
  jq -r '.fields[] | select(.label == "password" or .label == "Root Token") | .value')

# Store in OpenBao
kubectl exec -n openbao openbao-0 -- env BAO_TOKEN="$ROOT_TOKEN" \
  bao kv put secret/web-app/database-url url="$DB_URI"
```

The ExternalSecret will automatically sync this to the `web-database-url` secret in the `cto` namespace within the refresh interval (1 hour).

## CI/CD

The GitHub Actions workflow (`.github/workflows/web-ci.yaml`) handles:
- TypeScript linting
- Next.js build verification
- Helm chart validation
- Docker image building and pushing to GHCR

**Note**: CI uses dummy/mock environment variables for build checks - no actual database is needed for CI builds.

## Additional Resources

- [Next.js Documentation](https://nextjs.org/docs)
- [Drizzle ORM Documentation](https://orm.drizzle.team/)
- [Better Auth Documentation](https://www.better-auth.com/docs)
- [CloudNative-PG Documentation](https://cloudnative-pg.io/documentation/)
