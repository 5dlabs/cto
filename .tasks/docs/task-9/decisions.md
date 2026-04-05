## Decision Points

- What Cloudflare plan tier is available, and which CDN features (e.g., WAF, Bot Management, Advanced DDoS) should be enabled for public endpoints?
- Should Cloudflare Tunnel be deployed as a single replicated Deployment or as a DaemonSet for node-level redundancy?

## Coordination Notes

- Agent owner: Bolt
- Primary stack: Kubernetes/Helm