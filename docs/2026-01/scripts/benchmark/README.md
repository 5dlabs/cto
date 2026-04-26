# Docker Runtime Benchmark

Benchmark comparison of Docker Desktop, OrbStack, and Colima for Rust Docker builds.

## Quick Start

From your main Mac:

```bash
./scripts/benchmark/run-remote-benchmark.sh
```

This will:
1. Connect to the remote Mac (192.168.1.90)
2. Install OrbStack and Colima if needed
3. Run the benchmark with all three runtimes
4. Copy results back to `scripts/benchmark/results/`

## Prerequisites

### On Remote Mac

1. **Wake the Mac** - ensure it's not sleeping
2. **Enable Remote Login**:
   - System Preferences > Sharing > Remote Login
   - Allow access for your user
3. **SSH key access** - ensure your SSH key is authorized

### Manual Setup (if needed)

If the wrapper script doesn't work, SSH into the remote Mac and run:

```bash
# On remote Mac
/tmp/setup-remote-runtimes.sh
/tmp/benchmark-docker-runtimes.sh
```

## What's Tested

| Runtime | Description |
|---------|-------------|
| Docker Desktop | Apple's official Docker for Mac |
| OrbStack | Lightweight alternative, claims 2x faster I/O |
| Colima | Open-source, runs on Lima VMs |

## Build Types

| Build Type | Description |
|------------|-------------|
| **Cold** | No cache, full rebuild |
| **Warm** | With cargo registry cache |
| **Incremental** | Small code change rebuild |

## Colima Optimization

The benchmark configures Colima with optimal settings for Apple Silicon:

```bash
colima start \
  --cpu 8 \
  --memory 12 \
  --disk 60 \
  --vm-type vz \           # Apple Virtualization framework
  --vz-rosetta \           # Rosetta for x86_64 emulation
  --mount-type virtiofs    # Fast file sharing
```

## Results

Results are saved to:
- Remote: `/tmp/benchmark-results-YYYYMMDD-HHMMSS.txt`
- Local: `scripts/benchmark/results/`

Format: CSV with columns `runtime,build_type,duration_seconds`

## Expected Results

Based on community benchmarks:

| Runtime | Cold Build | File I/O | Memory |
|---------|-----------|----------|--------|
| Docker Desktop | Baseline | Baseline | ~4GB overhead |
| OrbStack | ~Same | ~2x faster | ~1GB overhead |
| Colima | ~Same | ~1.5x faster | Configurable |

Actual results will vary based on your specific workload.





