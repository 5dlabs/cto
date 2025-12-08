# Simple Talos Configuration

This directory contains the working configuration for the Talos Kubernetes cluster.



## Files



- **controlplane.yaml** - Configuration for the control plane node (Mac Mini at 192.168.1.77)


- **worker.yaml** - Configuration for the worker node (Dell with NVME at 192.168.1.72)


- **talosconfig** - Client configuration for talosctl commands

## Key Configuration Choices



1. **Flannel CNI** - Simple, reliable CNI without complexity


2. **Single NIC** - Using `enp4s0` with DHCP on worker


3. **NVME Storage** - Worker installs to `/dev/nvme0n1` for persistent storage


4. **No Complex Features** - No GPU, bonding, or advanced optimizations



## Quick Start

1. Apply control plane config:
   ```bash
   talosctl apply-config --insecure --nodes 192.168.1.77 --file controlplane.yaml







```

2. Apply worker config:
   ```bash
   talosctl apply-config --insecure --nodes 192.168.1.72 --file worker.yaml







```

3. Bootstrap cluster (first time only):
   ```bash
   talosctl --talosconfig talosconfig bootstrap -n 192.168.1.77







```

4. Get kubeconfig:
   ```bash
   talosctl --talosconfig talosconfig kubeconfig -n 192.168.1.77







```

## Network Configuration
- Control Plane: 192.168.1.77 (static)
- Worker: 192.168.1.72 (DHCP)
- Pod Network: 10.244.0.0/16
- Service Network: 10.96.0.0/12

## Important Notes



- Worker MUST boot from NVME disk (`/dev/nvme0n1`), not USB


- Both nodes use DHCP from the local network


- Storage volumes are only available on the worker node


- Ensure only one disk is connected during initial worker setup