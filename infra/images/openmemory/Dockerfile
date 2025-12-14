# OpenMemory Server Container
# Based on official Dockerfile from https://github.com/caviraoss/openmemory
FROM node:20-alpine AS builder

WORKDIR /app

# Install system dependencies required for native module compilation,
# plus curl and bash (needed to run the Bun installation script)
RUN apk add --no-cache python3 make g++ curl bash git

# Install Bun globally and relocate it to a shared, persistent location
RUN curl -fsSL https://bun.sh/install | bash \
 && mv /root/.bun /opt/bun \
 && ln -s /opt/bun/bin/bun /usr/local/bin/bun

# Ensure Bun is available in PATH for subsequent commands
ENV PATH="/opt/bun/bin:${PATH}"

# Clone OpenMemory repository
RUN git clone --depth 1 https://github.com/caviraoss/openmemory.git /src

WORKDIR /src/backend

# Install all dependencies (including devDependencies) using Bun
RUN bun install --concurrent-scripts=10 --network-concurrency=96

# Build the application
RUN bun run build

# Remove devDependencies to reduce image size
RUN npm prune --omit=dev


# ===== PRODUCTION STAGE =====
FROM node:20-alpine AS production

WORKDIR /app

# Create a dedicated non-root user for security (use 1001 to avoid conflicts)
RUN addgroup -g 1001 -S openmemory \
 && adduser -u 1001 -S openmemory -G openmemory

# Copy only production artifacts from the builder stage
COPY --from=builder /src/backend/node_modules ./node_modules/
COPY --from=builder /src/backend/dist ./dist/
COPY --from=builder /src/backend/package.json ./

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
