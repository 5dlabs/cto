Implement subtask 10008: Network policies: allow sigma1 egress to external APIs

## Objective
Create egress NetworkPolicy allowing sigma1 services to reach external APIs (OpenCorporates, Stripe, Google, etc.) and DNS resolution.

## Steps
Step-by-step:
1. Create `netpol-allow-external-egress.yaml`:
   - Since external API IPs change, use a broad egress allow for HTTPS (port 443) to `0.0.0.0/0` but exclude cluster CIDRs to prevent lateral movement:
     ```yaml
     egress:
       - to:
           - ipBlock:
               cidr: 0.0.0.0/0
               except:
                 - 10.0.0.0/8
                 - 172.16.0.0/12
                 - 192.168.0.0/16
         ports:
           - protocol: TCP
             port: 443
     ```
   - Apply to specific pods that need external access: customer-vetting (OpenCorporates), finance (Stripe), social-engine (Signal-CLI outbound if applicable).
2. Separately ensure DNS egress is allowed (port 53 to kube-dns) — this should be in the db-access policy or a shared DNS policy.
3. Create a dedicated `netpol-allow-dns.yaml` that allows all sigma1 pods egress to kube-dns on port 53 TCP/UDP.

## Validation
From the customer-vetting pod, run `curl -I https://api.opencorporates.com` and verify a response is received. From the equipment-catalog pod (if it doesn't need external access), verify the same curl times out or is denied. Verify DNS resolution works from all sigma1 pods.