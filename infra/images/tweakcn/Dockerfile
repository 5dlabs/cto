# Tweakcn - Visual Theme Editor for shadcn/ui
# Built from upstream: https://github.com/jnsahaj/tweakcn

FROM node:20-alpine AS base

# Install pnpm globally
RUN corepack enable && corepack prepare pnpm@latest --activate

# Install dependencies only when needed
FROM base AS deps
RUN apk add --no-cache libc6-compat git
WORKDIR /app

# Clone tweakcn repository
RUN git clone --depth 1 https://github.com/jnsahaj/tweakcn.git .

# Install dependencies with pnpm (matches upstream package manager)
RUN pnpm install --frozen-lockfile || pnpm install

# Rebuild the source code only when needed
FROM base AS builder
WORKDIR /app

COPY --from=deps /app ./

# Patch next.config.ts to add standalone output for Docker deployment
RUN sed -i 's/const nextConfig: NextConfig = {/const nextConfig: NextConfig = {\n  output: "standalone",/' next.config.ts

# Build arguments for environment
ARG NEXT_PUBLIC_APP_URL=http://localhost:3000
ENV NEXT_PUBLIC_APP_URL=${NEXT_PUBLIC_APP_URL}

# Disable telemetry during build
ENV NEXT_TELEMETRY_DISABLED=1

# Provide dummy env vars for build (the app works without DB for basic editing)
ENV DATABASE_URL=""
ENV BETTER_AUTH_SECRET="not-needed-for-basic-usage"
ENV BASE_URL="http://localhost:3000"

# Build the application
RUN pnpm run build

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
