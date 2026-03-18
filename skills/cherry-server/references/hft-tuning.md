# HFT & Solana Low-Latency Tuning Reference

Complete kernel, sysctl, and kubelet tuning for low-latency HFT trading and Solana/Agave RPC on Talos Linux.

## Kernel Arguments

All set via `machine.install.extraKernelArgs` in controlplane.yaml:

### CPU Performance
| Arg | Value | Why |
|-----|-------|-----|
| `cpufreq.default_governor` | `performance` | Max clock speed always, no frequency scaling |
| `idle` | `poll` | Never enter C-states, CPU always ready |
| `processor.max_cstate` | `0` | Belt-and-suspenders with idle=poll |
| `isolcpus` | `16-31` | Reserve cores 16-31 for trading/Agave pods exclusively |
| `nohz_full` | `16-31` | Disable scheduler tick on isolated cores |
| `rcu_nocbs` | `16-31` | Move RCU callbacks off isolated cores |
| `preempt` | `full` | Full kernel preemption for lowest latency |

### Timekeeping & Interrupts
| Arg | Value | Why |
|-----|-------|-----|
| `tsc` | `reliable` | Trust TSC, avoid fallback to HPET (100ns+ overhead) |
| `clocksource` | `tsc` | Use TSC as clocksource directly |
| `nmi_watchdog` | `0` | Disable NMI watchdog, eliminates periodic latency spikes |
| `nosoftlockup` | (flag) | Disable soft lockup detector |
| `skew_tick` | `1` | Stagger timer interrupts across CPUs, reduce lock contention |

### Memory
| Arg | Value | Why |
|-----|-------|-----|
| `hugepagesz` | `2M` | 2MB huge pages |
| `hugepages` | `2048` | 4 GB of 2MB huge pages pre-allocated |
| `transparent_hugepage` | `madvise` | App opt-in only (Agave uses madvise) |

### Security (Traded for Performance)
| Arg | Value | Why |
|-----|-------|-----|
| `mitigations` | `off` | Disable Spectre/Meltdown mitigations (~5-30% perf gain) |
| `audit` | `0` | Disable kernel audit framework (saves syscall overhead) |

### I/O & Hardware
| Arg | Value | Why |
|-----|-------|-----|
| `nvme_core.io_timeout` | `4294967295` | Infinite NVMe timeout (Agave can stall on snapshot loads) |
| `iommu.strict` | `0` | Bypass IOMMU overhead |

## Sysctls

All set via `machine.sysctls` in controlplane.yaml:

### Network — Solana UDP Gossip (ports 8000-10000)
```yaml
net.core.rmem_max: "134217728"           # 128 MB receive buffer
net.core.wmem_max: "134217728"           # 128 MB send buffer
net.core.rmem_default: "134217728"
net.core.wmem_default: "134217728"
net.ipv4.udp_mem: "65536 131072 262144"
```

### Network — TCP Tuning (exchange APIs, RPC serving)
```yaml
net.ipv4.tcp_rmem: "4096 1048576 67108864"
net.ipv4.tcp_wmem: "4096 1048576 67108864"
net.ipv4.tcp_fastopen: "3"               # Both client and server TFO
net.ipv4.tcp_tw_reuse: "1"
net.ipv4.tcp_fin_timeout: "10"
net.ipv4.tcp_low_latency: "1"            # Prefer latency over throughput
```

### Network — Low-Latency Socket Polling
```yaml
net.core.busy_poll: "50"                 # Busy-poll 50µs on sockets
net.core.busy_read: "50"                 # Busy-read 50µs on sockets
# Trades CPU cycles for ~10-50µs latency reduction per socket operation
```

### Network — Connection Handling
```yaml
net.core.somaxconn: "8192"
net.core.netdev_max_backlog: "10000"
net.ipv4.tcp_max_syn_backlog: "8192"
```

### File Descriptors (Agave opens thousands)
```yaml
fs.file-max: "2097152"
fs.nr_open: "2097152"
```

### Memory
```yaml
vm.max_map_count: "2000000"              # Agave mmap for accounts
vm.swappiness: "1"                       # Almost never swap
vm.dirty_background_ratio: "5"
vm.dirty_ratio: "10"
```

### Scheduler
Note: `kernel.numa_balancing` and `kernel.sched_min_granularity_ns` do NOT exist in Talos kernel 6.12.18. These are compile-time CONFIG options not enabled in the Talos kernel. Do not include them — they will silently fail.

### eBPF (for Cilium)
```yaml
net.core.bpf_jit_enable: "1"
net.core.bpf_jit_harden: "0"
```

### IP Forwarding (for Cilium native routing)
```yaml
net.ipv4.conf.all.forwarding: "1"
net.ipv6.conf.all.forwarding: "1"
```

## Kubelet Configuration

Set via `machine.kubelet.extraConfig` in controlplane.yaml:

```yaml
kubelet:
  extraConfig:
    cpuManagerPolicy: static              # Pin Guaranteed QoS pods to specific CPUs
    cpuManagerReconcilePeriod: 5s
    topologyManagerPolicy: single-numa-node  # CPU+memory on same NUMA node
    reservedSystemCPUs: "0-15"            # System gets cores 0-15, pods get 16-31
    evictionHard:
      memory.available: "1Gi"
      nodefs.available: "5%"
```

**Why `cpuManagerPolicy: static` matters:** Without this, `isolcpus=16-31` is wasted. K8s won't pin pods to isolated cores unless the kubelet uses the static CPU manager. Pods must be Guaranteed QoS (requests == limits) to get exclusive CPU access.

**Do NOT use `memoryManagerPolicy: Static`** unless you also configure `reservedMemory` per NUMA node. Without it, kubelet crashes with "total amount of type memory is not equal to the value determined by Node Allocatable".

## Applying Kernel Arg Changes

`machine.install.extraKernelArgs` only takes effect during install — NOT on config reboot. To update kernel args on an already-installed system:

```bash
# 1. Update controlplane.yaml with new extraKernelArgs
# 2. Apply the config
talosctl apply-config --mode auto ...
# 3. Trigger a reinstall (upgrade to same version)
talosctl upgrade --image factory.talos.dev/installer/{SCHEMATIC}:{VERSION} --preserve
```

The `--preserve` flag keeps etcd data and user disks intact.

## Kernel Modules

```yaml
kernel:
  modules:
    - name: br_netfilter    # Required for Cilium
    - name: ip_tables        # Required for Cilium
    - name: nvme             # NVMe driver
```

## What We Intentionally Did NOT Do

| Tuning | Why Skipped |
|--------|-------------|
| `nosmt` (disable hyperthreading) | Trades 50% CPU capacity for ~5% latency improvement. Not worth it on 32-core with mixed workloads (Solana + trading bots) |
| IRQ affinity pinning | Talos doesn't expose `irqbalance` config easily. The isolated cores + nohz_full handles most of it. Can be added via DaemonSet if needed |
| Ring buffer tuning (ethtool -G) | Requires runtime commands, not declarative in Talos. Can be added via machine config `machine.network.interfaces[].ringSize` if needed |
| Interrupt coalescing (ethtool -C) | Same — requires runtime commands. Low priority given busy_poll handles latency |
| Network namespacing / DPDK | Overkill for our throughput levels. Native kernel networking with eBPF is sufficient |
