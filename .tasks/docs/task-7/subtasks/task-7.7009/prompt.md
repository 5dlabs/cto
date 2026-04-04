Implement subtask 7009: Containerize Next.js app with multi-stage Dockerfile

## Objective
Create a multi-stage Dockerfile that builds the Next.js application and serves it with a minimal Node.js runtime. Configure the container to read `PM_SERVER_URL` from an environment variable and expose port 3000.

## Steps
1. Create `Dockerfile` in the project root:
   ```
   # Stage 1: Dependencies
   FROM node:20-alpine AS deps
   WORKDIR /app
   COPY package.json package-lock.json ./
   RUN npm ci --only=production

   # Stage 2: Build
   FROM node:20-alpine AS builder
   WORKDIR /app
   COPY --from=deps /app/node_modules ./node_modules
   COPY . .
   RUN npm run build

   # Stage 3: Runner
   FROM node:20-alpine AS runner
   WORKDIR /app
   ENV NODE_ENV=production
   COPY --from=builder /app/.next/standalone ./
   COPY --from=builder /app/.next/static ./.next/static
   COPY --from=builder /app/public ./public
   EXPOSE 3000
   CMD ["node", "server.js"]
   ```
2. Update `next.config.js` to set `output: 'standalone'` for optimized Docker builds.
3. Create `.dockerignore` excluding `node_modules`, `.next`, `.git`, `.env.local`.
4. Ensure `PM_SERVER_URL` is read at runtime (not baked at build time) — use `NEXT_PUBLIC_` prefix only for client-side needs, or use Next.js server-side env vars.
5. Create `docker-compose.yml` for local testing with `PM_SERVER_URL` set via environment.
6. Verify the built image size is reasonable (< 200MB).

## Validation
`docker build -t delegation-dashboard .` succeeds without errors. `docker run -p 3000:3000 -e PM_SERVER_URL=http://host.docker.internal:8080 delegation-dashboard` starts the app and serves it on port 3000. The running container responds to HTTP requests on port 3000. Image size is under 200MB.