# Talos Kubernetes Home Cluster

This repository contains the configuration for a minimal Talos Kubernetes cluster running on bare metal hardware.

## Current Cluster Configuration

- **Control Plane**: Intel Mac Mini at `192.168.1.77`
- **Worker Node**: Dell system with NVME storage at `192.168.1.72`
- **CNI**: Flannel (simple and reliable)
- **Ingress**: NGINX Ingress Controller with NodePort
- **Storage**: Local Path Provisioner with 100GB NVME volume



## Key Features

✅ **Simple Configuration** - Minimal Talos setup without complex features
✅ **Persistent Storage** - Local Path Provisioner on worker's NVME drive
✅ **NGINX Ingress** - Accessible via NodePort (HTTP: 31251, HTTPS: 31981)
✅ **Stable Networking** - Single NIC configuration with DHCP



## Directory Structure









```
talos-home/
├── config/simple/           # Current working cluster configuration
│   ├── controlplane.yaml    # Control plane configuration
│   ├── worker.yaml          # Worker node configuration
│   └── talosconfig          # Talos client configuration
└── local-path-provisioner/  # Storage provisioner configuration
    ├── kustomization.yaml   # Kustomize deployment configuration
    └── local-path-storage.yaml  # User volume configuration








```



## Quick Start



### Prerequisites



- macOS workstation with M2 chip (or any system with `kubectl` and `talosctl`)


- Two Intel x86-64 systems for the cluster


- USB drive for Talos installation


- Network with DHCP

### Installation

1. **Download Talos ISO**:
   ```bash
   curl -LO https://github.com/siderolabs/talos/releases/download/v1.10.4/metal-amd64.iso







```

2. **Create bootable USB** (on macOS):
   ```bash
   # Find your USB device
   diskutil list

   # Unmount the USB (replace diskX with your disk)
   diskutil unmountDisk /dev/diskX

   # Write the ISO to USB (replace diskX with your disk)
   sudo dd if=metal-amd64.iso of=/dev/rdiskX bs=1m

   # Eject the USB
   diskutil eject /dev/diskX







```

3. **Prepare Hardware**:
   - Control plane: Intel Mac Mini or similar x86-64 system
   - Worker: Intel/AMD system with NVME storage
   - **IMPORTANT**: Disconnect all extra disks from worker, leave only NVME

4. **Boot Control Plane**:


   - Boot Mac Mini from USB into maintenance mode


   - Note the IP address (should get 192.168.1.77 via DHCP)
   - Apply configuration:
     ```bash
     talosctl apply-config --insecure --nodes 192.168.1.77 --file config/simple/controlplane.yaml







```


   - Wait for it to reboot and start up

5. **Bootstrap Cluster** (first time only):
   ```bash
   talosctl --talosconfig=config/simple/talosconfig bootstrap -n 192.168.1.77







```

6. **Boot Worker Node**:


   - Boot Dell/worker from USB into maintenance mode


   - Note the IP address (should get 192.168.1.72 via DHCP)
   - Apply configuration:
     ```bash
     talosctl apply-config --insecure --nodes 192.168.1.72 --file config/simple/worker.yaml







```
   - **IMPORTANT**: Remove USB after configuration is applied


   - System will install to NVME and reboot

7. **Get kubeconfig**:
   ```bash
   talosctl --talosconfig=config/simple/talosconfig kubeconfig -n 192.168.1.77
   # This will save to ~/.kube/config by default







```

8. **Verify Cluster**:
   ```bash
   kubectl get nodes
   # Should show both nodes as Ready







```

9. **Deploy Storage** (Local Path Provisioner):
   ```bash
   # Apply storage volume configuration
   talosctl --talosconfig=config/simple/talosconfig --nodes 192.168.1.72 patch mc --patch @local-path-provisioner/local-path-storage.yaml

   # Deploy Local Path Provisioner
   kubectl apply -k local-path-provisioner/

   # Verify storage
   kubectl get storageclass







```

## Accessing Services

- **Kubernetes API**: `https://192.168.1.77:6443`
- **NGINX Ingress HTTP**: `http://192.168.1.77:31251` or `http://192.168.1.72:31251`
- **NGINX Ingress HTTPS**: `https://192.168.1.77:31981` or `https://192.168.1.72:31981`



## Storage

The cluster uses Local Path Provisioner with a 100GB NVME volume on the worker node. Any PVC created will automatically provision storage from `/var/mnt/local-path-provisioner`.

Example PVC:




```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: my-pvc
spec:
  accessModes:


    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi








```

## Important Notes

1. **Single NVME Disk**: The worker must boot from a single NVME disk to avoid volume mount issues
2. **Network Requirements**: Both nodes must be on the same network with DHCP
3. **Storage Location**: All persistent volumes are stored on the worker node only

## Troubleshooting

### Worker Won't Join


- Ensure the worker is booting from NVME, not USB


- Check that only one disk is connected during initial setup


- Verify network connectivity between nodes

### Pods Stuck in Init


- Usually indicates container runtime issues


- Check `talosctl logs kubelet -n <node-ip>`


- Ensure proper disk configuration



### Storage Issues
- Verify the user volume exists: `talosctl -n 192.168.1.72 get volumestatus`
- Check mount status: `talosctl -n 192.168.1.72 get mountstatus`

## Kubeconfig for External Tools

Generate a kubeconfig for tools like Lens or k9s:




```bash
talosctl --talosconfig=config/simple/talosconfig kubeconfig -n 192.168.1.77 > talos-kubeconfig.yaml








```

## Maintenance

- **Reboot node**: `talosctl -n <node-ip> reboot`
- **Check node health**: `talosctl -n <node-ip> health`
- **View logs**: `talosctl -n <node-ip> logs <service>`

## License

This is a personal homelab project. Feel free to use any configurations as reference for your own setup.
