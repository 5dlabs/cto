# OpenMemory Server Container
FROM node:20-alpine AS builder

# Install build dependencies
RUN apk add --no-cache git python3 make g++ sqlite

WORKDIR /app

# Clone OpenMemory repository
RUN git clone https://github.com/caviraoss/openmemory.git .

# Install dependencies and build
RUN npm ci --production=false
RUN npm run build

# Production image
FROM node:20-alpine

# Install runtime dependencies
RUN apk add --no-cache sqlite

# Create non-root user
RUN addgroup -g 1000 -S openmemory && \
    adduser -u 1000 -S openmemory -G openmemory

WORKDIR /app

# Copy built application
COPY --from=builder --chown=openmemory:openmemory /app/dist ./dist
COPY --from=builder --chown=openmemory:openmemory /app/node_modules ./node_modules
COPY --from=builder --chown=openmemory:openmemory /app/package*.json ./

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
