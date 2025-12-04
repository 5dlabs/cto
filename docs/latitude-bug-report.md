# Latitude Support: Orphaned Servers Bug Report

**Subject:** Bug Report: Deleted servers still running and billing (orphaned resources)

---

Hi Latitude Support,

I've encountered a bug where servers that were deleted via the API are still running and being billed, but no longer appear in the dashboard or API responses.

**Issue Summary:**
- Deleted two servers via `DELETE /servers/{id}` API calls
- API returned success
- Servers disappeared from dashboard and `GET /servers` returns empty
- However, the servers are **still running** and accessible via their IPs
- Billing API shows continued metering for these resources

**Affected Resources:**
- **IP:** 64.34.91.85 (was `cto-final-cp1`, server ID: `sv_ZozMaznL2a7kw`)
- **IP:** 64.34.91.91 (was `cto-final-worker1`, server ID: `sv_MDEOaPBlyNwgB`)
- **Project ID:** `proj_bBmw0KKxQ09VR`

---

## API Logs

### LOG 1: List Servers (returns empty)
```
REQUEST: GET https://api.latitude.sh/servers
RESPONSE:
{
  "data": [],
  "meta": {}
}
```

### LOG 2: Attempt to delete server 1 (returns 404)
```
REQUEST: DELETE https://api.latitude.sh/servers/sv_ZozMaznL2a7kw
RESPONSE:
{
  "errors": [
    {
      "code": "not_found",
      "status": "404",
      "title": "Error",
      "detail": "Specified Record Not Found",
      "meta": {}
    }
  ]
}
```

### LOG 3: Attempt to delete server 2 (returns 404)
```
REQUEST: DELETE https://api.latitude.sh/servers/sv_MDEOaPBlyNwgB
RESPONSE:
{
  "errors": [
    {
      "code": "not_found",
      "status": "404",
      "title": "Error",
      "detail": "Specified Record Not Found",
      "meta": {}
    }
  ]
}
```

### LOG 4: Proof servers are still running
```
Connectivity test to 64.34.91.85:
  Port 6443 (K8s API): OPEN
  Port 50000 (Talos API): OPEN

Connectivity test to 64.34.91.91:
  Port 50000 (Talos API): OPEN

Kubernetes cluster response (via kubectl):
NAME                  STATUS   ROLES           AGE    VERSION
hardware-197s004830   Ready    control-plane   130m   v1.33.1
hardware-197s006203   Ready    <none>          127m   v1.33.1
```

### LOG 5: Billing shows continued usage
```
REQUEST: GET https://api.latitude.sh/billing/usage?filter[project]=proj_bBmw0KKxQ09VR
RESPONSE (summarized):
{
  "period": {
    "start": "2025-12-03T06:20:17+00:00",
    "end": "2026-01-03T06:20:17+00:00"
  },
  "total_amount_cents": 396,
  "credit_balance_cents": -30000,
  "num_products": 12
}
```

---

**Request:**
1. Please terminate the orphaned servers at IPs 64.34.91.85 and 64.34.91.91
2. Credit back any charges incurred after the original deletion request
3. Investigate the root cause so this doesn't happen to other customers

Thank you



