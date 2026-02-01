# syntax=docker/dockerfile:1
# =============================================================================
# Web App (Next.js) - Unified Dockerfile
# =============================================================================
#
# Multi-stage build for Next.js application:
#   1. Dependencies stage - install npm dependencies
#   2. Build stage - build Next.js application
#   3. Production stage - runtime with minimal dependencies
#
# Build commands:
#   docker build --target production -f infra/images/web/Dockerfile -t ghcr.io/5dlabs/web:latest .
# =============================================================================

# =============================================================================
# Dependencies stage - install npm packages
# =============================================================================
FROM node:25-bookworm-slim AS deps

WORKDIR /app

# Copy package files
COPY apps/web/package.json apps/web/package-lock.json* ./

# Install dependencies
RUN npm ci --legacy-peer-deps

# =============================================================================
# Build stage - compile Next.js application
# =============================================================================
FROM node:25-bookworm-slim AS builder

WORKDIR /app

# Build arguments for Next.js public env vars (inlined at build time)
ARG NEXT_PUBLIC_APP_URL=https://app.5dlabs.ai

# Copy source code
COPY apps/web ./apps/web

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./apps/web/node_modules

# Set working directory to app
WORKDIR /app/apps/web

# Set environment variables for build
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV NEXT_PUBLIC_APP_URL=${NEXT_PUBLIC_APP_URL}

# Build Next.js application (creates .next/standalone in apps/web)
RUN npm run build

# =============================================================================
# Production stage - minimal runtime
# =============================================================================
FROM node:25-bookworm-slim AS production

WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000
ENV HOSTNAME=0.0.0.0

# Install runtime dependencies for Next.js standalone
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    wget \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -u 1001 -m -d /app -s /bin/bash nextjs

# Copy standalone output from builder
# Next.js standalone creates .next/standalone structure with server.js at the root
# The standalone output includes the minimal server files
COPY --from=builder --chown=nextjs:nextjs /app/apps/web/.next/standalone ./
COPY --from=builder --chown=nextjs:nextjs /app/apps/web/.next/static ./.next/static
COPY --from=builder --chown=nextjs:nextjs /app/apps/web/public ./public

# Switch to non-root user
USER nextjs

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1

# Next.js standalone creates server.js in the standalone root
CMD ["node", "server.js"]

# =============================================================================
# Migration stage - for database schema migrations
# =============================================================================
# Build with: docker build --target migrate -f infra/images/web/Dockerfile -t ghcr.io/5dlabs/web-migrate:latest .
# This is a separate image used as an init container to run database migrations
FROM node:25-bookworm-slim AS migrate

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy package files and source for migration
COPY apps/web/package.json apps/web/package-lock.json* ./
COPY apps/web/drizzle.config.ts ./
COPY apps/web/src/lib/db ./src/lib/db
COPY apps/web/tsconfig.json ./

# Install all dependencies (including devDependencies for drizzle-kit)
RUN npm ci --legacy-peer-deps

# Create non-root user
RUN useradd -r -u 1001 -m -d /app -s /bin/bash migrate
USER migrate

# Run migration script
CMD ["npx", "drizzle-kit", "push", "--force"]
