# OpenMemory Server Container
# Based on: https://openmemory.cavira.app/docs/installation
FROM node:20-alpine AS builder

# Install build dependencies
RUN apk add --no-cache git python3 make g++ sqlite

WORKDIR /build

# Clone OpenMemory repository
RUN git clone --depth 1 https://github.com/caviraoss/openmemory.git .

# Build from backend directory (per installation docs)
WORKDIR /build/backend
RUN npm install
RUN npm run build

# Production image
FROM node:20-alpine

# Install runtime dependencies
RUN apk add --no-cache sqlite

# Create non-root user (use 1001 since node:alpine uses 1000)
RUN addgroup -g 1001 -S openmemory && \
    adduser -u 1001 -S openmemory -G openmemory

WORKDIR /app

# Copy built application from backend directory
COPY --from=builder --chown=openmemory:openmemory /build/backend/dist ./dist
COPY --from=builder --chown=openmemory:openmemory /build/backend/node_modules ./node_modules
COPY --from=builder --chown=openmemory:openmemory /build/backend/package*.json ./

# Create data directory
RUN mkdir -p /data && chown -R openmemory:openmemory /data

# Switch to non-root user
USER openmemory

# Environment defaults
ENV NODE_ENV=production \
    PORT=3000 \
    DATABASE_PATH=/data/openmemory.db \
    EMBEDDING_PROVIDER=local \
    EMBEDDING_MODEL=Xenova/all-MiniLM-L6-v2

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3000/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"

# Expose ports
EXPOSE 3000 9090

# Start server
CMD ["node", "dist/index.js"]
