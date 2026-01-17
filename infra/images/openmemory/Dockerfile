# OpenMemory Server Container
# Based on official Dockerfile from https://github.com/caviraoss/openmemory
# Server now lives in packages/openmemory-js/

# ===== DOWNLOAD STAGE =====
# Download the repository archive using ADD instruction
# ADD with URLs uses the buildkit daemon's DNS config (from buildkitd.toml)
# which is properly configured, unlike RUN instruction containers
FROM alpine:3.21 AS downloader

WORKDIR /src

# Install unzip for extracting the archive
RUN apk add --no-cache unzip

# Download and extract the OpenMemory repository from GitHub archive
# Using ADD with URL instead of git clone avoids DNS issues in RUN containers
ADD https://github.com/caviraoss/openmemory/archive/refs/heads/main.zip /tmp/repo.zip
RUN unzip /tmp/repo.zip -d /tmp && \
    mv /tmp/openmemory-main/* /src/ && \
    rm -rf /tmp/repo.zip /tmp/openmemory-main

# ===== BUILD STAGE =====
# Use official Bun image which has Bun pre-installed (avoids DNS issues from curl install)
FROM oven/bun:1-alpine AS builder

WORKDIR /app

# Install system dependencies required for native module compilation
RUN apk add --no-cache python3 make g++ nodejs npm

# Copy the downloaded repository from the downloader stage
COPY --from=downloader /src/packages/openmemory-js ./

# Install all dependencies (including devDependencies) using Bun
RUN bun install --concurrent-scripts=10 --network-concurrency=96

# Build the application
RUN bun run build

# Remove devDependencies to reduce image size
RUN npm prune --omit=dev


# ===== PRODUCTION STAGE =====
FROM node:20-alpine AS production

WORKDIR /app

# Install timezone data for proper TZ support
RUN apk add --no-cache tzdata

# Create a dedicated non-root user for security (use 1001 to avoid conflicts)
RUN addgroup -g 1001 -S openmemory \
 && adduser -u 1001 -S openmemory -G openmemory

# Copy only production artifacts from the builder stage
COPY --from=builder /app/node_modules ./node_modules/
COPY --from=builder /app/dist ./dist/
COPY --from=builder /app/package.json ./

# Create a directory for persistent data and secure file permissions
RUN mkdir -p /data \
 && chown -R openmemory:openmemory /data /app \
 && chmod -R go-w /app

# Switch to non-root user
USER openmemory

# Environment defaults
ENV NODE_ENV=production \
    PORT=8080 \
    DATABASE_PATH=/data/openmemory.db

# Expose the application port
EXPOSE 8080

# Define a lightweight health check that verifies the /health endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD node -e "require('http').get('http://localhost:8080/health', (res) => process.exit(res.statusCode === 200 ? 0 : 1)).on('error', () => process.exit(1))"

# Start the application using npm
ENTRYPOINT ["npm", "start"]
