# Docker Runtime Benchmark Task

Run a benchmark comparing Docker Desktop, OrbStack, and Colima for Rust Docker builds. Save results to `~/Desktop/benchmark-results.md`.

## Setup

1. The repo is already cloned at `/tmp/cto-benchmark` (develop branch)
2. Docker config has been fixed at `~/.docker/config.json` (no credsStore)
3. All three runtimes are installed: Docker Desktop, OrbStack, Colima

## Benchmark Instructions

For each runtime (Docker Desktop, OrbStack, Colima):

### 1. Start the runtime
- **Docker Desktop**: `open -a Docker` then wait for `docker info` to work
- **OrbStack**: `open -a OrbStack` then wait for `docker info` to work  
- **Colima**: `colima start --cpu 8 --memory 12 --disk 60 --vm-type vz --mount-type virtiofs`

### 2. Record system info
```bash
docker info | grep -E "Server Version|CPUs|Total Memory"
```

### 3. Clean cache for cold build
```bash
docker builder prune -af
```

### 4. Run COLD build (timed)
```bash
cd /tmp/cto-benchmark
time docker build --platform linux/amd64 -t benchmark-pm:cold -f infra/images/pm-server/Dockerfile.build .
```

### 5. Run WARM build (timed) - immediately after cold build
```bash
time docker build --platform linux/amd64 -t benchmark-pm:warm -f infra/images/pm-server/Dockerfile.build .
```

### 6. Stop the runtime before testing next one
- **Docker Desktop**: `osascript -e 'quit app "Docker"'`
- **OrbStack**: `osascript -e 'quit app "OrbStack"'`
- **Colima**: `colima stop`

Wait 5 seconds between stopping one and starting the next.

## Output Format

Create `~/Desktop/benchmark-results.md` with this format:

```markdown
# Docker Runtime Benchmark Results

**Date**: [date]
**Machine**: MacBook Pro M1 Pro, 10 cores, 16GB RAM
**Test Image**: pm-server (Rust multi-stage build)

## Results Summary

| Runtime | Cold Build | Warm Build | Memory Allocated |
|---------|-----------|------------|------------------|
| Docker Desktop | Xs | Xs | X GB |
| OrbStack | Xs | Xs | X GB |
| Colima | Xs | Xs | X GB |

## Detailed Results

### Docker Desktop
- Server Version: X
- CPUs: X
- Total Memory: X
- Cold build time: X
- Warm build time: X

### OrbStack
- Server Version: X
- CPUs: X
- Total Memory: X
- Cold build time: X
- Warm build time: X

### Colima
- Server Version: X
- CPUs: X
- Total Memory: X
- Cold build time: X
- Warm build time: X

## Winner

[Which runtime was fastest and by how much]
```

## Notes

- If a build fails, note the error and move on
- The cold build will take 5-15 minutes each (Rust compilation is slow)
- Warm builds should be much faster (cached layers)
- Make sure to stop each runtime completely before starting the next



