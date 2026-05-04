# Build stage
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./
COPY tsconfig.json ./
RUN npm ci

COPY src/ ./src/
RUN npm test && npm run build

# Production stage
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}node:20-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production
COPY --from=builder /app/dist ./dist

USER node

EXPOSE 4001

CMD ["node", "dist/index.js"]
