# Tweakcn - Visual Theme Editor for shadcn/ui
# Built from upstream: https://github.com/jnsahaj/tweakcn
# Build trigger: HEAL-2485 - Initial GHCR image push for deployment

# ===== DOWNLOAD STAGE =====
# Download the repository archive using ADD instruction
# ADD with URLs uses the buildkit daemon's DNS config (from buildkitd.toml)
# which is properly configured, unlike RUN instruction containers
FROM alpine:3.21 AS downloader

WORKDIR /src

# Install unzip for extracting the archive
RUN apk add --no-cache unzip

# Download and extract the tweakcn repository from GitHub archive
# Using ADD with URL instead of git clone avoids DNS issues in RUN containers
# Note: Using cp -a with trailing dot to include hidden dotfiles (e.g., .npmrc, .nvmrc)
# The shell glob * does not match dotfiles by default, so we use cp -a instead of mv
ADD https://github.com/jnsahaj/tweakcn/archive/refs/heads/main.zip /tmp/repo.zip
RUN unzip /tmp/repo.zip -d /tmp && \
    cp -a /tmp/tweakcn-main/. /src/ && \
    rm -rf /tmp/repo.zip /tmp/tweakcn-main

FROM node:20-alpine AS base

# Install pnpm globally
RUN corepack enable && corepack prepare pnpm@latest --activate

# Install dependencies only when needed
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app

# Copy the downloaded repository from the downloader stage
COPY --from=downloader /src ./

# Install dependencies with pnpm (matches upstream package manager)
RUN pnpm install --frozen-lockfile || pnpm install

# Rebuild the source code only when needed
FROM base AS builder
WORKDIR /app

COPY --from=deps /app ./

# Patch next.config.ts to add standalone output for Docker deployment
RUN sed -i 's/const nextConfig: NextConfig = {/const nextConfig: NextConfig = {\n  output: "standalone",/' next.config.ts

# Disable telemetry during build
ENV NEXT_TELEMETRY_DISABLED=1

# Provide dummy environment variables for build
# These are required by the app during Next.js page collection but don't need real values
# At runtime, they get replaced by actual env vars
ENV DATABASE_URL="postgresql://build:build@localhost:5432/build?sslmode=disable"
ENV BETTER_AUTH_SECRET="build-time-dummy-secret-not-used-at-runtime"
ENV POLAR_WEBHOOK_SECRET="build-time-dummy-secret-not-used-at-runtime"

# Skip prebuild and run next build directly
RUN pnpm exec next build

# Production image
FROM base AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

# Create non-root user
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# Copy built assets
COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

CMD ["node", "server.js"]
