# Build stage
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY src/ ./src/

# Build TypeScript
RUN npm run build

# Production stage
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}node:20-alpine

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install production dependencies only
RUN npm ci --only=production

# Copy built files from builder
COPY --from=builder /app/dist ./dist

# Run as non-root user
USER node

# Start the bridge
CMD ["node", "dist/index.js"]
