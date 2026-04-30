# Build stage
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}node:24-alpine AS builder

WORKDIR /app

# Copy package files
COPY apps/discord-bridge/package*.json ./apps/discord-bridge/
COPY apps/discord-bridge/tsconfig.json ./apps/discord-bridge/
COPY shared/ ./shared/
WORKDIR /app/apps/discord-bridge

# Install dependencies
RUN npm ci

# Copy source code
COPY apps/discord-bridge/src/ ./src/

# Build TypeScript
RUN npm run build

# Production stage
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}node:24-alpine

WORKDIR /app

# Copy package files
COPY apps/discord-bridge/package*.json ./apps/discord-bridge/
COPY shared/ ./shared/
WORKDIR /app/apps/discord-bridge

# Install production dependencies only
RUN npm ci --only=production

# Copy built files from builder
COPY --from=builder /app/apps/discord-bridge/dist ./dist

ENV WORKSPACE=/tmp/discord-bridge
RUN mkdir -p /tmp/discord-bridge/.intake && chown -R node:node /tmp/discord-bridge

# Run as non-root user
USER node

# Start the bridge
CMD ["node", "dist/index.js"]
