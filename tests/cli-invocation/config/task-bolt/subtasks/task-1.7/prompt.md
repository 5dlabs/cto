# Configure Pod Security Policies

<task>
<agent>pod-security-agent</agent>
<objective>Configure pod security standards for infrastructure workloads</objective>

<context>
Infrastructure components are deployed. Your job is to configure pod security policies/standards to ensure workloads run with appropriate security constraints.
</context>

<requirements>
- Configure Pod Security Standards (PSS) labels on namespaces
- Set security contexts for database pods (non-root, read-only fs where possible)
- Configure seccomp profiles if applicable
- Ensure no privileged containers unless absolutely required
</requirements>

<deliverables>
- `namespace-pss.yaml` - Namespace PSS label configuration
- `pod-security-contexts.yaml` - Security context patches/overlays
- Applied to cluster
</deliverables>

<acceptance_criteria>
- [ ] Namespaces have appropriate PSS labels (baseline/restricted)
- [ ] Pods run as non-root where possible
- [ ] No unnecessary privileged containers
- [ ] Security contexts are explicitly defined
</acceptance_criteria>
</task>
