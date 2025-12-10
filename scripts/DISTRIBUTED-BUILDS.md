# Distributed Docker Builds via Thunderbolt Bridge

This guide explains how to use a second MacBook as a remote Docker build machine, connected via Thunderbolt for maximum speed (~40 Gbps).

## Prerequisites

- Two MacBooks with Thunderbolt/USB-C ports
- A Thunderbolt 3/4 cable (40 Gbps capable)
- Docker Desktop installed on both machines

## Step 1: Connect the Cable

1. Connect the Thunderbolt cable between the two MacBooks
2. macOS will automatically create a "Thunderbolt Bridge" network interface

## Step 2: Configure Network on Both Machines

### Option A: Automatic (Link-Local)

macOS often auto-assigns link-local addresses. Check if you can already ping:

```bash
# On your main machine, find the remote Mac
ping <other-mac-hostname>.local
```

### Option B: Manual Static IPs (More Reliable)

**On your main MacBook (the one running Tilt):**

```bash
# Find the Thunderbolt Bridge interface
networksetup -listallhardwareports | grep -A2 "Thunderbolt Bridge"

# Assign a static IP (typically en6 or similar)
sudo ifconfig bridge0 192.168.100.1 netmask 255.255.255.0 up

# Or via System Settings:
# System Settings → Network → Thunderbolt Bridge → Configure IPv4: Manually
# IP: 192.168.100.1, Subnet: 255.255.255.0
```

**On the remote MacBook (build machine):**

```bash
sudo ifconfig bridge0 192.168.100.2 netmask 255.255.255.0 up

# Or via System Settings:
# IP: 192.168.100.2, Subnet: 255.255.255.0
```

**Verify connectivity:**

```bash
# From main machine
ping 192.168.100.2

# From remote machine  
ping 192.168.100.1
```

## Step 3: Expose Docker Daemon on Remote Machine

**On the remote MacBook**, configure Docker Desktop to accept remote connections:

### Option A: SSH Tunnel (Secure, Recommended)

No Docker config changes needed. Use SSH to tunnel:

```bash
# On main machine - create SSH tunnel to remote Docker
ssh -fNL /tmp/docker-remote.sock:/var/run/docker.sock user@192.168.100.2

# Then use it
export DOCKER_HOST=unix:///tmp/docker-remote.sock
docker ps  # This runs on the remote machine!
```

### Option B: TCP (Faster, Less Secure - Only on Trusted Network)

**On the remote MacBook:**

1. Open Docker Desktop → Settings → Docker Engine
2. Add to the JSON config:

```json
{
  "hosts": ["unix:///var/run/docker.sock", "tcp://0.0.0.0:2375"]
}
```

3. Click "Apply & Restart"

**On main MacBook:**

```bash
export DOCKER_HOST=tcp://192.168.100.2:2375
docker ps  # This runs on the remote machine!
```

## Step 4: Test Remote Docker Builds

```bash
# Set Docker to use remote daemon
export DOCKER_HOST=tcp://192.168.100.2:2375

# Test a build
cd /path/to/cto
docker build -t test -f infra/images/pm-server/Dockerfile.build .
```

> **Note:** The build context (your code) is sent over Thunderbolt to the remote machine.
> At 40 Gbps, even 100MB of code transfers in ~20ms.

## Step 5: Integration with Tilt

### Option A: All Builds Remote

Set the environment before running Tilt:

```bash
export DOCKER_HOST=tcp://192.168.100.2:2375
tilt up
```

### Option B: Hybrid Script (Some Local, Some Remote)

Create a script for heavy builds:

```bash
#!/bin/bash
# scripts/remote-build.sh

SERVICE=$1
REMOTE_DOCKER="tcp://192.168.100.2:2375"
LOCAL_REGISTRY="192.168.1.72:30500"

case $SERVICE in
  controller)
    DOCKERFILE="infra/images/controller/Dockerfile.kind"
    ;;
  tools)
    DOCKERFILE="infra/images/tools/Dockerfile.kind"
    ;;
  pm)
    DOCKERFILE="infra/images/pm-server/Dockerfile.build"
    ;;
  *)
    echo "Usage: $0 <controller|tools|pm>"
    exit 1
    ;;
esac

echo "🔨 Building $SERVICE on remote machine..."
DOCKER_HOST=$REMOTE_DOCKER docker build \
  --platform linux/amd64 \
  -t $LOCAL_REGISTRY/$SERVICE:tilt-dev \
  -f $DOCKERFILE \
  .

echo "📤 Pushing $SERVICE..."
DOCKER_HOST=$REMOTE_DOCKER docker push $LOCAL_REGISTRY/$SERVICE:tilt-dev

echo "✅ $SERVICE complete"
```

Usage:

```bash
./scripts/remote-build.sh controller
./scripts/remote-build.sh tools
```

## Step 6: Parallel Builds on Both Machines

For maximum speed, build different services on different machines simultaneously:

```bash
#!/bin/bash
# scripts/parallel-distributed-build.sh

LOCAL_REGISTRY="192.168.1.72:30500"
REMOTE_DOCKER="tcp://192.168.100.2:2375"

# Build on LOCAL machine (background)
echo "🏠 Building pm, healer locally..."
(
  docker build -t $LOCAL_REGISTRY/pm:tilt-dev -f infra/images/pm-server/Dockerfile.build . &&
  docker push $LOCAL_REGISTRY/pm:tilt-dev
) &
LOCAL_PID1=$!

(
  docker build -t $LOCAL_REGISTRY/healer:tilt-dev -f infra/images/healer/Dockerfile.kind . &&
  docker push $LOCAL_REGISTRY/healer:tilt-dev
) &
LOCAL_PID2=$!

# Build on REMOTE machine (background)
echo "🌐 Building controller, tools on remote..."
(
  DOCKER_HOST=$REMOTE_DOCKER docker build -t $LOCAL_REGISTRY/controller:tilt-dev -f infra/images/controller/Dockerfile.kind . &&
  DOCKER_HOST=$REMOTE_DOCKER docker push $LOCAL_REGISTRY/controller:tilt-dev
) &
REMOTE_PID1=$!

(
  DOCKER_HOST=$REMOTE_DOCKER docker build -t $LOCAL_REGISTRY/tools:tilt-dev -f infra/images/tools/Dockerfile.kind . &&
  DOCKER_HOST=$REMOTE_DOCKER docker push $LOCAL_REGISTRY/tools:tilt-dev
) &
REMOTE_PID2=$!

# Wait for all
echo "⏳ Waiting for all builds..."
wait $LOCAL_PID1 $LOCAL_PID2 $REMOTE_PID1 $REMOTE_PID2

echo "✅ All builds complete!"
```

## Troubleshooting

### Thunderbolt Bridge not appearing

```bash
# Check if the interface exists
ifconfig | grep -A5 bridge

# Try disconnecting and reconnecting the cable
```

### "Cannot connect to Docker daemon"

```bash
# Verify remote Docker is listening
nc -zv 192.168.100.2 2375

# Check Docker Desktop is running on remote machine
```

### Slow transfers

```bash
# Test Thunderbolt speed
iperf3 -s  # On remote machine
iperf3 -c 192.168.100.2  # On main machine
# Should see ~10-20 Gbps
```

### Registry push fails from remote

The remote machine needs network access to your registry (`192.168.1.72:30500`).
Ensure both machines are on the same network or the registry is accessible.

## Performance Tips

1. **Increase Docker memory on BOTH machines** - More RAM = faster Rust compilation
2. **Use the remote for heavy builds** - Controller and Tools are the largest
3. **Keep light builds local** - Research, Healer are smaller
4. **Shared cargo registry** - Both machines benefit from BuildKit cache mounts

## Quick Reference

| Machine | IP | Role |
|---------|-----|------|
| Main MacBook | 192.168.100.1 | Runs Tilt, deploys |
| Remote MacBook | 192.168.100.2 | Docker builds |
| Registry | 192.168.1.72:30500 | Receives images |

```bash
# Quick test from main machine
export DOCKER_HOST=tcp://192.168.100.2:2375
docker info | grep "Server Version"
```





