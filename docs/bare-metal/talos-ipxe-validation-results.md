# Talos iPXE Boot Validation Results

**Date**: 2025-12-03
**Status**: ✅ SUCCESS - All Go/No-Go Criteria Passed

## Summary

Successfully validated that Talos Linux can boot on Latitude.sh bare metal servers via iPXE using the Talos Image Factory. This de-risks the critical assumption for the bare metal POC.

## Go/No-Go Criteria Results

| Criteria | Result | Notes |
|----------|--------|-------|
| Latitude API accepts Talos iPXE URL | ✅ PASS | API returned success, server entered reinstall flow |
| Server boots into Talos maintenance mode | ✅ PASS | Talos booted from network successfully |
| `talosctl` can connect on port 50000 | ✅ PASS | Full API access in maintenance mode |
| DHCP assigns IP automatically | ✅ PASS | IPv4 and IPv6 assigned via DHCP |

## Test Configuration

- **Plan**: `c2-small-x86` (4 cores, 32GB RAM, $0.18/hr)
- **Region**: MIA2 (Miami)
- **Server ID**: `sv_W9EKa36Bv5RoB`
- **Public IPv4**: `64.34.91.61`
- **Talos Version**: v1.9.0
- **iPXE URL**: `https://pxe.factory.talos.dev/pxe/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.9.0/metal-amd64`
- **Schematic**: Vanilla (default, no extensions)

## Timeline

1. **Server Provisioned**: ~5 minutes (initial Ubuntu deployment)
2. **iPXE Reinstall Triggered**: API accepted immediately
3. **Disk Erasing Phase**: ~10 minutes
4. **Talos Boot**: ~2 minutes after disk erase
5. **Total Time**: ~17 minutes from provision to Talos responding

## Verified Talos Functionality

### Network Interfaces (from `talosctl get links`)
```
enp1s0f0   ether   ac:1f:6b:f5:0a:74   up   true
enp1s0f1   ether   ac:1f:6b:f5:0a:75   up   true
```

### IP Addresses (from `talosctl get addresses`)
```
enp1s0f0/64.34.91.61/31                            (IPv4 - DHCP)
enp1s0f0/2605:6440:100a:a1:ae1f:6bff:fef5:a74/64   (IPv6 - SLAAC)
```

## Lessons Learned

1. **SSH Keys Required**: Initial server provisioning failed without SSH keys configured. Always add SSH keys to the project before provisioning.

2. **Region Matters**: First two servers (DAL, LAX) had IPMI issues. MIA2 worked immediately. Consider region failover in automation.

3. **iPXE Plan Limitation**: Only `c2-small-x86` supports iPXE in US regions. The cheaper `c1-tiny-x86` ($0.09/hr) does NOT support iPXE.

4. **Maintenance Mode**: Talos boots into maintenance mode from iPXE (expected). To persist to disk, a machine config must be applied via `talosctl apply-config`.

5. **Latitude Status**: The Latitude API status remains "disk_erasing" or "deploying" during iPXE boot - it doesn't reflect the actual Talos state. Must use `talosctl` to verify boot.

6. **Boot Loop**: Without applying a machine config, the server may reboot back into the reinstall flow. The iPXE boot is RAM-based only.

## Cost

- **Estimated**: ~$0.09 (30 min at $0.18/hr)
- **Actual**: ~$0.05 (~17 min active time)

## Next Steps

With validation complete, proceed to Phase 2: Rust Implementation

1. Implement `LatitudeClient` in `crates/installer/src/providers/latitude/`
2. Add methods for:
   - `create_server()` - Provision with Ubuntu
   - `wait_for_ready()` - Poll until status "on"
   - `reinstall_ipxe()` - Trigger Talos iPXE boot
   - `delete_server()` - Cleanup
3. Integrate with Talos machine config generation
4. Add full cluster bootstrap flow

## API Reference

### Create Server
```bash
curl -X POST "https://api.latitude.sh/servers" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "type": "servers",
      "attributes": {
        "project": "<PROJECT_ID>",
        "plan": "c2-small-x86",
        "site": "MIA2",
        "operating_system": "ubuntu_24_04_x64_lts",
        "hostname": "talos-node",
        "ssh_keys": ["<SSH_KEY_ID>"]
      }
    }
  }'
```

### Trigger iPXE Reinstall
```bash
curl -X POST "https://api.latitude.sh/servers/<SERVER_ID>/reinstall" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "type": "reinstalls",
      "attributes": {
        "operating_system": "ipxe",
        "hostname": "talos-node",
        "ipxe": "https://pxe.factory.talos.dev/pxe/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.9.0/metal-amd64"
      }
    }
  }'
```

### Delete Server
```bash
curl -X DELETE "https://api.latitude.sh/servers/<SERVER_ID>" \
  -H "Authorization: Bearer $API_KEY"
```

